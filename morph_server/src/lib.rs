use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use anyhow::Result;
use proto::{
    api_server::{Api, ApiServer},
    Analysis, Morpheme,
};
use serde::{Deserialize, Serialize};
use tonic::{transport::Server, Response, Status};

// TODO select via argument, not via feature
#[cfg(all(feature = "ipadic", feature = "unidic"))]
compile_error!("feature \"ipadic\" and feature \"unidic\" cannot be enabled at the same time");

#[cfg(feature = "ipadic")]
const DICT_NAME: &str = "ipadic";

#[cfg(feature = "unidic")]
const DICT_NAME: &str = "unidic";

#[derive(Debug, Clone, Copy)]
pub struct ServerConfig {
    /// The socket address of the gRPC JpnMorphAnalysisAPI::new().
    pub addr: SocketAddr,
    /// The socket address of the Lindera server.
    pub lindera_addr: SocketAddr,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 55555),
            lindera_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3333),
        }
    }
}

pub async fn serve(config: ServerConfig) {
    Server::builder()
        .add_service(ApiServer::new(JpnMorphAnalysisAPI {
            lindera_addr: config.lindera_addr,
        }))
        .serve(config.addr)
        .await
        .unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
struct LinderaToken {
    detail: Vec<String>,
}

// #[derive(PartialEq, Eq, Debug)]
// pub enum proto::Category {
//     Prefix,
//     Noun,
//     Keiyoudoushi,
//     AdjectiveIDictForm,
//     AdjectiveIConjForm,
//     NegationDictForm,
//     NegationConjForm,
//     Adverb,
//     AdverbificationParticle,
//     AdjectivisationParticle,
//     ContinuativeParticle,
//     ContinuativeAuxiliary,
//     Verb,
//     AuxiliaryVerb,
//     AttributiveParticle,
//     Particle,
//     CaseParticle,
//     ConnectingParticle,
//     Sign,
//     Other,
//     // below: yet unused
//     ProperNoun,
//     NounSuffix,
//     Pronoun,
//     SuruVerb,
// }

pub struct JpnMorphAnalysisAPI {
    lindera_addr: SocketAddr,
}

#[tonic::async_trait]
impl Api for JpnMorphAnalysisAPI {
    async fn dictionary(
        &self,
        _: tonic::Request<proto::Empty>,
    ) -> Result<tonic::Response<proto::DictName>, tonic::Status> {
        Ok(Response::new(proto::DictName {
            name: DICT_NAME.to_owned(),
        }))
    }
    async fn analyze(
        &self,
        request: tonic::Request<proto::Sentence>,
    ) -> Result<tonic::Response<proto::Analysis>, tonic::Status> {
        self.morphemes(&request.into_inner().sentence)
            .await
            .map(|morphemes| Response::new(Analysis { morphemes }))
            .map_err(|e| Status::internal(e.to_string()))
    }
}

impl JpnMorphAnalysisAPI {
    pub fn default() -> Self {
        let lindera_addr = env::var("LINDERA_ADDR")
            .unwrap_or("0.0.0.0:3333".to_owned())
            .parse()
            .unwrap();
        JpnMorphAnalysisAPI { lindera_addr }
    }

    pub async fn morphemes(&self, text: &str) -> Result<Vec<Morpheme>> {
        Ok(self
            .lindera_tokens(text)
            .await?
            .into_iter()
            .filter_map(move |t| categorize(t))
            .collect())
    }

    // TODO README
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
        let tokens: Vec<LinderaToken> = serde_json::from_str(&response)?;
        Ok(tokens
            .iter()
            .map(|token| token.detail.clone())
            .collect::<Vec<Vec<String>>>())
    }
}

