use std::{
    io,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{Receiver, Sender, channel},
    },
    thread::{self, JoinHandle},
};

use ignore::{DirEntry, WalkBuilder, overrides::OverrideBuilder};
use nucleo::{
    Match, Matcher, Utf32String,
    pattern::{CaseMatching, MultiPattern, Normalization},
};

#[derive(Debug, Clone, Default)]
pub struct FuzzyMatchResult {
    // Path of the matched entry.
    pub path: Utf32String,
    /// Matcher score, higher is better.
    pub score: u32,
    /// Matched indices of characters.
    pub indices: Vec<u32>,
    /// Is it a directory.
    pub is_dir: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FuzzyMatcherStatus {
    pub changed: bool,
    pub done: bool,
}

#[derive(Debug, Clone)]
struct MatchEntry {
    path: Utf32String,
    is_dir: bool,
}

/// A fuzzy matcher whose filesystem walk and scoring run on one daemon thread.
///
/// Nucleo's `Nucleo` engine unconditionally panics when rayon cannot create its
/// worker pool. Keeping the matcher synchronous inside our existing daemon
/// preserves a responsive UI while making OS thread exhaustion a recoverable
/// loss of suggestions instead of a process abort.
pub struct FuzzyFileMatcher {
    root: PathBuf,
    query: String,
    pattern: MultiPattern,
    matcher: Matcher,
    entries: Vec<MatchEntry>,
    matches: Vec<Match>,
    top_entries: Vec<FuzzyMatchResult>,
    dirs: bool,
    changed: bool,
}

impl FuzzyFileMatcher {
    /// Create a matcher without allocating an OS thread or rayon pool.
    pub fn new(root: &Path) -> Self {
        let matcher_config = nucleo::Config::DEFAULT.match_paths();
        Self {
            root: root.to_owned(),
            pattern: MultiPattern::new(1),
            matcher: Matcher::new(matcher_config),
            entries: Vec::new(),
            matches: Vec::new(),
            query: String::new(),
            top_entries: Vec::new(),
            dirs: false,
            changed: false,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    /// Refresh the indexed paths on the daemon thread.
    pub fn restart_walk_custom(
        &mut self,
        make_walker: impl FnOnce(&mut WalkBuilder) -> &mut WalkBuilder,
    ) {
        let completed = self.restart_walk_custom_controlled(make_walker, || false, |_| {});
        debug_assert!(completed, "an uncancelled walk must complete");
    }

    /// Run a sequential walk that can be superseded without creating more OS
    /// workers. The depth-one snapshot is published first; the caller can then
    /// show useful empty-query results while the complete tree is indexed.
    fn restart_walk_custom_controlled(
        &mut self,
        make_walker: impl FnOnce(&mut WalkBuilder) -> &mut WalkBuilder,
        mut is_cancelled: impl FnMut() -> bool,
        mut top_entries_ready: impl FnMut(&mut Self),
    ) -> bool {
        let walker_builder = make_walker(
            WalkBuilder::new(&self.root)
                .follow_links(false)
                .git_ignore(true)
                .git_global(true)
                .git_exclude(true)
                .ignore(true)
                .hidden(true)
                .require_git(false)
                .overrides(
                    OverrideBuilder::new(&self.root)
                        .add("!.git")
                        .unwrap()
                        .build()
                        .unwrap(),
                ),
        )
        .clone();

        fn check_entry<'a>(entry: &'a DirEntry, root: &Path) -> Option<(&'a str, bool)> {
            let path = entry.path();
            if path != root
                && let Some(file_type) = entry.file_type()
                && (file_type.is_file() || file_type.is_dir())
                && let Ok(path) = path.strip_prefix(root)
                && let Some(path) = path.as_os_str().to_str()
                && !path.is_empty()
            {
                Some((path, file_type.is_dir()))
            } else {
                None
            }
        }

        let mut top_entries = Vec::new();
        for entry in walker_builder
            .clone()
            .max_depth(Some(1))
            .sort_by_file_name(|a, b| a.cmp(b))
            .build()
        {
            if is_cancelled() {
                return false;
            }
            let Ok(entry) = entry else {
                continue;
            };
            let Some((path, is_dir)) = check_entry(&entry, &self.root) else {
                continue;
            };
            top_entries.push(FuzzyMatchResult {
                path: path.into(),
                score: 0,
                indices: Vec::new(),
                is_dir,
            });
        }

        if is_cancelled() {
            return false;
        }
        self.top_entries = top_entries;
        self.entries.clear();
        self.matches.clear();
        self.changed = true;
        top_entries_ready(self);

        let mut entries = Vec::new();
        for entry in walker_builder.build() {
            if is_cancelled() {
                return false;
            }
            let Ok(entry) = entry else {
                continue;
            };
            let Some((path, is_dir)) = check_entry(&entry, &self.root) else {
                continue;
            };
            entries.push(MatchEntry {
                path: path.into(),
                is_dir,
            });
        }

        if is_cancelled() {
            return false;
        }
        self.entries = entries;
        self.recompute_matches();
        self.changed = true;
        true
    }

    /// Restart the walk with default walker parameters.
    pub fn restart_walk(&mut self) {
        self.restart_walk_custom(|w| w);
    }

    /// Set the query to a given string and score the current path index.
    pub fn set_query(&mut self, mut query: &str, dirs: bool) {
        self.dirs = dirs;
        if dirs && query.ends_with('/') {
            query = &query[..query.len() - 1];
        }
        if query == self.query {
            return;
        }
        // see this re: backslash etc: https://github.com/helix-editor/nucleo/pull/87
        let append = query.as_bytes().starts_with(self.query.as_bytes())
            && !query.ends_with('\\')
            && !query
                .as_bytes()
                .last()
                .is_some_and(|ch| ch.is_ascii_whitespace());
        self.pattern
            .reparse(0, query, CaseMatching::Smart, Normalization::Smart, append);
        self.query = query.to_owned();
        self.recompute_matches();
        self.changed = true;
    }

    fn recompute_matches(&mut self) {
        self.matches.clear();
        if self.query.is_empty() {
            return;
        }
        for (idx, entry) in self.entries.iter().enumerate() {
            if let Some(score) = self
                .pattern
                .score(std::slice::from_ref(&entry.path), &mut self.matcher)
            {
                self.matches.push(Match {
                    score,
                    idx: idx as u32,
                });
            }
        }
        let entries = &self.entries;
        self.matches.sort_by(|left, right| {
            right.score.cmp(&left.score).then_with(|| {
                let left_path = &entries[left.idx as usize].path;
                let right_path = &entries[right.idx as usize].path;
                left_path
                    .len()
                    .cmp(&right_path.len())
                    .then_with(|| left_path.cmp(right_path))
            })
        });
    }

    /// Report completion to the daemon. Scoring is already complete here.
    pub fn tick(&mut self, _tick_timeout_ms: u64) -> FuzzyMatcherStatus {
        FuzzyMatcherStatus {
            done: true,
            changed: std::mem::take(&mut self.changed),
        }
    }

    /// Total number of currently matched items.
    pub fn num_items(&self) -> usize {
        if self.query.is_empty() {
            self.top_entries.len()
        } else {
            self.matches.len()
        }
    }

    /// Get top `k` items sorted by score, path length, and path.
    pub fn get_top_k(&mut self, k: usize) -> Vec<FuzzyMatchResult> {
        if self.query.is_empty() {
            return self
                .top_entries
                .iter()
                .filter(|entry| !self.dirs || entry.is_dir)
                .take(k)
                .cloned()
                .collect();
        }

        // https://github.com/helix-editor/helix/blob/d79cce4e4bfc24dd204f1b294c899ed73f7e9453/helix-term/src/ui/completion.rs#L369
        let min_score = 7 + self.query.chars().count() as u32 * 14;
        let pattern = self.pattern.column_pattern(0);
        self.matches
            .iter()
            .take_while(|matched| matched.score >= min_score)
            .filter_map(|matched| {
                let entry = &self.entries[matched.idx as usize];
                if self.dirs && !entry.is_dir {
                    return None;
                }
                let mut indices = Vec::new();
                if !pattern.atoms.is_empty() {
                    pattern.indices(entry.path.slice(..), &mut self.matcher, &mut indices);
                }
                Some(FuzzyMatchResult {
                    path: entry.path.clone(),
                    score: matched.score,
                    indices,
                    is_dir: entry.is_dir,
                })
            })
            .take(k)
            .collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct FuzzyMatcherDaemonResults {
    pub topk: Arc<[FuzzyMatchResult]>,
    pub num_items: usize,
    pub status: FuzzyMatcherStatus,
    pub generation: usize,
}

impl AsRef<[FuzzyMatchResult]> for FuzzyMatcherDaemonResults {
    fn as_ref(&self) -> &[FuzzyMatchResult] {
        self.topk.as_ref()
    }
}

#[derive(Debug, Clone)]
enum FuzzyMatcherDaemonMessage {
    RestartWalk {
        hidden: bool,
        walk_revision: usize,
        query: Option<(String, bool)>,
    },
    SetQuery {
        query: String,
        dirs: bool,
    },
    Stop,
}

#[derive(Default)]
struct FuzzyMatcherDaemonBatch {
    restart: Option<(bool, usize)>,
    query: Option<(String, bool)>,
    generation_steps: usize,
    stop: bool,
}

fn drain_daemon_messages(
    first: FuzzyMatcherDaemonMessage,
    rx: &Receiver<FuzzyMatcherDaemonMessage>,
) -> FuzzyMatcherDaemonBatch {
    let mut batch = FuzzyMatcherDaemonBatch::default();
    for message in std::iter::once(first).chain(rx.try_iter()) {
        match message {
            FuzzyMatcherDaemonMessage::RestartWalk {
                hidden,
                walk_revision,
                query,
            } => {
                batch.restart = Some((hidden, walk_revision));
                if let Some(query) = query {
                    batch.query = Some(query);
                }
                batch.generation_steps += 1;
            }
            FuzzyMatcherDaemonMessage::SetQuery { query, dirs } => {
                batch.query = Some((query, dirs));
                batch.generation_steps += 1;
            }
            FuzzyMatcherDaemonMessage::Stop => {
                batch.stop = true;
                break;
            }
        }
    }
    batch
}

fn publish_daemon_results(
    matcher: &mut FuzzyFileMatcher,
    topk_limit: usize,
    results: &Arc<Mutex<FuzzyMatcherDaemonResults>>,
    generation: usize,
    done: bool,
) {
    let num_items = matcher.num_items();
    let topk: Arc<[_]> = matcher.get_top_k(topk_limit).into();
    *results.lock().unwrap() = FuzzyMatcherDaemonResults {
        topk,
        num_items,
        status: FuzzyMatcherStatus {
            changed: true,
            done,
        },
        generation,
    };
}

pub struct FuzzyFileMatcherDaemon {
    results: Arc<Mutex<FuzzyMatcherDaemonResults>>,
    tx: Sender<FuzzyMatcherDaemonMessage>,
    walk_revision: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
    _handle: JoinHandle<()>,
}

fn retain_daemon_thread(result: io::Result<JoinHandle<()>>) -> Option<JoinHandle<()>> {
    match result {
        Ok(handle) => Some(handle),
        Err(error) => {
            tracing::warn!(
                error = %error,
                error_kind = ?error.kind(),
                "file search disabled because the OS could not create its daemon thread"
            );
            None
        }
    }
}

impl FuzzyFileMatcherDaemon {
    pub fn new(mut matcher: FuzzyFileMatcher, topk: usize) -> Option<Self> {
        let results = Arc::new(Mutex::new(FuzzyMatcherDaemonResults::default()));
        let walk_revision = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        // Standard mpsc sends do not wait for the daemon while a filesystem
        // walk is in progress; the receiver coalesces queued updates to latest.
        let (tx, rx) = channel();

        let thread_results = results.clone();
        let thread_walk_revision = walk_revision.clone();
        let thread_stop = stop.clone();
        let handle = retain_daemon_thread(
            thread::Builder::new()
                .name("grok-fuzzy-files".to_owned())
                .spawn(move || {
                    let mut generation = 0;
                    while let Ok(first) = rx.recv() {
                        let batch = drain_daemon_messages(first, &rx);
                        if batch.stop {
                            break;
                        }
                        generation += batch.generation_steps;
                        let restarting = batch.restart.is_some();
                        if restarting && let Some((query, dirs)) = batch.query.as_ref() {
                            // Apply the latest coalesced query before walking so
                            // the depth-one publication cannot briefly show
                            // unfiltered entries for a non-empty query.
                            matcher.set_query(query, *dirs);
                        }

                        if let Some((hidden, expected_walk_revision)) = batch.restart {
                            *thread_results.lock().unwrap() = FuzzyMatcherDaemonResults {
                                generation,
                                ..FuzzyMatcherDaemonResults::default()
                            };
                            let cancelled = || {
                                thread_stop.load(Ordering::Relaxed)
                                    || thread_walk_revision.load(Ordering::Relaxed)
                                        != expected_walk_revision
                            };
                            let publish_top = |matcher: &mut FuzzyFileMatcher| {
                                publish_daemon_results(
                                    matcher,
                                    topk,
                                    &thread_results,
                                    generation,
                                    false,
                                );
                            };
                            let completed = if hidden {
                                tracing::trace!("restarting hidden walk");
                                matcher.restart_walk_custom_controlled(
                                    |walker| walker.hidden(false).ignore(false).git_ignore(false),
                                    cancelled,
                                    publish_top,
                                )
                            } else {
                                tracing::trace!("restarting normal walk");
                                matcher.restart_walk_custom_controlled(
                                    |walker| walker,
                                    cancelled,
                                    publish_top,
                                )
                            };
                            if !completed {
                                continue;
                            }
                        }

                        if !restarting && let Some((query, dirs)) = batch.query {
                            matcher.set_query(&query, dirs);
                        }
                        let status = matcher.tick(0);
                        publish_daemon_results(
                            &mut matcher,
                            topk,
                            &thread_results,
                            generation,
                            status.done,
                        );
                    }
                }),
        )?;

        Some(Self {
            results,
            tx,
            walk_revision,
            stop,
            _handle: handle,
        })
    }

    pub fn get(&self) -> FuzzyMatcherDaemonResults {
        self.results.lock().unwrap().clone()
    }

    pub fn set_query(&self, query: impl AsRef<str>, dirs: bool) {
        let query = query.as_ref().to_owned();
        _ = self
            .tx
            .send(FuzzyMatcherDaemonMessage::SetQuery { query, dirs })
            .ok();
    }

    pub fn restart_walk(&self, hidden: bool) {
        self.send_restart(hidden, None);
    }

    /// Atomically restart the index and apply the query under one generation.
    /// This prevents a daemon wakeup between separate restart/query messages
    /// from publishing old-query results that the caller could accept as fresh.
    pub fn restart_walk_with_query(&self, hidden: bool, query: impl AsRef<str>, dirs: bool) {
        self.send_restart(hidden, Some((query.as_ref().to_owned(), dirs)));
    }

    fn send_restart(&self, hidden: bool, query: Option<(String, bool)>) {
        let walk_revision = self.walk_revision.fetch_add(1, Ordering::Relaxed) + 1;
        _ = self
            .tx
            .send(FuzzyMatcherDaemonMessage::RestartWalk {
                hidden,
                walk_revision,
                query,
            })
            .ok();
    }
}

impl Drop for FuzzyFileMatcherDaemon {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        _ = self.tx.send(FuzzyMatcherDaemonMessage::Stop).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synchronous_matcher_scores_files_without_a_worker_pool() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("nested")).unwrap();
        std::fs::write(root.path().join("nested/needle.txt"), "test").unwrap();
        std::fs::write(root.path().join("other.txt"), "test").unwrap();

        let mut matcher = FuzzyFileMatcher::new(root.path());
        matcher.restart_walk();
        matcher.set_query("needle", false);

        assert_eq!(matcher.num_items(), 1);
        let status = matcher.tick(0);
        assert!(status.done);
        assert!(status.changed);
        let matches = matcher.get_top_k(10);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].path.to_string(), "nested/needle.txt");
    }

