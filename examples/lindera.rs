use lindera::tokenizer::Tokenizer;

fn main() {
    let tokenizer = Tokenizer::new().unwrap();
    let text = "仕方がない。";
    let tokens = tokenizer
        //.tokenize("日本に行けば、仕事がさがせます。お金がかせげる。")
        //.tokenize("佐藤さんは背が高くて、髪が黒い人です。強くなりたい。")
        //.tokenize("きれいな月がもっときれいになった。")
        //.tokenize("ゆっくり上手に黒くなかったでしょう。")
        //.tokenize("別に。何でもない。")
        .tokenize(text)
        .unwrap();

    println!("{}", text);
    for t in &tokens {
        println!("{}: {:?}", t.text, t.detail);
    }
    println!("{}", text);

    // `Token.detail` structure (empirical)
    // [type, subtype, detail1, detail2, verb group, verb form, unconjugated, alt pronunciation1?, alt pronunciation2?]

    // 名詞: noun
    //   固有名詞: proper noun
    //   一般: universal
    //   接尾: suffix
    //   形容動詞語幹: quasi adjective (na)
    //   代名詞: pronoun
    // 助詞: particle
    //   格助詞: case marking particle
    //   接続助詞: conjunction particle
    //   係助詞: binding particle
    //   並立助詞: parallel marker
    // 動詞: verb
    //   自立: independant
    //   非自立: not independant
    // 助動詞: auxiliary verb
    // 形容詞: adjective
    // 副詞: adverb
    // 記号: sign
    //   句点: period
    //   読点: comma
    // 一段: ichidan
    // 五段: godan
    // 基本形: basic form
    // 仮定形: hypothetical/conditional form
    // 連用テ接続: adj continuous form (く)
}
