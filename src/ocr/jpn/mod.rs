use std::collections::BTreeMap;

use super::{OCRWord, OCR};

use anyhow::Result;
use lindera::tokenizer::Tokenizer;

pub struct JpnOCR {
    ocr: OCR,
    threshold: f32,
    discriminator: fn(&str) -> bool,
    tokenizer: Tokenizer,
}

pub struct JpnText {
    pub words: Vec<String>,
    pub text: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

fn is_kanji(c: char) -> bool {
    (c >= '\u{4e00}' && c <= '\u{9ffc}')          // https://www.unicode.org/charts/PDF/U4E00.pdf
        || (c >= '\u{f900}' && c <= '\u{faff}')   // https://www.unicode.org/charts/PDF/UF900.pdf
        || (c >= '\u{3400}' && c <= '\u{4dbf}')   // https://www.unicode.org/charts/PDF/U3400.pdf
        || (c >= '\u{20000}' && c <= '\u{2a6dd}') // https://www.unicode.org/charts/PDF/U3400.pdf
        || (c >= '\u{2a700}' && c <= '\u{2b734}') // https://www.unicode.org/charts/PDF/U2A700.pdf
        || (c >= '\u{2b740}' && c <= '\u{2b81d}') // https://www.unicode.org/charts/PDF/U2B740.pdf
        || (c >= '\u{2b820}' && c <= '\u{2cea1}') // https://www.unicode.org/charts/PDF/U2B820.pdf
        || (c >= '\u{2ceb0}' && c <= '\u{2ebe0}') // https://www.unicode.org/charts/PDF/U2CEB0.pdf
        || (c >= '\u{2f800}' && c <= '\u{2fa1d}') // https://www.unicode.org/charts/PDF/U2F800.pdf
        || (c >= '\u{30000}' && c <= '\u{3134a}') // https://www.unicode.org/charts/PDF/U30000.pdf
        || c == '\u{3005}' // 々 - https://www.unicode.org/charts/PDF/U3000.pdf
}

fn is_hiragana(c: char) -> bool {
    c >= '\u{3041}' && c <= '\u{3096}'          // https://www.unicode.org/charts/PDF/U3040.pdf
        || c == '\u{1b001}'                     // https://www.unicode.org/charts/PDF/U1B000.pdf
        || c == '\u{1b11f}'                     // https://www.unicode.org/charts/PDF/U1B100.pdf
        || c >= '\u{1b150}' && c <= '\u{1b152}' // https://www.unicode.org/charts/PDF/U1B130.pdf
}

fn is_katakana(c: char) -> bool {
    c >= '\u{30a1}' && c <= '\u{30fa}' || c == '\u{30fc}' // https://www.unicode.org/charts/PDF/U30A0.pdf
        || c >= '\u{31f0}' && c <= '\u{31ff}'   // https://www.unicode.org/charts/PDF/U31F0.pdf
        || c >= '\u{ff66}' && c<= '\u{ff9d}'    // https://www.unicode.org/charts/PDF/UFF00.pdf
        || c == '\u{1b000}'                     // https://www.unicode.org/charts/PDF/U1B000.pdf
        || c >= '\u{1b164}' && c <= '\u{1b167}' // https://www.unicode.org/charts/PDF/U1B130.pdf
}

impl JpnOCR {
    pub fn new() -> JpnOCR {
        JpnOCR {
            ocr: OCR {
                lang: String::from("jpn"),
            },
            threshold: 80.,
            discriminator: |s| {
                s.chars()
                    .all(|c| is_kanji(c) || is_katakana(c) || is_hiragana(c))
            },
            tokenizer: Tokenizer::new().unwrap(),
        }
    }

    pub fn recognize(
        self: &Self,
        frame_data: &[u8],
        width: i32,
        height: i32,
        bytes_per_pixel: i32,
        bytes_per_line: i32,
    ) -> Result<Vec<JpnText>> {
        let ocr_words =
            self.ocr
                .recognize_words(frame_data, width, height, bytes_per_pixel, bytes_per_line)?;
        Ok(self.from_ocr_words(&ocr_words))
    }

    pub fn from_ocr_words(self: &Self, words: &Vec<OCRWord>) -> Vec<JpnText> {
        words
            .into_iter()
            .fold(
                BTreeMap::new(),
                |mut acc: BTreeMap<(u32, u32, u32, u32), Vec<&OCRWord>>, word| {
                    acc.entry(word.line_id).or_default().push(word);
                    acc
                },
            )
            .values_mut()
            .flat_map(|line| self.from_line(line))
            .collect()
    }

    /// digest OCR'd Japanese characters belonging to the same OCR 'line' into tentative words
    pub fn from_line(self: &Self, line: &Vec<&OCRWord>) -> Vec<JpnText> {
        line.split(|w| w.conf <= self.threshold || !(self.discriminator)(&w.text))
            .filter_map(|seq| {
                if seq.len() == 0 {
                    None
                } else {
                    Some(self.from_word_seq(seq))
                }
            })
            .collect()
    }

    fn from_word_seq(self: &Self, seq: &[&OCRWord]) -> JpnText {
        let mut x = std::i32::MAX;
        let mut y = std::i32::MAX;
        let mut w = 0;
        let mut h = 0;
        let mut text = "".to_string();

        // TODO average out ys and hs, as tesseract jpn bounds are often off; ask tesseract? https://github.com/tesseract-ocr/tesseract#support
        for word in seq {
            x = std::cmp::min(x, word.x as i32);
            y = std::cmp::min(y, word.y as i32);
            w = std::cmp::max(w, word.w as i32 + word.x as i32 - x);
            h = std::cmp::max(h, word.h as i32 + word.y as i32 - y);
            text.push_str(&word.text);
        }

        let tokens = self.tokenizer.tokenize(&text).unwrap();

        // TODO include `token.detail`?
        // to filter out e.g.

        for t in &tokens {
            println!("{}: {:?}", t.text, t.detail);
        }

        JpnText {
            words: tokens.iter().map(|t| t.text.to_owned()).collect(),
            text,
            x: x as u32,
            y: y as u32,
            w: w as u32,
            h: h as u32,
        }
    }
}
