//! Grapheme-cluster rules for emoji that arrive as several code points.
//!
//! A terminal receives emoji sequences as separate code points. Most of them
//! (variation selectors, zero width joiners, combining enclosing keycaps, tag
//! characters) are zero width, so the grid already appends them to the previous
//! cell. Two kinds are not:
//!
//! * emoji modifiers, i.e. skin tones `U+1F3FB..=U+1F3FF`, which report a width
//!   of two on their own, and
//! * regional indicators `U+1F1E6..=U+1F1FF`, which form a flag when they come
//!   in pairs.
//!
//! Written into separate cells, those code points can never be shaped into the
//! single color glyph the font provides, so the ANSI handler asks this module
//! whether an incoming code point continues the previous cell's grapheme.

const EMOJI_MODIFIER_RANGE: std::ops::RangeInclusive<u32> = 0x1F3FB..=0x1F3FF;
const REGIONAL_INDICATOR_RANGE: std::ops::RangeInclusive<u32> = 0x1F1E6..=0x1F1FF;
const ZERO_WIDTH_JOINER: char = '\u{200D}';

fn is_emoji_modifier(c: char) -> bool {
    EMOJI_MODIFIER_RANGE.contains(&(c as u32))
}

fn is_regional_indicator(c: char) -> bool {
    REGIONAL_INDICATOR_RANGE.contains(&(c as u32))
}

/// Whether `c` can carry the emoji property on its own, i.e. whether a modifier
/// or a joiner may attach to it.
fn is_emoji_base(c: char) -> bool {
    let cp = c as u32;
    !is_emoji_modifier(c)
        && !is_regional_indicator(c)
        && (matches!(cp,
            0x2600..=0x27BF // misc symbols + dingbats
            | 0x2B00..=0x2BFF
            | 0x1F000..=0x1FAFF
            | 0x0023 | 0x002A | 0x0030..=0x0039 // keycap bases
        ) || c == ZERO_WIDTH_JOINER)
}

fn last_char(content: &str) -> Option<char> {
    content.chars().next_back()
}

/// Whether the incoming code point `c` continues the grapheme already stored in
/// `previous_cell`.
///
/// `previous_cell` is the full cell content, which may itself be a multi code
/// point cluster such as `👍🏽` or `🇪🇬`.
pub fn continues_emoji_cluster(previous_cell: &str, c: char) -> bool {
    let Some(last) = last_char(previous_cell) else {
        return false;
    };

    if is_emoji_modifier(c) {
        // 👍 + 🏽 -> 👍🏽. A modifier never stands alone when the previous cell
        // is an emoji base that does not already carry a modifier.
        return is_emoji_base(last) && !is_emoji_modifier(last);
    }

    if is_regional_indicator(c) {
        // A flag is exactly two regional indicators: 🇪 + 🇬 -> 🇪🇬.
        let existing_indicators = previous_cell
            .chars()
            .filter(|existing| is_regional_indicator(*existing))
            .count();
        return existing_indicators == 1;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skin_tone_joins_the_preceding_emoji() {
        assert!(continues_emoji_cluster("👍", '\u{1F3FD}'));
        assert!(continues_emoji_cluster("👨\u{200D}👩", '\u{1F3FE}'));
    }

    #[test]
    fn skin_tone_does_not_join_plain_text_or_a_second_modifier() {
        assert!(!continues_emoji_cluster("م", '\u{1F3FD}'));
        assert!(!continues_emoji_cluster("👍\u{1F3FD}", '\u{1F3FE}'));
    }

    #[test]
    fn second_regional_indicator_forms_a_flag() {
        assert!(continues_emoji_cluster("🇪", '🇬'));
        // A third indicator starts a new flag rather than extending the old one.
        assert!(!continues_emoji_cluster("🇪🇬", '🇸'));
    }

    #[test]
    fn ordinary_characters_are_not_cluster_continuations() {
        assert!(!continues_emoji_cluster("👍", 'a'));
        assert!(!continues_emoji_cluster("", '\u{1F3FD}'));
    }
}
