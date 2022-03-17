# 漢字錆 - kanjisabi

## Description

`rust` alternative to [KanjiTomo](https://www.kanjitomo.net/), i.e. on-screen Japanese OCR + translation hints.

Heavily WIP for now

Powered by:
- [Tesseract](https://github.com/tesseract-ocr/tesseract)
- [JMDict](http://edrdg.org/jmdict/j_jmdict.html) (soon)
- [SDL](https://www.libsdl.org/)
- [fontconfig](https://www.freedesktop.org/wiki/Software/fontconfig/)
- [a bunch of awesome Rust libraries and wrappers](Cargo.toml)

## Requirements

* `sdl2` and `sdl2_ttf`
* `leptonica` and `tesseract`, via [its Rust bindings](https://docs.rs/tesseract/latest/tesseract/index.html)
+ Tesseract language libs: `tesseract-data-jpn`, `tesseract-data-eng`; `tesseract-data-jpn_vert` can't hurt
* `fontconfig`
* Japanese fonts installed, e.g. `Aozora Mincho`, `Source Han Sans`, ...

## Usage

* `ctrl` + `alt` + `Esc`: quit
* `ctrl` + `alt` + `T`: toggle OCR and hints
* `ctrl` + `alt` + `up`|`right`|`down`|`left`: adjust OCR capture area
* `ctrl` + `alt` + `-`|`=`: adjust overlay font scaling
* `ctrl` + `alt` + `N`|`P`: cycle through available Japanese fonts

## Acknowledgments

[kanjitomo-ocr](https://github.com/sakarika/kanjitomo-ocr) for the original inspiration.
