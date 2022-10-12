use std::collections::BTreeMap;

use super::{OCRWord, OCR};

use anyhow::Result;
use jmdict::Entry;
use log::info;
use morph::{JpnMorphAnalysisAPI, Morpheme};
use tokio::runtime::{Builder, Runtime};

pub struct JpnOCR {
    ocr: OCR,
    threshold: f32,
    discriminator: fn(&str) -> bool,
    morph_api: JpnMorphAnalysisAPI,
    rt: Runtime,
}

#[derive(Debug)]
pub struct VisualMorpheme {
    pub morpheme: Morpheme,
    pub bbox: Option<(i32, i32, i32, i32)>,
}

#[derive(Debug)]
pub struct JpnText {
    pub morphemes: Vec<VisualMorpheme>,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

fn is_kanji(c: char) -> bool {
    ('\u{4e00}'..='\u{9ffc}').contains(&c)         // https://www.unicode.org/charts/PDF/U4E00.pdf
        || ('\u{f900}'..='\u{faff}').contains(&c)  // https://www.unicode.org/charts/PDF/UF900.pdf
        || ('\u{3400}'..='\u{4dbf}').contains(&c)  // https://www.unicode.org/charts/PDF/U3400.pdf
        || ('\u{20000}'..='\u{2a6dd}').contains(&c)// https://www.unicode.org/charts/PDF/U3400.pdf
        || ('\u{2a700}'..='\u{2b734}').contains(&c)// https://www.unicode.org/charts/PDF/U2A700.pdf
        || ('\u{2b740}'..='\u{2b81d}').contains(&c)// https://www.unicode.org/charts/PDF/U2B740.pdf
        || ('\u{2b820}'..='\u{2cea1}').contains(&c)// https://www.unicode.org/charts/PDF/U2B820.pdf
        || ('\u{2ceb0}'..='\u{2ebe0}').contains(&c)// https://www.unicode.org/charts/PDF/U2CEB0.pdf
        || ('\u{2f800}'..='\u{2fa1d}').contains(&c)// https://www.unicode.org/charts/PDF/U2F800.pdf
        || ('\u{30000}'..='\u{3134a}').contains(&c)// https://www.unicode.org/charts/PDF/U30000.pdf
        || c == '\u{3005}' // 々 - https://www.unicode.org/charts/PDF/U3000.pdf
}

fn is_hiragana(c: char) -> bool {
    ('\u{3041}'..='\u{3096}').contains(&c)          // https://www.unicode.org/charts/PDF/U3040.pdf
        || c == '\u{1b001}'                              // https://www.unicode.org/charts/PDF/U1B000.pdf
        || c == '\u{1b11f}'                              // https://www.unicode.org/charts/PDF/U1B100.pdf
        || ('\u{1b150}'..='\u{1b152}').contains(&c) // https://www.unicode.org/charts/PDF/U1B130.pdf
}

fn is_katakana(c: char) -> bool {
    ('\u{30a1}'..='\u{30fa}').contains(&c)|| c == '\u{30fc}' // https://www.unicode.org/charts/PDF/U30A0.pdf
        || ('\u{31f0}'..='\u{31ff}').contains(&c)            // https://www.unicode.org/charts/PDF/U31F0.pdf
        || ('\u{ff66}'..='\u{ff9d}').contains(&c)            // https://www.unicode.org/charts/PDF/UFF00.pdf
        || c == '\u{1b000}'                                       // https://www.unicode.org/charts/PDF/U1B000.pdf
        || ('\u{1b164}'..='\u{1b167}').contains(&c) // https://www.unicode.org/charts/PDF/U1B130.pdf
}

impl JpnOCR {
    pub fn new(morph_api: JpnMorphAnalysisAPI) -> JpnOCR {
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
            morph_api,
            rt: Builder::new_multi_thread().enable_all().build().unwrap(),
        }
    }

