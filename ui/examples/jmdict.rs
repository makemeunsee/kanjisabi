pub fn main() {
    let entry = jmdict::entries()
        .find(|e| e.kanji_elements().any(|k| k.text == "山"))
        .unwrap();
    let glosses: Vec<&str> = entry
        .senses()
        .flat_map(|s| s.glosses())
        .map(|g| g.text)
        .collect();
    println!("{:?}", glosses);
}
