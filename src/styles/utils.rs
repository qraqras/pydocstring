//! Shared utilities for docstring style parsers.

/// Find the byte offset of the first entry-separating colon in `text`.
///
/// Skips colons that appear inside balanced brackets (`()`, `[]`, `{}`, `<>`)
/// so that type annotations such as `Dict[str, int]` never trigger a false split.
pub(crate) fn find_entry_colon(text: &str) -> Option<usize> {
    let mut depth: u32 = 0;
    for (i, b) in text.bytes().enumerate() {
        match b {
            b'(' | b'[' | b'{' | b'<' => depth += 1,
            b')' | b']' | b'}' | b'>' => depth = depth.saturating_sub(1),
            b':' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Split `text` by top-level commas (respecting `()`, `[]`, `{}`, and `<>` depth).
///
/// Returns a `Vec` of `(byte_offset, segment)` pairs where
/// `byte_offset` is the start position of each segment within `text`.
pub(crate) fn split_comma_parts(text: &str) -> Vec<(usize, &str)> {
    let mut parts = Vec::new();
    let mut depth: u32 = 0;
    let mut start = 0;

    for (i, b) in text.bytes().enumerate() {
        match b {
            b'(' | b'[' | b'{' | b'<' => depth += 1,
            b')' | b']' | b'}' | b'>' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                parts.push((start, &text[start..i]));
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push((start, &text[start..]));
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_entry_colon() {
        // Basic colon
        assert_eq!(find_entry_colon("name: desc"), Some(4));
        assert_eq!(find_entry_colon("name:desc"), Some(4));
        assert_eq!(find_entry_colon("name:"), Some(4));
        // No colon
        assert_eq!(find_entry_colon("name"), None);
        // Colon inside brackets is skipped
        assert_eq!(find_entry_colon("Dict[str, int]: desc"), Some(14));
        assert_eq!(find_entry_colon("Tuple(a, b): desc"), Some(11));
        // Nested brackets
        assert_eq!(find_entry_colon("Dict[str, List[int]]: desc"), Some(20));
        // Only colon inside brackets — no match
        assert_eq!(find_entry_colon("Dict[k: v]"), None);
    }

    #[test]
    fn test_split_comma_parts() {
        let parts: Vec<_> = split_comma_parts("int, optional")
            .iter()
            .map(|(_, s)| s.trim())
            .collect();
        assert_eq!(parts, vec!["int", "optional"]);

        // Brackets respected
        let parts: Vec<_> = split_comma_parts("Dict[str, int], optional")
            .iter()
            .map(|(_, s)| s.trim())
            .collect();
        assert_eq!(parts, vec!["Dict[str, int]", "optional"]);

        // Offsets
        let parts = split_comma_parts("int, optional");
        assert_eq!(parts[0].0, 0);
        assert_eq!(parts[1].0, 4);
    }
}