    pub fn recognize(
        &mut self,
        frame_data: &[u8],
        width: i32,
        height: i32,
        bytes_per_pixel: i32,
        bytes_per_line: i32,
    ) -> Result<Vec<JpnText>> {
        let ocr_words =
            self.ocr
                .recognize_words(frame_data, width, height, bytes_per_pixel, bytes_per_line)?;
        Ok(self.ocr_words_to_text(&ocr_words))
    }

    fn ocr_words_to_text(&mut self, words: &[OCRWord]) -> Vec<JpnText> {
        words
            .iter()
            .fold(
                BTreeMap::new(),
                |mut acc: BTreeMap<(u32, u32, u32, u32), Vec<&OCRWord>>, word| {
                    acc.entry(word.line_id).or_default().push(word);
                    acc
                },
            )
            .values_mut()
            .flat_map(|line| self.line_to_text(line))
            .collect()
    }

    /// digest OCR'd Japanese characters belonging to the same OCR 'line' into tentative words
    fn line_to_text(&mut self, line: &[&OCRWord]) -> Vec<JpnText> {
        let threshold = self.threshold;
        let discriminator = self.discriminator;
        let is_valid_jpn = |w: &&OCRWord| w.conf <= threshold || !(discriminator)(&w.text);
        let to_jpn = |seq: &[&OCRWord]| {
            if seq.is_empty() {
                None
            } else {
                Some(self.word_seq_to_text(seq))
            }
        };
        line.split(is_valid_jpn).filter_map(to_jpn).collect()
    }

    fn word_seq_to_text(&mut self, seq: &[&OCRWord]) -> JpnText {
        let chars_in_seq = seq
            .iter()
            .map(|t| t.text.chars().count() as u32)
            .sum::<u32>();

        let mut x = std::i32::MAX;
        let mut y = 0;
        let mut w = 0;
        let mut h = 0;
        let mut text = "".to_owned();

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

        let morphemes = self
            .rt
            .block_on(self.morph_api.morphemes(&text))
            .unwrap_or_default();

        let chars_in_morphemes = morphemes
            .iter()
            .map(|m| m.text.chars().count() as u32)
            .sum::<u32>();

        if chars_in_seq != chars_in_morphemes {
            info!("Inconsistent morphological analysis results, discarding them");
            return JpnText {
                morphemes: vec![],
                x,
                y,
                w,
                h,
            };
        }

        let mut v_morphemes = vec![];

        let mut char_index = 0;
        for morpheme in morphemes {
            let len = morpheme.text.chars().count();
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
            let v_morpheme = VisualMorpheme {
                morpheme,
                bbox: Some(bbox),
            };
            char_index += len;

            // TODO restore
            // println!("{:?}", categorize(&token));

            // print_jmdict_results(&morpheme.text);
            // if let Some(dict_form) = morpheme.detail.get(6) {
            //     if dict_form != &morpheme.text {
            //         println!("morpheme dict form: {}", dict_form);
            //         print_jmdict_results(&dict_form);
            //     }
            // }

            v_morphemes.push(v_morpheme);
        }

        JpnText {
            morphemes: v_morphemes,
            x,
            y,
            w,
            h,
        }
    }
}

pub fn print_jmdict_results(text: &str) {
    println!("->");
    match jmdict::entries().find(|e| e.kanji_elements().any(|k| k.text == text)) {
        Some(entry) => print_entry(&entry),
        None => {
            if let Some(entry) =
                jmdict::entries().find(|e| e.reading_elements().any(|k| k.text == text))
            {
                print_entry(&entry)
            }
        }
    }

    fn print_entry(entry: &Entry) {
        let glosses: Vec<&str> = entry
            .senses()
            .flat_map(|s| s.glosses())
            .map(|g| g.text)
            .collect();
        let readings: Vec<&str> = entry.reading_elements().map(|re| re.text).collect();
        println!("{:?}\n{:?}", readings, glosses);
    }
}
