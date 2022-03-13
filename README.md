# 漢字錆 - kanjisabi

## Description

`rust` alternative to [KanjiTomo](https://www.kanjitomo.net/), i.e. on-screen Japanese OCR + translation hints.

Heavily WIP for now

Powered by:
- [Tesseract](https://github.com/tesseract-ocr/tesseract)
- [JMDict](http://edrdg.org/jmdict/j_jmdict.html) (soon)
- [SDL](https://www.libsdl.org/)
- [a bunch of awesome Rust libraries and wrappers](Cargo.toml)

## Requirements

TODO

* Tesseract + jpn libs
* Aozora Mincho font, until a way to specify the render font is implemented
* fontconfig
* ??

## Usage

* `ctrl` + `alt` + `Esc`: quit
* `ctrl` + `alt` + `T`: toggle OCR and hints
* `ctrl` + `alt` + `up`|`right`|`down`|`left`: adjust OCR capture area

## Acknowledgments

[kanjitomo-ocr](https://github.com/sakarika/kanjitomo-ocr) for the original inspiration.
