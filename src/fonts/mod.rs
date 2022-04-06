use std::{
    collections::{BTreeMap, HashSet},
    ffi::CStr,
    path::PathBuf,
};

use fontconfig::{Fontconfig, Pattern};

pub fn path_to_font(fc: &Fontconfig, family: &str, style: Option<&str>) -> Option<PathBuf> {
    fc.find(family, style).map(|f| f.path)
}

pub fn japanese_font_families_and_styles(fc: &Fontconfig) -> BTreeMap<String, HashSet<String>> {
    fontconfig::list_fonts(&Pattern::new(&fc), None)
        .iter()
        .filter(|p| p.lang_set().unwrap().any(|s| s == "ja"))
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

pub fn japanese_font_families_and_styles_flat(fc: &Fontconfig) -> Vec<(String, String)> {
    japanese_font_families_and_styles(fc)
        .into_iter()
        .flat_map(|e| {
            e.1.iter()
                .map(|style| (e.0.clone(), style.clone()))
                .collect::<Vec<_>>()
        })
        .collect()
}
