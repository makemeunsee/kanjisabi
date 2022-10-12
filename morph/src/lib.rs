#![feature(iter_intersperse)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, ToSocketAddrs};

// TODO select via argument, not via feature
// #[cfg(all(feature = "ipadic", feature = "unidic"))]
// compile_error!("feature \"ipadic\" and feature \"unidic\" cannot be enabled at the same time");

#[derive(Serialize, Deserialize, Debug)]
struct LinderaToken {
    detail: Vec<String>,
}

// TODO: integrate these translations (unidic)
// https://gist.github.com/masayu-a/e3eee0637c07d4019ec9
// https://gist.github.com/masayu-a/3e11168f9330e2d83a68
// https://gist.github.com/masayu-a/b3ce862336e47736e84f

#[derive(PartialEq, Eq, Debug)]
pub struct Morpheme {
    pub text: String,
    pub lemma: String,
    pub pronounciation: String,
    pub part_of_speech: String,
    pub inflection_type: Option<String>,
    pub inflection_form: Option<String>,
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
            .filter_map(categorize)
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

// see https://hayashibe.jp/tr/mecab/dictionary/ipadic
// #[cfg(feature = "ipadic")]
// pub fn categorize(details: &Vec<String>) -> Option<Morpheme> {
//     todo!()
// }

/*
from: https://hayashibe.jp/tr/mecab/dictionary/unidic/field

1 	品詞大分類 	pos1 	語形
2 	品詞中分類 	pos2 	語形
3 	品詞小分類 	pos3 	語形
4 	品詞細分類 	pos4 	語形
5 	活用型 	cType 	語形
6 	活用形 	cForm 	語形
7 	語彙素読み 	lForm 	語彙素 	lemmaのカタカナ表記
8 	語彙素表記 	lemma 	語彙素 	語彙素見出し
9 	書字形出現形 	orth 	書字形 	orthBaseが活用変化を受けたもの
10 	発音形出現形 	pron 	発音形 	pronBaseが活用変化を受けたもの
11 	書字形基本形 	orthBase 	書字形 	書字形見出し
12 	発音形基本形 	pronBase 	発音形 	発音形見出し（カタカナ表記）
13 	語種 	goshu 	語彙素
14 	語頭変化型 	iType 	語形
15 	語頭変化形 	iForm 	語形
16 	語末変化型 	fType 	語形
17 	語末変化形 	fForm 	語形

=>

[0,        1,        2,        3,       4,              5,              6,         7,    8,        9,             10,           11,        12,    13,   14,   15,   16]
[pos_major,pos_minor,pos_small,pos_tiny,inflection_type,inflection_form,lemma_kata,lemma,inflected,inflected_kata,lemma_written,lemma_kata,origin,iType,iForm,fType,fForm]
*/
#[cfg(feature = "unidic")]
pub fn categorize(details: Vec<String>) -> Option<Morpheme> {
    log::debug!("Lindera's output: {:?}", details);
    if details.len() != 17 {
        return None;
    }

    let text = details[8].to_owned();
    let lemma = details[7].to_owned();
    let pronounciation = details[6].to_owned();
    let inflection_type = Some(details[4].to_owned()).filter(|s| *s != "*");
    let inflection_form = Some(details[5].to_owned()).filter(|s| *s != "*");
    let part_of_speech = details
        .into_iter()
        .take(4)
        .take_while(|s| *s != "*")
        .intersperse("-".to_owned())
        .collect();

    Some(Morpheme {
        text,
        lemma,
        pronounciation,
        part_of_speech,
        inflection_type,
        inflection_form,
    })
}
