use lindera::tokenizer::Token;

#[derive(PartialEq, Eq, Debug)]
pub enum LexicalCategory {
    ProperNoun,
    SuruVerb,
    Noun,
    NounSuffix,
    Keiyoudoushi,
    Pronoun,
    AdjectiveI,
    Adverb,
    AdverbificationParticle,
    AdjectificationParticle,
    ConjunctionParticle,
    Verb,
    AuxiliaryVerb,
    AuxiliaryNa,
    Particle,
    Sign,
}

// TODO split into category & sub-category e.g. (noun, {proper noun | suru verb | na-adj | possible adverb | ...} )
pub fn categorize(token: &Token) -> Option<LexicalCategory> {
    let owned_details: Vec<&str> = token.detail.iter().map(String::as_str).collect();
    println!("{:?}", owned_details);
    match owned_details.as_slice() {
        // Verb stems
        ["動詞", _, _, _, _, _, _, _, _] => Some(LexicalCategory::Verb),

        // Noun stems
        // i.e. noun adjective, na-adj
        ["名詞", "形容動詞語幹", _, _, _, _, _, _, _] => {
            Some(LexicalCategory::Keiyoudoushi)
        }
        ["名詞", "一般", _, _, _, _, _, _, _] => Some(LexicalCategory::Noun),
        // number
        ["名詞", "数", _, _, _, _, _, _, _] => Some(LexicalCategory::Noun),
        // possible adverb, e.g. それぞれ
        // TODO adverb and/or noun?
        ["名詞", "副詞可能", _, _, _, _, _, _, _] => Some(LexicalCategory::Noun),
        ["名詞", "接尾", _, _, _, _, _, _, _] => Some(LexicalCategory::NounSuffix),
        ["名詞", "固有名詞", _, _, _, _, _, _, _] => Some(LexicalCategory::ProperNoun),
        ["名詞", "サ変接続", _, _, _, _, _, _, _] => Some(LexicalCategory::SuruVerb),
        ["名詞", _, _, _, _, _, _, _, _] => Some(LexicalCategory::Noun),

        // Adjective stems
        ["形容詞", _, _, _, "形容詞・イ段", "基本形", _, _, _] => {
            Some(LexicalCategory::AdjectiveI)
        }
        // adverbial form (く) of i-adj
        // TODO validate with more cases
        ["形容詞", _, _, _, _, "連用テ接続", _, _, _] => Some(LexicalCategory::Adverb),
        ["形容詞", _, _, _, _, _, _, _, _] => Some(LexicalCategory::AdjectiveI),

        // Copulas, auxiliary verbs
        // e.g. な of na-adj
        ["助動詞", _, _, _, _, "体言接続", _, _, _] => Some(LexicalCategory::AuxiliaryNa),
        // e.g. だ、です、まし(た)
        ["助動詞", _, _, _, _, _, _, _, _] => Some(LexicalCategory::AuxiliaryVerb),

        // Particles
        // e.g. に of きれいに
        ["助詞", "副詞化", _, _, _, _, _, _, _] => {
            Some(LexicalCategory::AdverbificationParticle)
        }
        // e.g. −て
        ["助詞", "接続助詞", _, _, _, _, _, _, _] => {
            Some(LexicalCategory::ConjunctionParticle)
        }
        // e.g. −の−
        ["助詞", "連体化", _, _, _, _, _, _, _] => {
            Some(LexicalCategory::AdjectificationParticle)
        }
        // case marking particle
        ["助詞", "格助詞", _, _, _, _, _, _, _] => Some(LexicalCategory::Particle),
        // binding particle
        ["助詞", "係助詞", _, _, _, _, _, _, _] => Some(LexicalCategory::Particle),
        // adverbial particle e.g. まで
        ["助詞", "副助詞", _, _, _, _, _, _, _] => Some(LexicalCategory::Particle),
        ["助詞", _, _, _, _, _, _, _, _] => Some(LexicalCategory::Particle),

        // Adverbs
        ["副詞", _, _, _, _, _, _, _, _] => Some(LexicalCategory::Adverb),

        _ => None,
    }
}

#[test]
fn きれい_is_keiyoudoushi() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("きれい").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Keiyoudoushi));
}

#[test]
fn きれいな_is_keiyoudoushi() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("きれいな").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Keiyoudoushi));
    assert_eq!(categorize(&tokens[1]), Some(LexicalCategory::AuxiliaryNa));
}

#[test]
fn きれいに_is_keiyoudoushi() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("きれいに").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Keiyoudoushi));
    assert_eq!(
        categorize(&tokens[1]),
        Some(LexicalCategory::AdverbificationParticle)
    );
}

#[test]
fn 重大_is_keiyoudoushi() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("重大").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Keiyoudoushi));
}

#[test]
fn 重大な_is_keiyoudoushi() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("重大な").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Keiyoudoushi));
    assert_eq!(categorize(&tokens[1]), Some(LexicalCategory::AuxiliaryNa));
}

#[test]
fn 新幹線_is_noun() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("新幹線").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Noun));
}

#[test]
fn 難しい_is_i_adjective() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("難しい").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::AdjectiveI));
}

#[test]
fn 難しく_is_adverb() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("難しく").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Adverb));
}

// #[test]
// fn 大抵_is_adverb() {
//     use lindera::tokenizer::Tokenizer;
//     let tokens = Tokenizer::new().unwrap().tokenize("大抵").unwrap();
//     assert_eq!(tokens.len(), 1);
//     assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Adverb));
// }

#[test]
fn あまり_is_adverb() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("あまり").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Adverb));
}

#[test]
fn 高くない() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("高くない").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Adverb));
    assert_eq!(categorize(&tokens[1]), Some(LexicalCategory::AuxiliaryVerb));
}

#[test]
fn 高くなく_is_adverb() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("高くなく").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Adverb));
    assert_eq!(categorize(&tokens[1]), Some(LexicalCategory::Adverb));
}

#[test]
fn たかくない() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("たかくない").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Adverb));
    assert_eq!(categorize(&tokens[1]), Some(LexicalCategory::AuxiliaryVerb));
}

#[test]
fn 降る_is_verb() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("降る").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Verb));
}

#[test]
fn 降ります_is_verb() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("降ります").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Verb));
    assert_eq!(categorize(&tokens[1]), Some(LexicalCategory::AuxiliaryVerb));
}

#[test]
fn 降って_is_verb() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("降って").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Verb));
    assert_eq!(
        categorize(&tokens[1]),
        Some(LexicalCategory::ConjunctionParticle)
    );
}

#[test]
fn 降った_is_verb() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("降った").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Verb));
    assert_eq!(categorize(&tokens[1]), Some(LexicalCategory::AuxiliaryVerb));
}

#[test]
fn 降りました_is_verb() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("降りました").unwrap();
    assert_eq!(tokens.len(), 3);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Verb));
    assert_eq!(categorize(&tokens[1]), Some(LexicalCategory::AuxiliaryVerb));
    assert_eq!(categorize(&tokens[2]), Some(LexicalCategory::AuxiliaryVerb));
}

#[test]
fn 降らない_is_verb() {
    use lindera::tokenizer::Tokenizer;
    let tokens = Tokenizer::new().unwrap().tokenize("降らない").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(categorize(&tokens[0]), Some(LexicalCategory::Verb));
    assert_eq!(categorize(&tokens[1]), Some(LexicalCategory::AuxiliaryVerb));
}
