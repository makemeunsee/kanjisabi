pub mod jpn;

use tesseract::Tesseract;

use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct OCRWord {
    pub text: String,
    pub line_id: (u32, u32, u32, u32),
    pub word_num: u32,
    pub conf: f32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub struct OCR {
    pub lang: String,
}

impl OCR {
    pub fn recognize_words(
        self: &Self,
        frame_data: &[u8],
        width: i32,
        height: i32,
        bytes_per_pixel: i32,
        bytes_per_line: i32,
    ) -> Result<Vec<OCRWord>> {
        let tsv = Tesseract::new(None, Some(self.lang.as_str()))?
            .set_frame(frame_data, width, height, bytes_per_pixel, bytes_per_line)?
            .recognize()?
            .get_tsv_text(0)?;

        Ok(tsv
            .lines()
            .filter(|l| l.starts_with("5"))
            .filter_map(|l| self.maybe_word(l).ok())
            .collect())
    }

    fn maybe_word(self: &Self, s: &str) -> Result<OCRWord> {
        let tokens: Vec<String> = s.split_terminator("\t").map(String::from).collect();
        if tokens.len() < 12 {
            return Err(anyhow!(
                "unable to parse tsv result from Tesseract: {:?}",
                s
            ));
        }

        let conf = tokens[10].parse::<f32>()?;

        let page = tokens[1].parse::<u32>()?;
        let block = tokens[2].parse::<u32>()?;
        let paragraph = tokens[3].parse::<u32>()?;
        let line = tokens[4].parse::<u32>()?;
        let word_num = tokens[5].parse::<u32>()?;

        let line_id = (page, block, paragraph, line);

        let x = tokens[6].parse::<i32>()?;
        let y = tokens[7].parse::<i32>()?;
        let w = tokens[8].parse::<i32>()?;
        let h = tokens[9].parse::<i32>()?;
        let text = tokens[11].clone();

        Ok(OCRWord {
            text,
            line_id,
            word_num,
            conf,
            x,
            y,
            w,
            h,
        })
    }
}
