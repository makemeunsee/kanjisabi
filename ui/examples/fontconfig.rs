use fontconfig::Fontconfig;
use kanjisabi::fonts::japanese_font_families_and_styles;

fn main() {
    let fc = Fontconfig::new().unwrap();
    for fam_and_styles in japanese_font_families_and_styles(&fc) {
        println!("{:?}", fam_and_styles);
    }
}
