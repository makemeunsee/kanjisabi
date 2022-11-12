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
  - A `X11` server, until someone passionate wants to port the UI logic to Wayland/Windows/whatever
  - A compositor, e.g. `picom`, to handle transparency; this should only be relevant to people running tiling windows managers such as `xmonad` or `i3`, full-fledged desktop environments like KDE or GNOME have their own compositor.
  - A [Lindera server](https://github.com/lindera-morphology/lindera-server), using the dictionary matching the configuration of the Kanjisabi server (so far, using [features](morph_server/Cargo.toml)). The Lindera server can actually run remotely, and its socket address (IP+port) can be set in the [configuration](#configuration)
- Libraries installed on the system:
  - `sdl2` and `sdl2_ttf`
  - `leptonica` and `tesseract`
  - Tesseract language libs: `tesseract-data-jpn`; `tesseract-data-eng` for the Tesseract example
  - `fontconfig`
- Japanese fonts, `Source Han Sans JP` and `Source Han Code JP` are personal recommendations

## Usage

- Hold `lctrl` + `lalt` to start capturing an area on screen by moving the moving cursor
- Release `lctrl` + `lalt` to trigger OCR, morphological analysis and translation hints
- Press `lshift` while the overlay is displayed to cycle through hints
- Press `lctrl` while the overlay is displayed to cycle through morphemes to detail within a hint
- Press `rshift` or `rctrl` while the overlay is displayed to increase or decrease the hints font size
- Press again `lctrl` + `lalt` without moving the mouse to discard the overlay
- `lctrl` + `lalt` + `escape` to exit the program

## Configuration

`kanjisabi` looks for an optional TOML configuration file at `$XDG_CONFIG_HOME/kanjisabi.toml`.

Here's an annotated configuration showing the default values:

```toml
[lindera]
# the address of the Lindera server, to which morphological analysis is delegated
server_address = "0.0.0.0:3333"

[font]
# what font to use when displaying hints; the first Japanese font found will be used if empty
family = ""
# valid only if `font_family` is defined and valid; the default font style of the actually used font will be used if empty or not valid
style = ""

[colors]
# ARGB, the color used to visualize the screen capture area
capture = 0x20002000
# ARGB, the color used to highlight the parts of the captured area that the OCR managed to read
highlight = 0x20200000
# ARGB, the font color used when displaying hints in the overlay
hint = 0xFF32FF00
# ARGB, the background color used when displaying hints in the overlay
hint_bg = 0xC0000024

[preproc]
# float, the contrast increase applied to the captured screen area prior to performing OCR
contrast = 100

# global hotkeys for controlling the app; keys are device_query's Keycodes: <https://docs.rs/device_query/latest/device_query/keymap/enum.Keycode.html>
[keys]
# main action: screen capture followed by OCR, dismissal of the overlay when displayed
trigger = ["LControl", "LAlt"]
# exit the application
quit = ["LControl", "LAlt", "Escape"]
# increase the font size
font_up = ["RControl"]
# decrease the font size
font_down = ["RShift"]
# cycle through the visual hints to display
next_hint = ["LControl"]
# cycle through the translation morphemes (within a hint) to detail
next_morpheme = ["LShift"]
```

## Future features

Dependency parsing, similar to / based on [UniDic2UD](https://github.com/KoichiYasuoka/UniDic2UD)

## Acknowledgments and licenses

[kanjitomo-ocr](https://github.com/sakarika/kanjitomo-ocr) for the original inspiration.

The database files (JMdict) compiled into the [`jmdict` crate](https://github.com/majewsky/rust-jmdict) this project depends on are licensed from the Electronic Dictionary Research and Development Group under Creative Commons licenses. Please refer to the [EDRDG's license statement](http://www.edrdg.org/edrdg/licence.html) for details.