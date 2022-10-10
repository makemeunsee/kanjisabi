use std::{
    fmt,
    net::{SocketAddr, ToSocketAddrs},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

// TODO select via argument, not via feature
// #[cfg(all(feature = "ipadic", feature = "unidic"))]
// compile_error!("feature \"ipadic\" and feature \"unidic\" cannot be enabled at the same time");

#[derive(Serialize, Deserialize, Debug)]
struct LinderaToken {
    detail: Vec<String>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Category {
    Prefix,
    Noun,
    Keiyoudoushi,
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
    Other,
    // below: yet unused
    ProperNoun,
    NounSuffix,
    Pronoun,
    SuruVerb,
}

impl fmt::Display for Category {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, formatter)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Morpheme {
    text: String,
    dict_form: String,
    category: Category,
}

impl Morpheme {
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn dict_form(&self) -> &str {
        &self.dict_form
    }
    pub fn category(&self) -> &Category {
        &self.category
    }
}

pub struct JpnMorphAnalysisAPI {
    lindera_addr: SocketAddr,
}

impl Default for JpnMorphAnalysisAPI {
    fn default() -> Self {
        Self::with_lindera_address("0.0.0.0:3333").unwrap()
    }
}

impl JpnMorphAnalysisAPI {
    pub fn with_lindera_address(lindera_addr: impl ToSocketAddrs) -> Result<Self> {
        let lindera_addr = lindera_addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow::anyhow!("failed to resolve `lindera_addr`"))?;
        Ok(JpnMorphAnalysisAPI { lindera_addr })
    }

    pub async fn morphemes(&self, text: &str) -> Result<Vec<Morpheme>> {
        Ok(self
            .lindera_tokens(text)
            .await?
            .into_iter()
            .filter_map(|t| categorize(&t))
            .collect())
    }

    async fn lindera_tokens(&self, text: &str) -> Result<Vec<Vec<String>>> {
        let client = reqwest::Client::new();
        let response = client
            .post(format!("http://{}/tokenize", self.lindera_addr))
            .body(text.to_owned())
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        Ok(serde_json::from_str::<Vec<LinderaToken>>(&response)?
            .into_iter()
            .map(move |token| token.detail)
            .collect::<Vec<Vec<String>>>())
    }
}

// #[cfg(feature = "ipadic")]
// pub fn categorize(details: &Vec<String>) -> Option<Morpheme> {
// println!("{:?}", details);
// match details
//     .iter()
//     .map(String::as_str)
//     .collect::<Vec<&str>>()
//     .as_slice()
// {
//     // Verb stems
//     ["動詞", _, _, _, _, _, _, _, _] => Some(Category::Verb),

//     // Noun stems
//     // i.e. noun adjective, na-adj
//     ["名詞", "形容動詞語幹", _, _, _, _, _, _, _] => Some(Category::Keiyoudoushi),
//     ["名詞", "一般", _, _, _, _, _, _, _] => Some(Category::Noun),
//     // number
//     ["名詞", "数", _, _, _, _, _, _, _] => Some(Category::Noun),
//     // possible adverb, e.g. それぞれ
//     // TODO adverb and/or noun?
//     ["名詞", "副詞可能", _, _, _, _, _, _, _] => Some(Category::Noun),
//     ["名詞", "接尾", _, _, _, _, _, _, _] => Some(Category::NounSuffix),
//     ["名詞", "固有名詞", _, _, _, _, _, _, _] => Some(Category::ProperNoun),
//     ["名詞", "サ変接続", _, _, _, _, _, _, _] => Some(Category::SuruVerb),
//     ["名詞", _, _, _, _, _, _, _, _] => Some(Category::Noun),

//     // Adjective stems
//     ["形容詞", _, _, _, "形容詞・イ段", "基本形", _, _, _] => {
//         Some(Category::AdjectiveI)
//     }
//     // adverbial form (く) of i-adj
//     // TODO validate with more cases
//     ["形容詞", _, _, _, _, "連用テ接続", _, _, _] => Some(Category::Adverb),
//     ["形容詞", _, _, _, _, _, _, _, _] => Some(Category::AdjectiveI),

//     // Copulas, auxiliary verbs
//     // e.g. な of na-adj
//     ["助動詞", _, _, _, _, "体言接続", _, _, _] => Some(Category::AuxiliaryNa),
//     // continuous (て) / adverbial form
//     ["助動詞", _, _, _, _, "連用テ接続", _, _, _] => Some(Category::Adverb),
//     // e.g. だ、です、まし(た)
//     ["助動詞", _, _, _, _, _, _, _, _] => Some(Category::AuxiliaryVerb),

//     // Particles
//     // e.g. に of きれいに
//     ["助詞", "副詞化", _, _, _, _, _, _, _] => Some(Category::AdverbificationParticle),
//     // e.g. −て
//     ["助詞", "接続助詞", _, _, _, _, _, _, _] => Some(Category::ConjunctionParticle),
//     // e.g. −の−
//     ["助詞", "連体化", _, _, _, _, _, _, _] => Some(Category::AdjectivisationParticle),
//     // case marking particle
//     ["助詞", "格助詞", _, _, _, _, _, _, _] => Some(Category::Particle),
//     // binding particle
//     ["助詞", "係助詞", _, _, _, _, _, _, _] => Some(Category::Particle),
//     // adverbial particle e.g. まで
//     ["助詞", "副助詞", _, _, _, _, _, _, _] => Some(Category::Particle),
//     ["助詞", _, _, _, _, _, _, _, _] => Some(Category::Particle),

//     // Adverbs
//     ["副詞", _, _, _, _, _, _, _, _] => Some(Category::Adverb),

//     _ => None,
// }
//     None
// }

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
//   自立: independent
//   非自立: not independent
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

#[cfg(feature = "unidic")]
pub fn categorize(details: &Vec<String>) -> Option<Morpheme> {
    if details.len() != 17 {
        return None;
    }
    //TODO remove
    println!("{:?}", details);
    let category = match details
        .iter()
        .map(String::as_str)
        .collect::<Vec<&str>>()
        .as_slice()
    {
        ["動詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => Category::Verb,
        ["名詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => Category::Noun,
        ["接頭辞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => Category::Prefix,
        ["形状詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => Category::Keiyoudoushi,
        ["助動詞", _, _, _, _, "連体形-一般", _, _, "な", _, _, _, _, _, _, _, _] => {
            Category::AttributiveParticle
        }
        ["助動詞", _, _, _, _, _, _, _, "に", _, _, _, _, _, _, _, _] => {
            Category::AdverbificationParticle
        }
        ["助動詞", _, _, _, _, "連用形-一般", _, _, _, _, _, _, _, _, _, _, _] => {
            Category::ContinuativeAuxiliary
        }
        ["助動詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => Category::AuxiliaryVerb,
        ["助詞", "接続助詞", _, _, _, _, _, _, "て", _, _, _, _, _, _, _, _] => {
            Category::ContinuativeParticle
        }
        ["助詞", "格助詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => {
            Category::CaseParticle
        }
        ["助詞", "係助詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => {
            Category::ConnectingParticle
        }
        ["形容詞", "非自立可能", _, _, _, "終止形-一般", _, "無い", _, _, _, _, _, _, _, _, _] => {
            Category::NegationDictForm
        }
        ["形容詞", "非自立可能", _, _, _, "連用形-一般", _, "無い", _, _, _, _, _, _, _, _, _] => {
            Category::NegationConjForm
        }
        ["形容詞", "非自立可能", _, _, _, "連用形-促音便", _, "無い", _, _, _, _, _, _, _, _, _] => {
            Category::NegationConjForm
        }
        ["形容詞", _, _, _, _, "終止形-一般", _, _, _, _, _, _, _, _, _, _, _] => {
            Category::AdjectiveIDictForm
        }
        ["形容詞", _, _, _, _, "連用形-一般", _, _, _, _, _, _, _, _, _, _, _] => {
            Category::AdjectiveIConjForm
        }
        ["形容詞", _, _, _, _, "連用形-促音便", _, _, _, _, _, _, _, _, _, _, _] => {
            Category::AdjectiveIConjForm
        }
        ["補助記号", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => Category::Sign,
        ["副詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => Category::Adverb,
        _ => Category::Other,
    };
    Some(Morpheme {
        text: details[8].to_owned(),
        dict_form: details[10].to_owned(),
        category,
    })
}

#[cfg(feature = "unidic")]
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn 降る_is_verb() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("降る")
            .await
            .unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].category, Category::Verb);
    }

    #[tokio::test]
    async fn 降ります_is_verb() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("降ります")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, Category::Verb);
        assert_eq!(details[1].category, Category::AuxiliaryVerb);
    }

    #[tokio::test]
    async fn 降って_is_verb() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("降って")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, Category::Verb);
        assert_eq!(details[1].category, Category::ContinuativeParticle);
    }

    #[tokio::test]
    async fn 降った_is_verb() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("降った")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, Category::Verb);
        assert_eq!(details[1].category, Category::AuxiliaryVerb);
    }

    #[tokio::test]
    async fn 降りました_is_verb() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("降りました")
            .await
            .unwrap();
        assert_eq!(details.len(), 3);
        assert_eq!(details[0].category, Category::Verb);
        assert_eq!(details[1].category, Category::ContinuativeAuxiliary);
        assert_eq!(details[2].category, Category::AuxiliaryVerb);
    }

    #[tokio::test]
    async fn 降らない_is_verb() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("降らない")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, Category::Verb);
        assert_eq!(details[1].category, Category::AuxiliaryVerb);
    }

    #[tokio::test]
    async fn ために() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("ために")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, Category::Noun);
        assert_eq!(details[1].category, Category::CaseParticle);
    }

    #[tokio::test]
    async fn ため() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("ため")
            .await
            .unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].category, Category::Noun);
    }

    #[tokio::test]
    async fn たかくない() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("たかくない")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, Category::AdjectiveIConjForm);
        assert_eq!(details[1].category, Category::NegationDictForm);
    }

    #[tokio::test]
    async fn 大抵_is_adverb() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("大抵")
            .await
            .unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].category, Category::Adverb);
    }

    #[tokio::test]
    async fn あまり_is_adverb() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("あまり")
            .await
            .unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].category, Category::Adverb);
    }

    #[tokio::test]
    async fn 簡単じゃありません() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("簡単じゃありません")
            .await
            .unwrap();
        assert_eq!(details.len(), 5);
        assert_eq!(details[0].category, Category::Keiyoudoushi);
        assert_eq!(details[1].category, Category::AuxiliaryVerb);
        assert_eq!(details[2].category, Category::Verb);
        assert_eq!(details[3].category, Category::AuxiliaryVerb);
        assert_eq!(details[4].category, Category::AuxiliaryVerb);
    }

    #[tokio::test]
    async fn 簡単ではありません() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("簡単ではありません")
            .await
            .unwrap();
        assert_eq!(details.len(), 6);
        assert_eq!(details[0].category, Category::Keiyoudoushi);
        assert_eq!(details[1].category, Category::ContinuativeAuxiliary);
        assert_eq!(details[2].category, Category::ConnectingParticle);
        assert_eq!(details[3].category, Category::Verb);
        assert_eq!(details[4].category, Category::AuxiliaryVerb);
        assert_eq!(details[5].category, Category::AuxiliaryVerb);
    }

    #[tokio::test]
    async fn 簡単ではない() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("簡単ではない")
            .await
            .unwrap();
        assert_eq!(details.len(), 4);
        assert_eq!(details[0].category, Category::Keiyoudoushi);
        assert_eq!(details[1].category, Category::ContinuativeAuxiliary);
        assert_eq!(details[2].category, Category::ConnectingParticle);
        assert_eq!(details[3].category, Category::NegationDictForm);
    }

    #[tokio::test]
    async fn 簡単じゃない() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("簡単じゃない")
            .await
            .unwrap();
        assert_eq!(details.len(), 3);
        assert_eq!(details[0].category, Category::Keiyoudoushi);
        assert_eq!(details[1].category, Category::AuxiliaryVerb);
        assert_eq!(details[2].category, Category::NegationDictForm);
    }

    #[tokio::test]
    async fn 難しくなかった() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("難しくなかった")
            .await
            .unwrap();
        assert_eq!(details.len(), 3);
        assert_eq!(details[0].category, Category::AdjectiveIConjForm);
        assert_eq!(details[1].category, Category::NegationConjForm);
        assert_eq!(details[2].category, Category::AuxiliaryVerb);
    }

    #[tokio::test]
    async fn 難しくなくて() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("難しくなくて")
            .await
            .unwrap();
        assert_eq!(details.len(), 3);
        assert_eq!(details[0].category, Category::AdjectiveIConjForm);
        assert_eq!(details[1].category, Category::NegationConjForm);
        assert_eq!(details[2].category, Category::ContinuativeParticle);
    }

    #[tokio::test]
    async fn 難しくて() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("難しくて")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, Category::AdjectiveIConjForm);
        assert_eq!(details[1].category, Category::ContinuativeParticle);
    }

    #[tokio::test]
    async fn 難しく() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("難しく")
            .await
            .unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].category, Category::AdjectiveIConjForm);
    }

    #[tokio::test]
    async fn 難しかった() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("難しかった")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, Category::AdjectiveIConjForm);
        assert_eq!(details[1].category, Category::AuxiliaryVerb);
    }

    #[tokio::test]
    async fn 難しくない() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("難しくない")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, Category::AdjectiveIConjForm);
        assert_eq!(details[1].category, Category::NegationDictForm);
    }

    #[tokio::test]
    async fn 難しい() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("難しい")
            .await
            .unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].category, Category::AdjectiveIDictForm);
    }

    #[tokio::test]
    async fn 新幹線() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("新幹線")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, Category::Prefix);
        assert_eq!(details[1].category, Category::Noun);
    }

    #[tokio::test]
    async fn 重大な_is_keiyoudoushi() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("重大な")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, Category::Keiyoudoushi);
        assert_eq!(details[1].category, Category::AttributiveParticle);
    }

    #[tokio::test]
    async fn 重大_is_keiyoudoushi() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("重大")
            .await
            .unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].category, Category::Keiyoudoushi);
    }

    #[tokio::test]
    async fn きれいで() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("きれいで")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, Category::Keiyoudoushi);
        assert_eq!(details[1].category, Category::ContinuativeAuxiliary);
    }

    #[tokio::test]
    async fn きれいに_is_keiyoudoushi() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("きれいに")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, Category::Keiyoudoushi);
        assert_eq!(details[1].category, Category::AdverbificationParticle);
    }

    #[tokio::test]
    async fn きれいな_is_keiyoudoushi() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("きれいな")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, Category::Keiyoudoushi);
        assert_eq!(details[1].category, Category::AttributiveParticle);
    }

    #[tokio::test]
    async fn きれい_is_keiyoudoushi() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("きれい")
            .await
            .unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].category, Category::Keiyoudoushi);
    }

    #[tokio::test]
    async fn うちにいる() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("うちにいる")
            .await
            .unwrap();
        assert_eq!(details.len(), 3);
        assert_eq!(details[0].category, Category::Noun);
        assert_eq!(details[1].category, Category::CaseParticle);
        assert_eq!(details[2].category, Category::Verb);
    }
}
