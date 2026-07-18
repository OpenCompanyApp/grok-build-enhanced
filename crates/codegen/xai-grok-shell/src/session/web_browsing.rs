//! Pure intent classification for bounded native web-tool recovery.
//!
//! This module examines only the current genuine user request. Callers must
//! never pass synthetic reminders, tool output, or external web content.

/// Whether the current genuine user request requires native browsing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum BrowseRequirement {
    /// The user explicitly prohibited browsing. This always wins.
    Forbidden,
    /// The request explicitly asks for browsing/sources or is clearly
    /// time-sensitive enough that an offline answer would be misleading.
    Required,
    /// Browsing may help, but the request does not require it.
    #[default]
    Optional,
}

/// Classify a genuine user request without inspecting conversation history or
/// rendered external content.
///
/// The classifier is deliberately conservative. It strips code fences, inline
/// code, quoted lines, and quoted spans before matching so instructions quoted
/// for discussion do not become user intent.
pub(crate) fn classify_browse_requirement(request: &str) -> BrowseRequirement {
    let visible = strip_quoted_or_code_content(request).to_ascii_lowercase();
    let normalized = visible.split_whitespace().collect::<Vec<_>>().join(" ");

    const FORBIDDEN: &[&str] = &[
        "do not browse",
        "don't browse",
        "dont browse",
        "without browsing",
        "no browsing",
        "do not search the web",
        "don't search the web",
        "dont search the web",
        "no web search",
        "do not use the internet",
        "don't use the internet",
        "offline only",
        "not asking you to browse",
    ];
    if FORBIDDEN.iter().any(|phrase| normalized.contains(phrase)) {
        return BrowseRequirement::Forbidden;
    }

    const EXPLICIT_REQUIRED: &[&str] = &[
        "browse the web",
        "browse online",
        "search the web",
        "search online",
        "search the internet",
        "web search",
        "look this up",
        "look it up",
        "look up online",
        "check online",
        "fetch this url",
        "fetch the url",
        "open this url",
        "open the url",
        "open this link",
        "find sources",
        "cite sources",
        "with sources",
        "provide sources",
        "provide citations",
        "verify online",
        "verify on the web",
    ];
    if EXPLICIT_REQUIRED
        .iter()
        .any(|phrase| normalized.contains(phrase))
    {
        return BrowseRequirement::Required;
    }

    // These terms have a strong temporal meaning on their own. Avoid broad
    // words such as "current" and "recent", which commonly describe local
    // code or conversation state rather than changing external facts.
    let time_sensitive = normalized
        .split(|c: char| !c.is_alphanumeric() && c != '-')
        .any(|word| matches!(word, "latest" | "today" | "tonight" | "yesterday"));
    let real_time = ["right now", "as of today", "real-time", "real time"]
        .iter()
        .any(|phrase| normalized.contains(phrase));
    if time_sensitive || real_time {
        BrowseRequirement::Required
    } else {
        BrowseRequirement::Optional
    }
}

fn strip_quoted_or_code_content(input: &str) -> String {
    let mut without_blocks = String::with_capacity(input.len());
    let mut in_fence = false;
    for line in input.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence || trimmed.starts_with('>') {
            continue;
        }
        without_blocks.push_str(line);
        without_blocks.push('\n');
    }

    let chars = without_blocks.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(without_blocks.len());
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (index, ch) in chars.iter().copied().enumerate() {
        if escaped {
            escaped = false;
            if quote.is_none() {
                output.push(ch);
            }
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        match quote {
            Some(active) if ch == active => quote = None,
            Some(_) => {}
            None if matches!(ch, '`' | '"' | '\'') => {
                let apostrophe_in_word = ch == '\''
                    && index > 0
                    && chars[index - 1].is_alphanumeric()
                    && chars
                        .get(index + 1)
                        .is_some_and(|next| next.is_alphanumeric());
                let has_closing_quote = chars[index + 1..].contains(&ch);
                if apostrophe_in_word || !has_closing_quote {
                    output.push(ch);
                } else {
                    quote = Some(ch);
                }
            }
            None => output.push(ch),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_browsing_and_time_sensitive_requests_are_required() {
        for request in [
            "Browse the web and compare these products.",
            "Please cite sources for this claim.",
            "Open this URL and summarize it.",
            "What is the latest stable Rust release?",
            "Who won the match today?",
            "Check the service status right now.",
        ] {
            assert_eq!(
                classify_browse_requirement(request),
                BrowseRequirement::Required,
                "request: {request}"
            );
        }
    }

    #[test]
    fn explicit_browsing_prohibition_wins() {
        for request in [
            "Do not browse; answer from memory.",
            "Don't browse; answer from memory.",
            "Give me an offline-only explanation and don't search the web.",
            "I am not asking you to browse the web.",
            "Without browsing, explain the algorithm.",
        ] {
            assert_eq!(
                classify_browse_requirement(request),
                BrowseRequirement::Forbidden,
                "request: {request}"
            );
        }
    }

    #[test]
    fn quoted_or_code_browsing_language_is_not_intent() {
        for request in [
            "Explain what `browse the web` means.",
            "What does the phrase \"search the web\" imply?",
            "Explain the phrase 'search the web' without doing it.",
            "> browse the web\nExplain the quoted instruction.",
            "```text\nsearch the internet\n```\nReview this sample.",
        ] {
            assert_eq!(
                classify_browse_requirement(request),
                BrowseRequirement::Optional,
                "request: {request}"
            );
        }
    }

    #[test]
    fn ordinary_local_requests_remain_optional() {
        for request in [
            "Explain this function.",
            "Review the current code in src/main.rs.",
            "Summarize the recent conversation.",
        ] {
            assert_eq!(
                classify_browse_requirement(request),
                BrowseRequirement::Optional,
                "request: {request}"
            );
        }
    }
}
