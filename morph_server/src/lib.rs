use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug)]
pub enum LexicalCategory {
    ProperNoun,
    SuruVerb,
    Prefix,
    Noun,
    NounSuffix,
    Keiyoudoushi,
    Pronoun,
    AdjectiveIDictForm,
    AdjectiveIConjForm,
    NegationDictForm,
    NegationConjForm,
    Adverb,
    AdverbificationParticle,
    AdjectivisationParticle,
    ContinuativeParticle,
    ContinuativeAuxiliary,
    Verb,
    AuxiliaryVerb,
    AttributiveParticle,
    Particle,
    CaseParticle,
    ConnectingParticle,
    Sign,
}

#[cfg(feature = "unidic")]
#[derive(Serialize, Deserialize, Debug)]
struct LinderaToken {
    detail: Vec<String>,
}

// TODO config/args + README
pub async fn tokenize(text: &str) -> Result<Vec<Vec<String>>> {
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3333/tokenize")
        .body(text.to_owned())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let tokens: Vec<LinderaToken> = serde_json::from_str(&response)?;
    Ok(tokens
        .iter()
        .map(|token| token.detail.clone())
        .collect::<Vec<Vec<String>>>())
}

#[cfg(feature = "ipadic")]
pub fn categorize(details: &Vec<String>) -> Option<LexicalCategory> {
    println!("{:?}", details);
    match details
        .iter()
        .map(String::as_str)
        .collect::<Vec<&str>>()
        .as_slice()
    {
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
        // continuous (て) / adverbial form
        ["助動詞", _, _, _, _, "連用テ接続", _, _, _] => Some(LexicalCategory::Adverb),
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
            Some(LexicalCategory::AdjectivisationParticle)
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

//TODO split into category and sub-category? +dict form morpheme
#[cfg(feature = "unidic")]
pub fn categorize(details: &Vec<String>) -> Option<LexicalCategory> {
    //TODO remove
    println!("{:?}", details);
    match details
        .iter()
        .map(String::as_str)
        .collect::<Vec<&str>>()
        .as_slice()
    {
        ["動詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => Some(LexicalCategory::Verb),
        ["名詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => Some(LexicalCategory::Noun),
        ["接頭辞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => {
            Some(LexicalCategory::Prefix)
        }
        ["形状詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => {
            Some(LexicalCategory::Keiyoudoushi)
        }
        ["助動詞", _, _, _, _, "連体形-一般", _, _, "な", _, _, _, _, _, _, _, _] => {
            Some(LexicalCategory::AttributiveParticle)
        }
        ["助動詞", _, _, _, _, _, _, _, "に", _, _, _, _, _, _, _, _] => {
            Some(LexicalCategory::AdverbificationParticle)
        }
        ["助動詞", _, _, _, _, "連用形-一般", _, _, _, _, _, _, _, _, _, _, _] => {
            Some(LexicalCategory::ContinuativeAuxiliary)
        }
        ["助動詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => {
            Some(LexicalCategory::AuxiliaryVerb)
        }
        ["助詞", "接続助詞", _, _, _, _, _, _, "て", _, _, _, _, _, _, _, _] => {
            Some(LexicalCategory::ContinuativeParticle)
        }
        ["助詞", "格助詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => {
            Some(LexicalCategory::CaseParticle)
        }
        ["助詞", "係助詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => {
            Some(LexicalCategory::ConnectingParticle)
        }
        ["形容詞", "非自立可能", _, _, _, "終止形-一般", _, "無い", _, _, _, _, _, _, _, _, _] => {
            Some(LexicalCategory::NegationDictForm)
        }
        ["形容詞", "非自立可能", _, _, _, "連用形-一般", _, "無い", _, _, _, _, _, _, _, _, _] => {
            Some(LexicalCategory::NegationConjForm)
        }
        ["形容詞", "非自立可能", _, _, _, "連用形-促音便", _, "無い", _, _, _, _, _, _, _, _, _] => {
            Some(LexicalCategory::NegationConjForm)
        }
        ["形容詞", _, _, _, _, "終止形-一般", _, _, _, _, _, _, _, _, _, _, _] => {
            Some(LexicalCategory::AdjectiveIDictForm)
        }
        ["形容詞", _, _, _, _, "連用形-一般", _, _, _, _, _, _, _, _, _, _, _] => {
            Some(LexicalCategory::AdjectiveIConjForm)
        }
        ["形容詞", _, _, _, _, "連用形-促音便", _, _, _, _, _, _, _, _, _, _, _] => {
            Some(LexicalCategory::AdjectiveIConjForm)
        }
        ["副詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => Some(LexicalCategory::Adverb),
        _ => None,
    }
}

#[tokio::test]
async fn 降る_is_verb() {
    let details = tokenize("降る").await.unwrap();
    assert_eq!(details.len(), 1);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Verb));
}

#[tokio::test]
async fn 降ります_is_verb() {
    let details = tokenize("降ります").await.unwrap();
    assert_eq!(details.len(), 2);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Verb));
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::AuxiliaryVerb)
    );
}

