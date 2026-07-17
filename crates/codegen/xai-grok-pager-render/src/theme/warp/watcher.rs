use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::{mpsc, watch};

pub struct WarpThemeWatcher {
    rx: watch::Receiver<u64>,
    _watcher: RecommendedWatcher,
    handle: tokio::task::JoinHandle<()>,
}

impl WarpThemeWatcher {
    pub fn start(paths: impl IntoIterator<Item = PathBuf>) -> Option<Self> {
        let targets = watch_targets(paths);
        if targets.is_empty() {
            return None;
        }
        let (raw_tx, mut raw_rx) = mpsc::unbounded_channel::<()>();
        let mut watcher = RecommendedWatcher::new(
            move |event: notify::Result<notify::Event>| {
                if event.is_ok() {
                    let _ = raw_tx.send(());
                }
            },
            Config::default(),
        )
        .ok()?;
        let mut watched_any = false;
        for (target, recursive) in targets {
            let mode = if recursive {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            if watcher.watch(&target, mode).is_ok() {
                watched_any = true;
            }
        }
        if !watched_any {
            return None;
        }

        let (tx, rx) = watch::channel(0u64);
        let handle = tokio::spawn(async move {
            let mut generation = 0u64;
            while raw_rx.recv().await.is_some() {
                tokio::time::sleep(Duration::from_millis(150)).await;
                while raw_rx.try_recv().is_ok() {}
                generation = generation.wrapping_add(1);
                let _ = tx.send(generation);
            }
        });
        Some(Self {
            rx,
            _watcher: watcher,
            handle,
        })
    }

    pub async fn changed(&mut self) -> Result<(), watch::error::RecvError> {
        self.rx.changed().await
    }
}

impl Drop for WarpThemeWatcher {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

fn watch_targets(paths: impl IntoIterator<Item = PathBuf>) -> Vec<(PathBuf, bool)> {
    let mut unique = HashMap::<PathBuf, bool>::new();
    for path in paths {
        // Watch files through their parent so atomic replace/rename keeps the
        // watch alive. Existing theme directories are recursive. A missing
        // path may watch its immediate parent when that parent exists, but we
        // deliberately do not climb toward HOME/XDG roots: an absent Warp
        // channel must not turn every unrelated home-directory event into a
        // catalog refresh.
        let (target, recursive) = if path.is_dir() {
            (path, true)
        } else if path.is_file() {
            (
                path.parent()
                    .map(|parent| parent.to_path_buf())
                    .unwrap_or(path),
                false,
            )
        } else if let Some(parent) = path.parent().filter(|parent| parent.is_dir()) {
            (parent.to_path_buf(), false)
        } else {
            continue;
        };
        unique
            .entry(target)
            .and_modify(|current| *current |= recursive)
            .or_insert(recursive);
    }
    unique.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_are_watched_through_their_parent_directory() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("settings.toml");
        std::fs::write(&file, "theme = 'one'").unwrap();
        let targets = watch_targets([file]);
        assert_eq!(targets, vec![(temp.path().to_path_buf(), false)]);
    }

    #[test]
    fn missing_paths_use_only_the_immediate_existing_parent() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("theme.yaml");
        let targets = watch_targets([missing]);
        assert_eq!(targets, vec![(temp.path().to_path_buf(), false)]);
    }

    #[test]
    fn missing_nested_paths_do_not_fall_back_to_a_broad_ancestor() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("not-yet/themes/theme.yaml");
        assert!(watch_targets([missing]).is_empty());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn atomic_file_replacement_emits_one_debounced_invalidation() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("settings.toml");
        std::fs::write(&file, "theme = 'one'").unwrap();
        let mut watcher = WarpThemeWatcher::start([file.clone()]).expect("watcher starts");

        let replacement = temp.path().join("settings.toml.new");
        std::fs::write(&replacement, "theme = 'two'").unwrap();
        std::fs::rename(&replacement, &file).unwrap();

        tokio::time::timeout(Duration::from_secs(3), watcher.changed())
            .await
            .expect("filesystem invalidation timed out")
            .expect("watch channel remained open");
    }
}
