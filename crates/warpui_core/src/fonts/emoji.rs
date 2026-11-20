//! Helpers for color emoji rendering.
//!
//! Terminals render text on a fixed grid, so emoji characters reach the
//! rasterizer as individual characters (or as short strings when the terminal
//! passed a grapheme cluster such as a ZWJ sequence to `glyph_for_string`).
//! Two things are needed for them to show up in full color:
//!
//! 1. The glyph must be looked up in a *color* emoji font before any monochrome
//!    fallback (`Symbola`, `Noto Emoji`, `DejaVu Sans`, ...) claims it. Font
//!    configuration does not guarantee this: on Fedora, `fontconfig` happily
//!    sorts a monochrome symbol font ahead of the color emoji font.
//! 2. The rasterizer must be asked for RGBA output rather than an alpha mask,
//!    because a mask drops the palette entirely. That part lives in the
//!    platform rasterizer, not here.
//!
//! This module therefore only holds the character classification and the pure
//! fallback-ordering rule, so both can be unit tested without a font stack.

use super::FontId;

/// Returns whether `c` belongs to a Unicode block or code point that terminal
/// emoji fonts are expected to supply.
///
/// The ranges follow Unicode's `Emoji` property closely enough for font
/// selection: they intentionally include the zero width joiner and the emoji
/// variation selector, since a terminal may ask for per-character fallbacks for
/// those code points when it is *not* shaping a whole cluster.
///
/// ASCII keycap bases (`0`-`9`, `#`, `*`) are deliberately excluded: they are
/// emoji only when followed by an enclosing keycap, and claiming them here would
/// send every digit in the terminal to the emoji font.
pub fn is_emoji_char(c: char) -> bool {
    let cp = c as u32;
    matches!(cp,
        0x200D                 // zero width joiner
        | 0x20E3               // combining enclosing keycap
        | 0xFE0F               // emoji variation selector-16
        | 0xFE0E               // text variation selector-15
        | 0x238C | 0x2394 | 0x2395 // misc technical emoji
        // Noto/Twemoji place U+2600..=U+27BF ("misc symbols" + "dingbats")
        // here: ☀ ✅ ❤ ✨ ✂ etc.
        | 0x2600..=0x27BF
        // Supplemental arrows and misc symbols used as emoji.
        | 0x2B00..=0x2BFF
        // Emoji modifiers (skin tones) and regional indicators from the
        // supplemental symbols block.
        | 0x1F000..=0x1FAFF
        // Tags used by flag sequences (Scotland, Wales, England).
        | 0xE0020..=0xE007F
    )
}

/// Orders the fallback fonts for `ch` so that the configured color emoji font is
/// tried first for emoji characters.
///
/// `fallbacks` is the platform's fontconfig-derived chain for the primary font.
/// The emoji font is deliberately *not* moved to the front for ordinary text,
/// because it only covers emoji and would simply be skipped again.
pub fn prioritize_emoji_fallback(
    ch: char,
    emoji_font: Option<FontId>,
    fallbacks: Vec<FontId>,
) -> Vec<FontId> {
    let Some(emoji_font) = emoji_font else {
        return fallbacks;
    };
    if !is_emoji_char(ch) {
        return fallbacks;
    }
    let mut ordered = Vec::with_capacity(fallbacks.len() + 1);
    ordered.push(emoji_font);
    ordered.extend(fallbacks.into_iter().filter(|font| *font != emoji_font));
    ordered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_common_emoji_code_points() {
        for c in ['😀', '🚀', '✅', '❌', '❤', '🇪', '🇬', '👨', '🏽', '\u{200D}', '\u{FE0F}', '\u{20E3}'] {
            assert!(is_emoji_char(c), "expected {c:?} (U+{:04X}) to be emoji", c as u32);
        }
    }

    #[test]
    fn does_not_treat_regular_text_as_emoji() {
        for c in ['a', 'Z', 'م', 'ي', '1', '.', '~', '\u{0640}', '─', '│'] {
            assert!(
                !is_emoji_char(c),
                "expected {c:?} (U+{:04X}) not to be emoji",
                c as u32
            );
        }
    }

    #[test]
    fn emoji_font_leads_fallback_chain_for_emoji() {
        let emoji = FontId(7);
        let latin = FontId(1);
        let symbol = FontId(2);
        let ordered = prioritize_emoji_fallback('😀', Some(emoji), vec![latin, symbol]);
        assert_eq!(ordered, vec![emoji, latin, symbol]);
    }

    #[test]
    fn fallback_chain_untouched_for_non_emoji_text() {
        let emoji = FontId(7);
        let latin = FontId(1);
        assert_eq!(
            prioritize_emoji_fallback('م', Some(emoji), vec![latin]),
            vec![latin]
        );
    }

    #[test]
    fn emoji_font_is_not_duplicated() {
        let emoji = FontId(7);
        let ordered = prioritize_emoji_fallback('🚀', Some(emoji), vec![emoji, FontId(1)]);
        assert_eq!(ordered, vec![emoji, FontId(1)]);
    }

    #[test]
    fn missing_emoji_font_is_a_no_op() {
        let latin = FontId(1);
        assert_eq!(
            prioritize_emoji_fallback('😀', None, vec![latin]),
            vec![latin]
        );
    }
}
