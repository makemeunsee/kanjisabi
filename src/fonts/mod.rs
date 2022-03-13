use std::{
    collections::{BTreeMap, HashSet},
    ffi::CStr,
};

use fontconfig::{Fontconfig, Pattern};

pub fn japanese_font_families_and_styles(fc: &Fontconfig) -> BTreeMap<String, HashSet<String>> {
    fontconfig::list_fonts(&Pattern::new(&fc), None)
        .iter()
        .filter(|p| p.get_lang_set().unwrap().contains(&"ja"))
        .fold(
            BTreeMap::new(),
            |mut acc: BTreeMap<String, HashSet<String>>, p| {
                let family = p
                    .get_string(CStr::from_bytes_with_nul(b"family\0").unwrap())
                    .unwrap()
                    .to_owned();
                let set = acc.entry(family).or_default();
                match p.get_string(CStr::from_bytes_with_nul(b"style\0").unwrap()) {
                    Some(style) => set.insert(style.to_owned()),
                    None => false,
                };
                acc
            },
        )
}
