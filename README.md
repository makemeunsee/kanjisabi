# 漢字錆 - kanjisabi

## Description

`rust` alternative to [KanjiTomo](https://www.kanjitomo.net/), i.e. on-screen Japanese OCR + translation hints.

Heavily WIP for now

Powered by:

- OCR: [Tesseract](https://github.com/tesseract-ocr/tesseract)
- Morphological analysis: [Lindera](https://github.com/lindera-morphology/lindera)
- Translation: [JMDict](http://edrdg.org/jmdict/j_jmdict.html) (soon)
- Presentation: libX11 via [x11rb](https://crates.io/crates/x11rb)
- Drawing: SDL via [sdl2](https://crates.io/crates/sdl2)
- [Many other awesome Rust crates](Cargo.toml)

## Requirements

- Running on the system:
  - A compositor, e.g. `picom`, to handle transparency across windows; should only matter to people running tiling windows managers such as `xmonad` or `i3`
  - A `X11` server, until someone passionate wants to port the UI logic to Wayland/Windows/whatever
- Libraries installed on the system:
  - `sdl2` and `sdl2_ttf`
  - `leptonica` and `tesseract`
  - Tesseract language libs: `tesseract-data-jpn`; `tesseract-data-eng` for the Tesseract example
  - `fontconfig`
- Japanese fonts, `Source Han Sans JP` and `Source Han Code JP` are personal recommendations

## Usage

- Hold `ctrl` + `alt` to start capturing an area on screen by moving the moving cursor
- Release `ctrl` + `alt` to trigger OCR, morphological analysis and translation hints
- Press `lshift` or `rshift` while the overlay is displayed to increase or decrease the hints font size
- Press again `ctrl` + `alt` without moving the mouse to discard the overlay
- `ctrl` + `alt` + `escape` to exit the program 

## Acknowledgments

[kanjitomo-ocr](https://github.com/sakarika/kanjitomo-ocr) for the original inspiration.