#[cfg(feature = "ipadic")]
pub fn categorize(details: &Vec<String>) -> Option<proto::Category> {
    println!("{:?}", details);
    match details
        .iter()
        .map(String::as_str)
        .collect::<Vec<&str>>()
        .as_slice()
    {
        // Verb stems
        ["動詞", _, _, _, _, _, _, _, _] => Some(proto::Category::Verb),

        // Noun stems
        // i.e. noun adjective, na-adj
        ["名詞", "形容動詞語幹", _, _, _, _, _, _, _] => {
            Some(proto::Category::Keiyoudoushi)
        }
        ["名詞", "一般", _, _, _, _, _, _, _] => Some(proto::Category::Noun),
        // number
        ["名詞", "数", _, _, _, _, _, _, _] => Some(proto::Category::Noun),
        // possible adverb, e.g. それぞれ
        // TODO adverb and/or noun?
        ["名詞", "副詞可能", _, _, _, _, _, _, _] => Some(proto::Category::Noun),
        ["名詞", "接尾", _, _, _, _, _, _, _] => Some(proto::Category::NounSuffix),
        ["名詞", "固有名詞", _, _, _, _, _, _, _] => Some(proto::Category::ProperNoun),
        ["名詞", "サ変接続", _, _, _, _, _, _, _] => Some(proto::Category::SuruVerb),
        ["名詞", _, _, _, _, _, _, _, _] => Some(proto::Category::Noun),

        // Adjective stems
        ["形容詞", _, _, _, "形容詞・イ段", "基本形", _, _, _] => {
            Some(proto::Category::AdjectiveI)
        }
        // adverbial form (く) of i-adj
        // TODO validate with more cases
        ["形容詞", _, _, _, _, "連用テ接続", _, _, _] => Some(proto::Category::Adverb),
        ["形容詞", _, _, _, _, _, _, _, _] => Some(proto::Category::AdjectiveI),

        // Copulas, auxiliary verbs
        // e.g. な of na-adj
        ["助動詞", _, _, _, _, "体言接続", _, _, _] => Some(proto::Category::AuxiliaryNa),
        // continuous (て) / adverbial form
        ["助動詞", _, _, _, _, "連用テ接続", _, _, _] => Some(proto::Category::Adverb),
        // e.g. だ、です、まし(た)
        ["助動詞", _, _, _, _, _, _, _, _] => Some(proto::Category::AuxiliaryVerb),

        // Particles
        // e.g. に of きれいに
        ["助詞", "副詞化", _, _, _, _, _, _, _] => {
            Some(proto::Category::AdverbificationParticle)
        }
        // e.g. −て
        ["助詞", "接続助詞", _, _, _, _, _, _, _] => {
            Some(proto::Category::ConjunctionParticle)
        }
        // e.g. −の−
        ["助詞", "連体化", _, _, _, _, _, _, _] => {
            Some(proto::Category::AdjectivisationParticle)
        }
        // case marking particle
        ["助詞", "格助詞", _, _, _, _, _, _, _] => Some(proto::Category::Particle),
        // binding particle
        ["助詞", "係助詞", _, _, _, _, _, _, _] => Some(proto::Category::Particle),
        // adverbial particle e.g. まで
        ["助詞", "副助詞", _, _, _, _, _, _, _] => Some(proto::Category::Particle),
        ["助詞", _, _, _, _, _, _, _, _] => Some(proto::Category::Particle),

        // Adverbs
        ["副詞", _, _, _, _, _, _, _, _] => Some(proto::Category::Adverb),

        _ => None,
    }
}

