//! Parse human page ranges like `1-3,5,8-10` into 0-based page indices.

use std::collections::BTreeSet;

/// Parse a page-range string into sorted unique 0-based page indices.
///
/// Input is 1-based. Supports `1`, `1-3`, and comma-separated combinations.
pub fn parse_page_ranges(input: &str, page_count: usize) -> Result<Vec<usize>, String> {
    let groups = parse_page_range_groups(input, page_count)?;
    let mut set = BTreeSet::new();
    for group in groups {
        set.extend(group);
    }
    Ok(set.into_iter().collect())
}

/// Parse into one group per comma-separated segment (preserves split file boundaries).
///
/// Example: `"1-2,4"` with 4 pages → `[[0, 1], [3]]`.
pub fn parse_page_range_groups(input: &str, page_count: usize) -> Result<Vec<Vec<usize>>, String> {
    if page_count == 0 {
        return Err("Document has no pages".into());
    }
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Enter a page range (e.g. 1-3,5)".into());
    }

    let mut groups = Vec::new();
    for part in trimmed.split(',') {
        let part = part.trim();
        if part.is_empty() {
            return Err("Empty range segment".into());
        }
        let pages = parse_one_segment(part, page_count)?;
        if pages.is_empty() {
            return Err(format!("Range '{part}' selected no pages"));
        }
        groups.push(pages);
    }
    Ok(groups)
}

fn parse_one_segment(part: &str, page_count: usize) -> Result<Vec<usize>, String> {
    if let Some((a, b)) = part.split_once('-') {
        let start = parse_one_based(a.trim(), page_count)?;
        let end = parse_one_based(b.trim(), page_count)?;
        if start > end {
            return Err(format!("Inverted range '{part}'"));
        }
        Ok((start..=end).collect())
    } else {
        let page = parse_one_based(part, page_count)?;
        Ok(vec![page])
    }
}

fn parse_one_based(s: &str, page_count: usize) -> Result<usize, String> {
    if s.is_empty() {
        return Err("Missing page number".into());
    }
    let n: usize = s
        .parse()
        .map_err(|_| format!("Invalid page number '{s}'"))?;
    if n == 0 {
        return Err("Page numbers are 1-based (start at 1)".into());
    }
    if n > page_count {
        return Err(format!("Page {n} is out of range (1–{page_count})"));
    }
    Ok(n - 1)
}

/// Format 0-based pages as a filename suffix like `p1-3` or `p5`.
pub fn pages_filename_suffix(pages: &[usize]) -> String {
    if pages.is_empty() {
        return "p".into();
    }
    let first = pages[0] + 1;
    let last = pages[pages.len() - 1] + 1;
    if first == last {
        format!("p{first}")
    } else {
        format!("p{first}-{last}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_page() {
        assert_eq!(parse_page_ranges("1", 5).unwrap(), vec![0]);
        assert_eq!(parse_page_ranges("5", 5).unwrap(), vec![4]);
    }

    #[test]
    fn range() {
        assert_eq!(parse_page_ranges("1-3", 5).unwrap(), vec![0, 1, 2]);
    }

    #[test]
    fn mixed() {
        assert_eq!(
            parse_page_ranges("1-3,5,8-10", 10).unwrap(),
            vec![0, 1, 2, 4, 7, 8, 9]
        );
    }

    #[test]
    fn dedupes_and_sorts() {
        assert_eq!(parse_page_ranges("3,1,2,1", 5).unwrap(), vec![0, 1, 2]);
    }

    #[test]
    fn groups_preserve_segments() {
        assert_eq!(
            parse_page_range_groups("1-2,4", 4).unwrap(),
            vec![vec![0, 1], vec![3]]
        );
    }

    #[test]
    fn rejects_empty() {
        assert!(parse_page_ranges("", 5).is_err());
        assert!(parse_page_ranges("  ", 5).is_err());
    }

    #[test]
    fn rejects_inverted() {
        assert!(parse_page_ranges("3-1", 5).is_err());
    }

    #[test]
    fn rejects_out_of_range() {
        assert!(parse_page_ranges("6", 5).is_err());
        assert!(parse_page_ranges("0", 5).is_err());
    }

    #[test]
    fn suffix() {
        assert_eq!(pages_filename_suffix(&[0, 1, 2]), "p1-3");
        assert_eq!(pages_filename_suffix(&[4]), "p5");
    }
}
