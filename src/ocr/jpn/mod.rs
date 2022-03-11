use std::collections::BTreeMap;

use super::{OCRWord, OCR};

use anyhow::Result;

pub struct JpnOCR {
    ocr: OCR,
    threshold: f32,
    discriminator: fn(&String) -> bool, // TODO filter kanji,hiragana,katakana
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
            discriminator: |_| true,
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
            .values()
            .flat_map(|line| self.from_line(line))
            .collect()
    }

    // digest OCR'd Japanese characters belonging to the same OCR 'line' into tentative words
    pub fn from_line(self: &Self, line: &Vec<&OCRWord>) -> Vec<JpnWord> {
        line.into_iter()
            .filter(|w| w.conf > self.threshold && (self.discriminator)(&w.text))
            // TODO sort then split on holes then map sequences to JpnWords
            .map(move |w| JpnWord {
                text: w.text.to_owned(),
                x: w.x,
                y: w.y,
                w: w.w,
                h: w.h,
            })
            .collect()
    }
}
