use tesseract::Tesseract;

use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct OCRWord {
    pub word: String,
    pub conf: f32,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub fn recognize_words(
    lang: &String,
    frame_data: &[u8],
    width: i32,
    height: i32,
    bytes_per_pixel: i32,
    bytes_per_line: i32,
) -> Result<Vec<OCRWord>> {
    let tsv = Tesseract::new(None, Some(lang.as_str()))?
        .set_frame(frame_data, width, height, bytes_per_pixel, bytes_per_line)?
        .recognize()?
        .get_tsv_text(0)?;

    Ok(tsv
        .lines()
        // TODO look into <5 categories
        .filter(|l| l.starts_with("5"))
        .filter_map(|l| maybe_word(l).ok())
        .collect())
}

fn maybe_word(s: &str) -> Result<OCRWord> {
    let tokens: Vec<String> = s.split_terminator("\t").map(String::from).collect();
    if tokens.len() < 12 {
        return Err(anyhow!(
            "unable to parse tsv result from Tesseract: {:?}",
            s
        ));
    }
    let x = tokens[6].parse::<u32>()?;
    let y = tokens[7].parse::<u32>()?;
    let w = tokens[8].parse::<u32>()?;
    let h = tokens[9].parse::<u32>()?;
    let conf = tokens[10].parse::<f32>()?;
    let word = tokens[11].clone();
    Ok(OCRWord {
        word,
        conf,
        x,
        y,
        w,
        h,
    })
}
