//! Chapter selection parsing for the `--chapters` flag.
//!
//! Supports single chapters and inclusive ranges in Rust-slice style
//! (chapters are 1-indexed):
//! - `5`    → chapter 5
//! - `2..5` → chapters 2 through 5 (inclusive)
//! - `2..`  → chapter 2 to the end
//! - `..5`  → chapters 1 through 5
//! - `1,3..5,7..` → any mix, comma-separated (whitespace trimmed)
//!
//! Malformed tokens are silently ignored. Reversed ranges (`5..2`) are
//! normalized to `2..5`. Out-of-bounds chapters simply never match.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChapterSpec {
    Single(u32),
    Range(Option<u32>, Option<u32>),
}

impl ChapterSpec {
    /// Returns `true` when the 1-indexed `chapter_num` is selected by this spec.
    pub fn matches(&self, chapter_num: u32) -> bool {
        match *self {
            ChapterSpec::Single(n) => chapter_num == n,
            ChapterSpec::Range(start, end) => {
                let start = start.unwrap_or(1);
                let end = end.unwrap_or(u32::MAX);
                chapter_num >= start && chapter_num <= end
            }
        }
    }
}

/// Parse a `--chapters` value into a list of specs.
///
/// Tokens are comma-separated and whitespace-trimmed. Malformed tokens
/// (non-numeric, stray hyphens, empty tokens) are silently dropped.
pub fn parse_chapters(s: &str) -> Vec<ChapterSpec> {
    s.split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .filter_map(parse_token)
        .collect()
}

fn parse_token(token: &str) -> Option<ChapterSpec> {
    if let Some((start, end)) = token.split_once("..") {
        // Only one ".." is allowed per token; "1..3..5" is malformed.
        if end.contains("..") {
            return None;
        }
        // A bare ".." (both ends empty) is malformed.
        if start.is_empty() && end.is_empty() {
            return None;
        }
        let start = if start.is_empty() {
            None
        } else {
            Some(start.parse::<u32>().ok()?)
        };
        let end = if end.is_empty() {
            None
        } else {
            Some(end.parse::<u32>().ok()?)
        };
        Some(normalize_range(start, end))
    } else {
        token.parse::<u32>().ok().map(ChapterSpec::Single)
    }
}

/// Swap reversed ranges (`5..2` → `2..5`) so matches stay well-defined.
fn normalize_range(start: Option<u32>, end: Option<u32>) -> ChapterSpec {
    match (start, end) {
        (Some(s), Some(e)) if s > e => ChapterSpec::Range(Some(e), Some(s)),
        (s, e) => ChapterSpec::Range(s, e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn any_matches(parsed: &[ChapterSpec], chapter_num: u32) -> bool {
        parsed.iter().any(|spec| spec.matches(chapter_num))
    }

    #[test]
    fn parses_single_chapters() {
        assert_eq!(
            parse_chapters("1,2,3"),
            vec![
                ChapterSpec::Single(1),
                ChapterSpec::Single(2),
                ChapterSpec::Single(3)
            ]
        );
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(
            parse_chapters(" 1 , 3..5 , 7 "),
            vec![
                ChapterSpec::Single(1),
                ChapterSpec::Range(Some(3), Some(5)),
                ChapterSpec::Single(7)
            ]
        );
    }

    #[test]
    fn matches_inclusive_range() {
        let parsed = parse_chapters("2..5");
        assert!(!any_matches(&parsed, 1));
        assert!(any_matches(&parsed, 2));
        assert!(any_matches(&parsed, 5));
        assert!(!any_matches(&parsed, 6));
    }

    #[test]
    fn matches_open_ended_start() {
        let parsed = parse_chapters("3..");
        assert!(!any_matches(&parsed, 2));
        assert!(any_matches(&parsed, 3));
        assert!(any_matches(&parsed, 99));
    }

    #[test]
    fn matches_open_ended_end() {
        let parsed = parse_chapters("..4");
        assert!(any_matches(&parsed, 1));
        assert!(any_matches(&parsed, 4));
        assert!(!any_matches(&parsed, 5));
    }

    #[test]
    fn swaps_reversed_ranges() {
        let parsed = parse_chapters("5..2");
        assert!(!any_matches(&parsed, 1));
        assert!(any_matches(&parsed, 2));
        assert!(any_matches(&parsed, 5));
        assert!(!any_matches(&parsed, 6));
    }

    #[test]
    fn drops_malformed_tokens() {
        let parsed = parse_chapters("1,abc,..,1-3,,3");
        assert_eq!(
            parsed,
            vec![ChapterSpec::Single(1), ChapterSpec::Single(3)]
        );
    }

    #[test]
    fn out_of_bounds_chapters_never_match() {
        // Chapter numbers are 1-indexed in the download loop, so 0 and
        // ranges beyond the book's chapters simply never match.
        let parsed = parse_chapters("0,10..99");
        assert!(!any_matches(&parsed, 1));
        assert!(any_matches(&parsed, 10));
        assert!(any_matches(&parsed, 99));
        assert!(!any_matches(&parsed, 100));
    }

    #[test]
    fn mixes_single_ranges_and_open_ends() {
        let parsed = parse_chapters("1,3..5,7..,..4");
        assert!(any_matches(&parsed, 1));
        assert!(any_matches(&parsed, 4));
        assert!(any_matches(&parsed, 7));
        assert!(any_matches(&parsed, 8));
    }

    #[test]
    fn empty_input_is_empty_filter() {
        assert!(parse_chapters("").is_empty());
        assert!(parse_chapters(" , , ").is_empty());
    }
}