#[tokio::test]
async fn 降って_is_verb() {
    let details = tokenize("降って").await.unwrap();
    assert_eq!(details.len(), 2);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Verb));
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::ContinuativeParticle)
    );
}

#[tokio::test]
async fn 降った_is_verb() {
    let details = tokenize("降った").await.unwrap();
    assert_eq!(details.len(), 2);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Verb));
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::AuxiliaryVerb)
    );
}

#[tokio::test]
async fn 降りました_is_verb() {
    let details = tokenize("降りました").await.unwrap();
    assert_eq!(details.len(), 3);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Verb));
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::ContinuativeAuxiliary)
    );
    assert_eq!(
        categorize(&details[2]),
        Some(LexicalCategory::AuxiliaryVerb)
    );
}

#[tokio::test]
async fn 降らない_is_verb() {
    let details = tokenize("降らない").await.unwrap();
    assert_eq!(details.len(), 2);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Verb));
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::AuxiliaryVerb)
    );
}

#[tokio::test]
async fn ために() {
    let details = tokenize("ために").await.unwrap();
    assert_eq!(details.len(), 2);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Noun));
    assert_eq!(categorize(&details[1]), Some(LexicalCategory::CaseParticle));
}

#[tokio::test]
async fn ため() {
    let details = tokenize("ため").await.unwrap();
    assert_eq!(details.len(), 1);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Noun));
}

#[tokio::test]
async fn たかくない() {
    let details = tokenize("たかくない").await.unwrap();
    assert_eq!(details.len(), 2);
    assert_eq!(
        categorize(&details[0]),
        Some(LexicalCategory::AdjectiveIConjForm)
    );
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::NegationDictForm)
    );
}

#[tokio::test]
async fn 大抵_is_adverb() {
    let details = tokenize("大抵").await.unwrap();
    assert_eq!(details.len(), 1);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Adverb));
}

#[tokio::test]
async fn あまり_is_adverb() {
    let details = tokenize("あまり").await.unwrap();
    assert_eq!(details.len(), 1);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Adverb));
}

#[tokio::test]
async fn 簡単じゃありません() {
    let details = tokenize("簡単じゃありません").await.unwrap();
    assert_eq!(details.len(), 5);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Keiyoudoushi));
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::AuxiliaryVerb)
    );
    assert_eq!(categorize(&details[2]), Some(LexicalCategory::Verb));
    assert_eq!(
        categorize(&details[3]),
        Some(LexicalCategory::AuxiliaryVerb)
    );
    assert_eq!(
        categorize(&details[4]),
        Some(LexicalCategory::AuxiliaryVerb)
    );
}

#[tokio::test]
async fn 簡単ではありません() {
    let details = tokenize("簡単ではありません").await.unwrap();
    assert_eq!(details.len(), 6);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Keiyoudoushi));
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::ContinuativeAuxiliary)
    );
    assert_eq!(
        categorize(&details[2]),
        Some(LexicalCategory::ConnectingParticle)
    );
    assert_eq!(categorize(&details[3]), Some(LexicalCategory::Verb));
    assert_eq!(
        categorize(&details[4]),
        Some(LexicalCategory::AuxiliaryVerb)
    );
    assert_eq!(
        categorize(&details[5]),
        Some(LexicalCategory::AuxiliaryVerb)
    );
}

#[tokio::test]
async fn 簡単ではない() {
    let details = tokenize("簡単ではない").await.unwrap();
    assert_eq!(details.len(), 4);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Keiyoudoushi));
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::ContinuativeAuxiliary)
    );
    assert_eq!(
        categorize(&details[2]),
        Some(LexicalCategory::ConnectingParticle)
    );
    assert_eq!(
        categorize(&details[3]),
        Some(LexicalCategory::NegationDictForm)
    );
}

#[tokio::test]
async fn 簡単じゃない() {
    let details = tokenize("簡単じゃない").await.unwrap();
    assert_eq!(details.len(), 3);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Keiyoudoushi));
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::AuxiliaryVerb)
    );
    assert_eq!(
        categorize(&details[2]),
        Some(LexicalCategory::NegationDictForm)
    );
}

