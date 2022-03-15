use std::collections::BTreeMap;

use super::{OCRWord, OCR};

use anyhow::Result;
// TODO probably an overkill to use a lib for this; filter Unicode range directly instead
use kanji::{is_hiragana, is_kanji, is_katakana};

pub struct JpnOCR {
    ocr: OCR,
    threshold: f32,
    discriminator: fn(&str) -> bool,
}

pub struct JpnWord {
    pub text: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
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
        }
    }

    pub fn recognize_words(
        self: &Self,
        frame_data: &[u8],
        width: i32,
        height: i32,
        bytes_per_pixel: i32,
        bytes_per_line: i32,
    ) -> Result<Vec<JpnWord>> {
        let ocr_words =
            self.ocr
                .recognize_words(frame_data, width, height, bytes_per_pixel, bytes_per_line)?;
        Ok(self.from_ocr_words(&ocr_words))
    }

    pub fn from_ocr_words(self: &Self, words: &Vec<OCRWord>) -> Vec<JpnWord> {
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

    // digest OCR'd Japanese characters belonging to the same OCR 'line' into tentative words
    pub fn from_line(self: &Self, line: &mut Vec<&OCRWord>) -> Vec<JpnWord> {
        line.sort_by(|a, b| a.word_num.cmp(&b.word_num));
        line.into_iter()
            .filter(|w| w.conf > self.threshold && (self.discriminator)(&w.text))
            .fold(vec![], |mut acc: Vec<Vec<&OCRWord>>, word| {
                let last_seq = acc.last_mut();
                let last_id = last_seq
                    .as_ref()
                    .and_then(|v| v.last())
                    .map_or(std::u32::MAX - 1, |w| w.word_num);
                if last_id + 1 == word.word_num {
                    last_seq.unwrap().push(word);
                } else {
                    acc.push(vec![word]);
                }
                acc
            })
            .into_iter()
            .map(|w| from_word_seq(&w))
            .collect()
    }
}

fn from_word_seq(seq: &Vec<&OCRWord>) -> JpnWord {
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
    JpnWord {
        text: text.to_owned(),
        x: x as u32,
        y: y as u32,
        w: w as u32,
        h: h as u32,
    }
}
