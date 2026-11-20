//! Bidirectional (BiDi) text and cursive-joining support for the terminal grid.
//!
//! Terminal cells are stored in *logical* order: cell `n` holds the `n`th character the
//! application wrote. That ordering is wrong for display as soon as a row contains
//! characters that take part in the Unicode Bidirectional Algorithm (Arabic, Hebrew,
//! Syriac, Thaana, NKo, Adlam, ...) or in Arabic-style cursive joining, because:
//!
//! * RTL text has to be displayed right-to-left, so the visual position of a cell is not
//!   its column, and
//! * Arabic letters change shape depending on their neighbours, so a cell cannot be shaped
//!   in isolation — the whole joining run has to be shaped together.
//!
//! The renderer therefore shapes such rows as a single line (the "ligature" render path
//! already lays out a whole row through cosmic-text, which implements the UBA and uses
//! HarfBuzz-compatible shaping). This module supplies the analysis that decides *which*
//! rows need that treatment and the logical-cell -> visual-column mapping that the cursor,
//! backgrounds and decorations need so they stay aligned with the shaped text.

use unicode_bidi::{BidiInfo, Level};

/// Returns whether `c` can make a row require BiDi-aware rendering.
///
/// The ranges cover the RTL scripts supported by Unicode as well as the invisible controls
/// (ZWJ/ZWNJ, LRM/RLM, ALM and the explicit embedding/isolate controls) that change how
/// neighbouring cells are ordered or shaped.
pub fn is_bidi_or_joining_char(c: char) -> bool {
    matches!(c,
        '\u{0590}'..='\u{05FF}'   // Hebrew
        | '\u{0600}'..='\u{06FF}' // Arabic
        | '\u{0700}'..='\u{074F}' // Syriac
        | '\u{0750}'..='\u{077F}' // Arabic Supplement
        | '\u{0780}'..='\u{07BF}' // Thaana
        | '\u{07C0}'..='\u{07FF}' // NKo
        | '\u{0800}'..='\u{083F}' // Samaritan
        | '\u{0840}'..='\u{085F}' // Mandaic
        | '\u{0860}'..='\u{08FF}' // Syriac Supplement, Arabic Extended-A/B
        | '\u{FB1D}'..='\u{FDFF}' // Arabic Presentation Forms-A
        | '\u{FE70}'..='\u{FEFF}' // Arabic Presentation Forms-B
        | '\u{10800}'..='\u{10FFF}' // Cypriot..Arabic Extended-C (incl. Yezidi, Old Sogdian)
        | '\u{1E800}'..='\u{1EFFF}' // Mende Kikakui, Adlam, Arabic Mathematical Alphabetic
        | '\u{200C}'                  // ZWNJ
        | '\u{200D}'                  // ZWJ
        | '\u{200E}' | '\u{200F}'     // LRM / RLM
        | '\u{202A}'..='\u{202E}'     // LRE/RLE/PDF/LRO/RLO
        | '\u{2066}'..='\u{2069}'     // LRI/RLI/FSI/PDI
    )
}

/// Whether any of `cells` requires BiDi-aware rendering.
pub fn row_needs_bidi(cells: impl Iterator<Item = char>) -> bool {
    cells.into_iter().any(is_bidi_or_joining_char)
}

/// Visual ordering information for one grid row.
#[derive(Debug, Clone)]
pub struct BidiRowLayout {
    /// `cell_to_visual[cell]` is the visual column that `cell` is displayed at.
    cell_to_visual: Vec<usize>,
    /// Whether the paragraph direction resolved to RTL.
    paragraph_is_rtl: bool,
}

impl BidiRowLayout {
    /// Visual column for a logical cell index.
    pub fn visual_column(&self, cell: usize) -> usize {
        self.cell_to_visual
            .get(cell)
            .copied()
            .unwrap_or(cell)
    }

    pub fn paragraph_is_rtl(&self) -> bool {
        self.paragraph_is_rtl
    }

    #[cfg(test)]
    pub fn visual_order(&self) -> Vec<usize> {
        let mut order = vec![usize::MAX; self.cell_to_visual.len()];
        for (cell, visual) in self.cell_to_visual.iter().enumerate() {
            order[*visual] = cell;
        }
        order
    }
}

/// Resolves the visual order of `levels` per rule L2 of the Unicode Bidirectional Algorithm.
///
/// The returned vector maps a logical index to its visual index.
fn reorder_map(levels: &[Level]) -> Vec<usize> {
    let mut visual: Vec<usize> = (0..levels.len()).collect();

    let lowest_odd = levels
        .iter()
        .filter(|level| level.is_rtl())
        .map(|level| level.number())
        .min();
    let highest = levels.iter().map(|level| level.number()).max();

    if let (Some(lowest_odd), Some(highest)) = (lowest_odd, highest) {
        for level in (lowest_odd..=highest).rev() {
            let mut i = 0;
            while i < visual.len() {
                if levels[visual[i]].number() >= level {
                    let start = i;
                    while i < visual.len() && levels[visual[i]].number() >= level {
                        i += 1;
                    }
                    visual[start..i].reverse();
                } else {
                    i += 1;
                }
            }
        }
    }

    visual
}

/// Analyses one grid row.
///
/// `cells` must contain exactly one character per grid cell, in logical order (a cell that
/// holds a zero-width cluster contributes its base character). Wide-character spacer cells
/// must contain a placeholder character.
///
/// Returns `None` when the row does not need BiDi-aware rendering.
pub fn analyze_row(cells: &[char]) -> Option<BidiRowLayout> {
    if cells.is_empty() || !cells.iter().any(|c| is_bidi_or_joining_char(*c)) {
        return None;
    }

    let text: String = cells.iter().collect();
    let info = BidiInfo::new(&text, None);
    let Some(para) = info.paragraphs.first() else {
        return None;
    };

    // `levels` is indexed by byte offset; map it to characters.
    let levels: Vec<Level> = text
        .char_indices()
        .map(|(byte_index, _)| info.levels[byte_index])
        .collect();

    let cell_to_visual = reorder_map(&levels);

    Some(BidiRowLayout {
        cell_to_visual,
        paragraph_is_rtl: para.level.is_rtl(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    #[test]
    fn plain_latin_rows_are_not_analysed() {
        assert!(analyze_row(&cells("cargo build --release")).is_none());
    }

    #[test]
    fn pure_arabic_row_is_reversed() {
        // Three logical cells, paragraph is RTL, so cell 0 goes to the rightmost column.
        let layout = analyze_row(&cells("مرح")).expect("row needs bidi");
        assert_eq!(layout.visual_order(), vec![2, 1, 0]);
        assert!(layout.paragraph_is_rtl());
    }

    #[test]
    fn mixed_row_keeps_latin_word_intact_and_last() {
        // "abc مرح": "abc" is LTR and must stay left-to-right, Arabic is displayed to its right
        // and reversed, so the rightmost column holds the first Arabic letter.
        let layout = analyze_row(&cells("abc مرح")).expect("row needs bidi");
        assert_eq!(layout.visual_order(), vec![0, 1, 2, 3, 6, 5, 4]);
    }

    #[test]
    fn single_arabic_cell_maps_to_itself() {
        let layout = analyze_row(&cells("ب")).expect("row needs bidi");
        assert_eq!(layout.visual_column(0), 0);
    }

    #[test]
    fn zwj_marks_row_as_needing_bidi() {
        assert!(row_needs_bidi("👩‍👩‍👧".chars()));
        assert!(!row_needs_bidi("plain ascii".chars()));
    }
}