#[cfg(feature = "unidic")]
pub fn categorize(details: Vec<String>) -> Option<Morpheme> {
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
        ["動詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => proto::Category::Verb,
        ["名詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => proto::Category::Noun,
        ["接頭辞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => proto::Category::Noun, // Prefix
        ["形状詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => {
            proto::Category::Keiyoudoushi
        }
        ["助動詞", _, _, _, _, "連体形-一般", _, _, "な", _, _, _, _, _, _, _, _] => {
            proto::Category::Particle // Attributive
        }
        ["助動詞", _, _, _, _, _, _, _, "に", _, _, _, _, _, _, _, _] => {
            proto::Category::Particle // Adverbification
        }
        ["助動詞", _, _, _, _, "連用形-一般", _, _, _, _, _, _, _, _, _, _, _] => {
            proto::Category::Conjunction // Continuative
        }
        ["助動詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => proto::Category::Auxiliary,
        ["助詞", "接続助詞", _, _, _, _, _, _, "て", _, _, _, _, _, _, _, _] => {
            proto::Category::Conjunction // Continuative
        }
        ["助詞", "格助詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => {
            proto::Category::Particle // Case
        }
        ["助詞", "係助詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => {
            proto::Category::Particle // Connecting
        }
        ["形容詞", "非自立可能", _, _, _, "終止形-一般", _, "無い", _, _, _, _, _, _, _, _, _] =>
        {
            proto::Category::Keiyoushi // NegationDictForm
        }
        ["形容詞", "非自立可能", _, _, _, "連用形-一般", _, "無い", _, _, _, _, _, _, _, _, _] =>
        {
            proto::Category::Conjunction // NegationConjForm
        }
        ["形容詞", "非自立可能", _, _, _, "連用形-促音便", _, "無い", _, _, _, _, _, _, _, _, _] =>
        {
            proto::Category::Conjunction // NegationConjForm
        }
        ["形容詞", _, _, _, _, "終止形-一般", _, _, _, _, _, _, _, _, _, _, _] => {
            proto::Category::Keiyoushi // AdjectiveIDictForm
        }
        ["形容詞", _, _, _, _, "連用形-一般", _, _, _, _, _, _, _, _, _, _, _] => {
            proto::Category::Conjunction // AdjectiveIConjForm
        }
        ["形容詞", _, _, _, _, "連用形-促音便", _, _, _, _, _, _, _, _, _, _, _] => {
            proto::Category::Conjunction // AdjectiveIConjForm
        }
        ["補助記号", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => proto::Category::Sign,
        ["副詞", _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => proto::Category::Adverb,
        _ => proto::Category::Other,
    };
    Some(Morpheme {
        text: details[8].clone(),
        dict_form: details[10].clone(),
        category: category as i32,
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
        assert_eq!(details[0].category, proto::Category::Verb as i32);
    }

    #[tokio::test]
    async fn 降ります_is_verb() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("降ります")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, proto::Category::Verb as i32);
        assert_eq!(details[1].category, proto::Category::Auxiliary as i32);
    }

    #[tokio::test]
    async fn 降って_is_verb() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("降って")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, proto::Category::Verb as i32);
        assert_eq!(details[1].category, proto::Category::Conjunction as i32);
    }

    #[tokio::test]
    async fn 降った_is_verb() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("降った")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, proto::Category::Verb as i32);
        assert_eq!(details[1].category, proto::Category::Auxiliary as i32);
    }

    #[tokio::test]
    async fn 降りました_is_verb() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("降りました")
            .await
            .unwrap();
        assert_eq!(details.len(), 3);
        assert_eq!(details[0].category, proto::Category::Verb as i32);
        assert_eq!(details[1].category, proto::Category::Conjunction as i32);
        assert_eq!(details[2].category, proto::Category::Auxiliary as i32);
    }

    #[tokio::test]
    async fn 降らない_is_verb() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("降らない")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, proto::Category::Verb as i32);
        assert_eq!(details[1].category, proto::Category::Auxiliary as i32);
    }

    #[tokio::test]
    async fn ために() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("ために")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, proto::Category::Noun as i32);
        assert_eq!(details[1].category, proto::Category::Particle as i32);
    }

    #[tokio::test]
    async fn ため() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("ため")
            .await
            .unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].category, proto::Category::Noun as i32);
    }

    #[tokio::test]
    async fn たかくない() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("たかくない")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, proto::Category::Conjunction as i32);
        assert_eq!(details[1].category, proto::Category::Keiyoushi as i32);
    }

    #[tokio::test]
    async fn 大抵_is_adverb() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("大抵")
            .await
            .unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].category, proto::Category::Adverb as i32);
    }

    #[tokio::test]
    async fn あまり_is_adverb() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("あまり")
            .await
            .unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].category, proto::Category::Adverb as i32);
    }

    #[tokio::test]
    async fn 簡単じゃありません() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("簡単じゃありません")
            .await
            .unwrap();
        assert_eq!(details.len(), 5);
        assert_eq!(details[0].category, proto::Category::Keiyoudoushi as i32);
        assert_eq!(details[1].category, proto::Category::Auxiliary as i32);
        assert_eq!(details[2].category, proto::Category::Verb as i32);
        assert_eq!(details[3].category, proto::Category::Auxiliary as i32);
        assert_eq!(details[4].category, proto::Category::Auxiliary as i32);
    }

    #[tokio::test]
    async fn 簡単ではありません() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("簡単ではありません")
            .await
            .unwrap();
        assert_eq!(details.len(), 6);
        assert_eq!(details[0].category, proto::Category::Keiyoudoushi as i32);
        assert_eq!(details[1].category, proto::Category::Conjunction as i32);
        assert_eq!(details[2].category, proto::Category::Particle as i32);
        assert_eq!(details[3].category, proto::Category::Verb as i32);
        assert_eq!(details[4].category, proto::Category::Auxiliary as i32);
        assert_eq!(details[5].category, proto::Category::Auxiliary as i32);
    }

    #[tokio::test]
    async fn 簡単ではない() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("簡単ではない")
            .await
            .unwrap();
        assert_eq!(details.len(), 4);
        assert_eq!(details[0].category, proto::Category::Keiyoudoushi as i32);
        assert_eq!(details[1].category, proto::Category::Conjunction as i32);
        assert_eq!(details[2].category, proto::Category::Particle as i32);
        assert_eq!(details[3].category, proto::Category::Keiyoushi as i32);
    }

    #[tokio::test]
    async fn 簡単じゃない() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("簡単じゃない")
            .await
            .unwrap();
        assert_eq!(details.len(), 3);
        assert_eq!(details[0].category, proto::Category::Keiyoudoushi as i32);
        assert_eq!(details[1].category, proto::Category::Auxiliary as i32);
        assert_eq!(details[2].category, proto::Category::Keiyoushi as i32);
    }

    #[tokio::test]
    async fn 難しくなかった() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("難しくなかった")
            .await
            .unwrap();
        assert_eq!(details.len(), 3);
        assert_eq!(details[0].category, proto::Category::Conjunction as i32);
        assert_eq!(details[1].category, proto::Category::Conjunction as i32);
        assert_eq!(details[2].category, proto::Category::Auxiliary as i32);
    }

    #[tokio::test]
    async fn 難しくなくて() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("難しくなくて")
            .await
            .unwrap();
        assert_eq!(details.len(), 3);
        assert_eq!(details[0].category, proto::Category::Conjunction as i32);
        assert_eq!(details[1].category, proto::Category::Conjunction as i32);
        assert_eq!(details[2].category, proto::Category::Conjunction as i32);
    }

    #[tokio::test]
    async fn 難しくて() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("難しくて")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, proto::Category::Conjunction as i32);
        assert_eq!(details[1].category, proto::Category::Conjunction as i32);
    }

    #[tokio::test]
    async fn 難しく() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("難しく")
            .await
            .unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].category, proto::Category::Conjunction as i32);
    }

    #[tokio::test]
    async fn 難しかった() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("難しかった")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, proto::Category::Conjunction as i32);
        assert_eq!(details[1].category, proto::Category::Auxiliary as i32);
    }

    #[tokio::test]
    async fn 難しくない() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("難しくない")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, proto::Category::Conjunction as i32);
        assert_eq!(details[1].category, proto::Category::Keiyoushi as i32);
    }

    #[tokio::test]
    async fn 難しい() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("難しい")
            .await
            .unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].category, proto::Category::Keiyoushi as i32);
    }

    #[tokio::test]
    async fn 新幹線() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("新幹線")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, proto::Category::Noun as i32);
        assert_eq!(details[1].category, proto::Category::Noun as i32);
    }

    #[tokio::test]
    async fn 重大な_is_keiyoudoushi() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("重大な")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, proto::Category::Keiyoudoushi as i32);
        assert_eq!(details[1].category, proto::Category::Particle as i32);
    }

    #[tokio::test]
    async fn 重大_is_keiyoudoushi() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("重大")
            .await
            .unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].category, proto::Category::Keiyoudoushi as i32);
    }

    #[tokio::test]
    async fn きれいで() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("きれいで")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, proto::Category::Keiyoudoushi as i32);
        assert_eq!(details[1].category, proto::Category::Conjunction as i32);
    }

    #[tokio::test]
    async fn きれいに_is_keiyoudoushi() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("きれいに")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, proto::Category::Keiyoudoushi as i32);
        assert_eq!(details[1].category, proto::Category::Particle as i32);
    }

    #[tokio::test]
    async fn きれいな_is_keiyoudoushi() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("きれいな")
            .await
            .unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details[0].category, proto::Category::Keiyoudoushi as i32);
        assert_eq!(details[1].category, proto::Category::Particle as i32);
    }

    #[tokio::test]
    async fn きれい_is_keiyoudoushi() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("きれい")
            .await
            .unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].category, proto::Category::Keiyoudoushi as i32);
    }

    #[tokio::test]
    async fn うちにいる() {
        let details = JpnMorphAnalysisAPI::default()
            .morphemes("うちにいる")
            .await
            .unwrap();
        assert_eq!(details.len(), 3);
        assert_eq!(details[0].category, proto::Category::Noun as i32);
        assert_eq!(details[1].category, proto::Category::Particle as i32);
        assert_eq!(details[2].category, proto::Category::Verb as i32);
    }
}