#[tokio::test]
async fn 難しくなかった() {
    let details = tokenize("難しくなかった").await.unwrap();
    assert_eq!(details.len(), 3);
    assert_eq!(
        categorize(&details[0]),
        Some(LexicalCategory::AdjectiveIConjForm)
    );
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::NegationConjForm)
    );
    assert_eq!(
        categorize(&details[2]),
        Some(LexicalCategory::AuxiliaryVerb)
    );
}

#[tokio::test]
async fn 難しくなくて() {
    let details = tokenize("難しくなくて").await.unwrap();
    assert_eq!(details.len(), 3);
    assert_eq!(
        categorize(&details[0]),
        Some(LexicalCategory::AdjectiveIConjForm)
    );
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::NegationConjForm)
    );
    assert_eq!(
        categorize(&details[2]),
        Some(LexicalCategory::ContinuativeParticle)
    );
}

#[tokio::test]
async fn 難しくて() {
    let details = tokenize("難しくて").await.unwrap();
    assert_eq!(details.len(), 2);
    assert_eq!(
        categorize(&details[0]),
        Some(LexicalCategory::AdjectiveIConjForm)
    );
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::ContinuativeParticle)
    );
}

#[tokio::test]
async fn 難しく() {
    let details = tokenize("難しく").await.unwrap();
    assert_eq!(details.len(), 1);
    assert_eq!(
        categorize(&details[0]),
        Some(LexicalCategory::AdjectiveIConjForm)
    );
}

#[tokio::test]
async fn 難しかった() {
    let details = tokenize("難しかった").await.unwrap();
    assert_eq!(details.len(), 2);
    assert_eq!(
        categorize(&details[0]),
        Some(LexicalCategory::AdjectiveIConjForm)
    );
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::AuxiliaryVerb)
    );
}

#[tokio::test]
async fn 難しくない() {
    let details = tokenize("難しくない").await.unwrap();
    assert_eq!(details.len(), 2);
    assert_eq!(
        categorize(&details[0]),
        Some(LexicalCategory::AdjectiveIConjForm)
    );
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::NegationDictForm)
    );
}

#[tokio::test]
async fn 難しい() {
    let details = tokenize("難しい").await.unwrap();
    assert_eq!(details.len(), 1);
    assert_eq!(
        categorize(&details[0]),
        Some(LexicalCategory::AdjectiveIDictForm)
    );
}

#[tokio::test]
async fn 新幹線() {
    let details = tokenize("新幹線").await.unwrap();
    assert_eq!(details.len(), 2);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Prefix));
    assert_eq!(categorize(&details[1]), Some(LexicalCategory::Noun));
}

#[tokio::test]
async fn 重大な_is_keiyoudoushi() {
    let details = tokenize("重大な").await.unwrap();
    assert_eq!(details.len(), 2);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Keiyoudoushi));
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::AttributiveParticle)
    );
}

#[tokio::test]
async fn 重大_is_keiyoudoushi() {
    let details = tokenize("重大").await.unwrap();
    assert_eq!(details.len(), 1);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Keiyoudoushi));
}

#[tokio::test]
async fn きれいで() {
    let details = tokenize("きれいで").await.unwrap();
    assert_eq!(details.len(), 2);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Keiyoudoushi));
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::ContinuativeAuxiliary)
    );
}

#[tokio::test]
async fn きれいに_is_keiyoudoushi() {
    let details = tokenize("きれいに").await.unwrap();
    assert_eq!(details.len(), 2);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Keiyoudoushi));
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::AdverbificationParticle)
    );
}

#[tokio::test]
async fn きれいな_is_keiyoudoushi() {
    let details = tokenize("きれいな").await.unwrap();
    assert_eq!(details.len(), 2);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Keiyoudoushi));
    assert_eq!(
        categorize(&details[1]),
        Some(LexicalCategory::AttributiveParticle)
    );
}

#[tokio::test]
async fn きれい_is_keiyoudoushi() {
    let details = tokenize("きれい").await.unwrap();
    assert_eq!(details.len(), 1);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Keiyoudoushi));
}

#[tokio::test]
async fn うちにいる() {
    let details = tokenize("うちにいる").await.unwrap();
    assert_eq!(details.len(), 3);
    assert_eq!(categorize(&details[0]), Some(LexicalCategory::Noun));
    assert_eq!(categorize(&details[1]), Some(LexicalCategory::CaseParticle));
    assert_eq!(categorize(&details[2]), Some(LexicalCategory::Verb));
}
