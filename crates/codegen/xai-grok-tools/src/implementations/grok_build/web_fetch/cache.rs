//! In-memory cache for self-contained text fetches with TTL expiry and eviction.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::types::output::WebFetchOutput;

#[derive(Clone)]
struct CachedPage {
    output: WebFetchOutput,
    inserted: Instant,
}

pub(crate) struct FetchCache {
    entries: HashMap<String, CachedPage>,
    ttl: Duration,
    max_entries: usize,
}

#[derive(Debug, Clone)]
pub(crate) enum FetchCacheLookup {
    Hit(WebFetchOutput),
    Miss,
    Stale,
}

/// Simple cache that holds N completed fetch requests on a TTL.
impl FetchCache {
    pub(crate) fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
            max_entries,
        }
    }

    pub(crate) fn lookup(&mut self, url: &str) -> FetchCacheLookup {
        let Some(entry) = self.entries.get(url) else {
            return FetchCacheLookup::Miss;
        };
        if entry.inserted.elapsed() < self.ttl {
            return FetchCacheLookup::Hit(entry.output.clone());
        }
        self.entries.remove(url);
        FetchCacheLookup::Stale
    }

    #[cfg(test)]
    pub(crate) fn get(&mut self, url: &str) -> Option<WebFetchOutput> {
        match self.lookup(url) {
            FetchCacheLookup::Hit(output) => Some(output),
            FetchCacheLookup::Miss | FetchCacheLookup::Stale => None,
        }
    }

    /// Cache only inline text; path-bearing outputs must be materialized per call.
    pub(crate) fn insert_text(&mut self, url: String, output: WebFetchOutput, was_truncated: bool) {
        if was_truncated {
            return;
        }
        if self.entries.len() >= self.max_entries {
            // Evict oldest entry.
            let oldest_key = self
                .entries
                .iter()
                .max_by_key(|(_, v)| v.inserted.elapsed())
                .map(|(k, _)| k.clone());
            if let Some(key) = oldest_key {
                self.entries.remove(&key);
            }
        }
        self.entries.insert(
            url,
            CachedPage {
                output,
                inserted: Instant::now(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::output::WebFetchContent;

    fn output(content: &str) -> WebFetchOutput {
        WebFetchOutput::Content(WebFetchContent {
            url: "https://example.com/".to_string(),
            content: content.to_string(),
            content_type: "markdown".to_string(),
            status_code: 200,
            bytes: content.len(),
            source_artifact: None,
            inline_fallback: None,
            output_location: None,
        })
    }

    #[test]
    fn truncated_artifact_output_is_never_cached() {
        let mut cache = FetchCache::new(Duration::from_secs(60), 10);
        let url = "https://example.com/";
        cache.insert_text(url.to_string(), output("/sessions/a/web_fetch/1.md"), true);
        assert!(cache.get(url).is_none());

        cache.insert_text(url.to_string(), output("fully inline"), false);
        assert!(cache.get(url).is_some());
    }

    #[test]
    fn lookup_distinguishes_hit_miss_and_stale_revalidation() {
        let url = "https://example.com/";
        let mut live = FetchCache::new(Duration::from_secs(60), 10);
        assert!(matches!(live.lookup(url), FetchCacheLookup::Miss));
        live.insert_text(url.to_owned(), output("cached"), false);
        assert!(matches!(live.lookup(url), FetchCacheLookup::Hit(_)));

        let mut expired = FetchCache::new(Duration::ZERO, 10);
        expired.insert_text(url.to_owned(), output("expired"), false);
        assert!(matches!(expired.lookup(url), FetchCacheLookup::Stale));
        assert!(matches!(expired.lookup(url), FetchCacheLookup::Miss));
    }
}
