use std::collections::BTreeMap;

use super::{OCRWord, OCR};

use anyhow::Result;
use lindera::tokenizer::Tokenizer;
use log::warn;

pub struct JpnOCR {
    ocr: OCR,
    threshold: f32,
    discriminator: fn(&str) -> bool,
    tokenizer: Tokenizer,
}

#[derive(Debug)]
pub struct Morpheme {
    pub text: String,
    /// Same structure as `lindera::tokenizer::Token.detail` - present documentation is empirical
    /// if the original token is valid:
    /// [type, subtype, detail1, detail2, verb group, verb form, dict form, alt pronunciation1?, alt pronunciation2?]
    /// otherwise:
    /// ["UNK"]
    pub detail: Vec<String>,
    pub bbox: Option<(i32, i32, i32, i32)>,
}

#[derive(Debug)]
pub struct JpnText {
    pub morphemes: Vec<Morpheme>,
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
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
            // TODO try to support 'jpn_vert' too; initial tries gave very bad results
            ocr: OCR {
                lang: String::from("jpn"),
            },
            threshold: 80.,
            discriminator: |s| {
                // assumption: OCR does not group non-Japanese and Japanese characters (e.g. ２階), or it's ok not to care about them
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
        let chars_in_seq = seq
            .iter()
            .map(|t| t.text.chars().count() as u32)
            .sum::<u32>();

        let mut x = std::i32::MAX;
        let mut y = 0;
        let mut w = 0;
        let mut h = 0;
        let mut text = "".to_string();

        // for each character in all words of the sequence, assign it a bounding box if it's the first character of its word
        // later used for assigning bounding boxes to morphemes
        let mut bounding_boxes = vec![];

        // averaging out ys and hs, as Tesseract bboxes are not accurate
        // see https://github.com/tesseract-ocr/tesseract/labels/bounding%20box
        for word in seq {
            bounding_boxes.push(Some((word.x, word.y, word.w, word.h)));

            for _ in 1..word.text.chars().count() {
                bounding_boxes.push(None);
            }
            x = std::cmp::min(x, word.x);
            y += word.y;
            w = std::cmp::max(w, word.w + word.x - x);
            h += word.h;
            text.push_str(&word.text);
        }
        y = (y as f32 / chars_in_seq as f32) as i32;
        h = (h as f32 / chars_in_seq as f32) as i32;

        let tokens = self.tokenizer.tokenize(&text).unwrap();

        let chars_in_tokens = tokens
            .iter()
            .map(|t| t.text.chars().count() as u32)
            .sum::<u32>();

        if chars_in_seq != chars_in_tokens {
            warn!("Inconsistent morphological analysis results, discarding them");
            return JpnText {
                morphemes: vec![],
                text,
                x,
                y,
                w,
                h,
            };
        }

        let mut morphemes = vec![];

        let mut char_index = 0;
        for t in tokens {
            let len = t.text.chars().count();
            let mut x = std::i32::MAX;
            let mut y = std::i32::MAX;
            let mut w = 0;
            let mut h = 0;
            for i in 0..len {
                if let Some((bx, by, bw, bh)) = bounding_boxes[char_index + i] {
                    x = std::cmp::min(x, bx);
                    y = std::cmp::min(y, by);
                    w = std::cmp::max(w, bw + bx - x);
                    h = std::cmp::max(h, bh + by - y);
                }
            }

            let bbox = (x, y, w, h);
            let morpheme = Morpheme {
                text: t.text.to_owned(),
                detail: t.detail.clone(),
                bbox: Some(bbox),
            };
            char_index += len;
            print_jmdict_results(&morpheme.text);
            if morpheme.detail.len() >= 7 {
                print_jmdict_results(&morpheme.detail[6]);
            }
            morphemes.push(morpheme);
        }

        JpnText {
            morphemes,
            text,
            x,
            y,
            w,
            h,
        }
    }
}

pub fn print_jmdict_results(text: &str) {
    match jmdict::entries().find(|e| e.kanji_elements().any(|k| k.text == text)) {
        Some(entry) => {
            let glosses: Vec<&str> = entry
                .senses()
                .flat_map(|s| s.glosses())
                .map(|g| g.text)
                .collect();
            println!("{} -> {:?}", text, glosses);
        }
        None => (),
    }
}