    #[test]
    fn directory_filter_is_applied_before_the_top_k_limit() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("match"), "test").unwrap();
        std::fs::create_dir(root.path().join("match-directory")).unwrap();

        let mut matcher = FuzzyFileMatcher::new(root.path());
        matcher.restart_walk();
        matcher.set_query("match", true);

        let matches = matcher.get_top_k(1);
        assert_eq!(matches.len(), 1);
        assert!(matches[0].is_dir);
        assert_eq!(matches[0].path.to_string(), "match-directory");
    }

    #[test]
    fn equal_scores_are_ordered_by_path_length_then_path() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("a")).unwrap();
        std::fs::create_dir(root.path().join("b")).unwrap();
        std::fs::write(root.path().join("a/needle.txt"), "test").unwrap();
        std::fs::write(root.path().join("b/needle.txt"), "test").unwrap();

        let mut matcher = FuzzyFileMatcher::new(root.path());
        matcher.restart_walk();
        matcher.set_query("needle", false);

        let paths: Vec<String> = matcher
            .get_top_k(10)
            .into_iter()
            .filter(|entry| !entry.is_dir)
            .map(|entry| entry.path.to_string())
            .collect();
        assert_eq!(paths, ["a/needle.txt", "b/needle.txt"]);
    }

    #[test]
    fn restart_with_query_is_one_atomic_generation() {
        let (_sender, receiver) = channel();
        let batch = drain_daemon_messages(
            FuzzyMatcherDaemonMessage::RestartWalk {
                hidden: true,
                walk_revision: 7,
                query: Some(("needle".to_owned(), false)),
            },
            &receiver,
        );

        assert_eq!(batch.restart, Some((true, 7)));
        assert_eq!(batch.query, Some(("needle".to_owned(), false)));
        assert_eq!(batch.generation_steps, 1);
    }

    #[test]
    fn daemon_thread_creation_failure_is_nonfatal() {
        let error = io::Error::from(io::ErrorKind::WouldBlock);
        let failed: io::Result<JoinHandle<()>> = Err(error);

        assert!(retain_daemon_thread(failed).is_none());
    }
}
