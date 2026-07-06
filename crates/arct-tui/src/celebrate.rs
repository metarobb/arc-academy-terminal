//! Celebration banners for the output panel
//!
//! Box-drawing + sparkle banners that make lesson completions and unlocks
//! feel like a moment, not a log line.

/// Inner width of the banner (between the ║ borders)
const BANNER_WIDTH: usize = 52;

/// Center `text` within `width` display columns (char-count approximation,
/// good enough for the ASCII + sparkle glyphs we feed it)
fn center(text: &str, width: usize) -> String {
    let len = text.chars().count();
    if len >= width {
        return text.chars().take(width).collect();
    }
    let left = (width - len) / 2;
    let right = width - len - left;
    format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
}

/// Build a full-width celebration banner.
///
/// `headline` is the shout ("LESSON COMPLETE!"), `title` the thing being
/// celebrated, and `points` the XP/points earned (omitted when zero).
pub fn banner(headline: &str, title: &str, points: u32) -> String {
    let mut out = String::new();
    let horizontal = "═".repeat(BANNER_WIDTH);

    out.push_str(&format!("╔{}╗\n", horizontal));
    out.push_str(&format!("║{}║\n", center("", BANNER_WIDTH)));
    out.push_str(&format!(
        "║{}║\n",
        center(&format!("✦ ✧ {} ✧ ✦", headline), BANNER_WIDTH)
    ));
    out.push_str(&format!("║{}║\n", center(title, BANNER_WIDTH)));
    if points > 0 {
        out.push_str(&format!(
            "║{}║\n",
            center(&format!("+{} XP earned", points), BANNER_WIDTH)
        ));
    }
    out.push_str(&format!("║{}║\n", center("", BANNER_WIDTH)));
    out.push_str(&format!("╚{}╝\n", horizontal));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_banner_contains_headline_title_and_points() {
        let b = banner("LESSON COMPLETE!", "Navigation Basics", 50);
        assert!(b.contains("LESSON COMPLETE!"));
        assert!(b.contains("Navigation Basics"));
        assert!(b.contains("+50 XP earned"));
        assert!(b.contains("✦"));
    }

    #[test]
    fn test_banner_omits_points_line_when_zero() {
        let b = banner("UNLOCKED!", "Theme Explorer", 0);
        assert!(!b.contains("XP earned"));
    }

    #[test]
    fn test_banner_lines_have_consistent_width() {
        let b = banner("LESSON COMPLETE!", "Navigation Basics", 50);
        let widths: Vec<usize> = b.lines().map(|l| l.chars().count()).collect();
        assert!(!widths.is_empty());
        assert!(
            widths.iter().all(|w| *w == widths[0]),
            "banner box is ragged: {:?}",
            widths
        );
    }

    #[test]
    fn test_banner_truncates_very_long_titles() {
        let long = "x".repeat(200);
        let b = banner("LESSON COMPLETE!", &long, 10);
        let widths: Vec<usize> = b.lines().map(|l| l.chars().count()).collect();
        assert!(widths.iter().all(|w| *w == widths[0]));
    }
}
