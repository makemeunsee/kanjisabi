use anyhow::Result;
use proto::{
    api_server::{Api, ApiServer},
    Token, TokenList,
};
use tonic::{transport::Server, Response};

pub struct MyApi {}

#[tonic::async_trait]
impl Api for MyApi {
    async fn dictionary(
        &self,
        _: tonic::Request<proto::Empty>,
    ) -> Result<tonic::Response<proto::DictName>, tonic::Status> {
        todo!()
    }

    async fn analyze(
        &self,
        request: tonic::Request<proto::Sentence>,
    ) -> Result<tonic::Response<proto::TokenList>, tonic::Status> {
        let tokens = self
            .tokenizer
            .tokenize(&request.into_inner().sentence)
            .unwrap()
            .iter()
            .map(|token| Token {
                text: token.text.into(),
                dict_form: todo!(),
                part_of_speech: todo!(),
            })
            .collect();
        Ok(Response::new(TokenList { tokens }))
    }
}

// TODO config/args
#[tokio::main]
async fn main() -> Result<()> {
    let addr = "[::1]:55555".parse().unwrap();
    let api = MyApi {
        tokenizer: Tokenizer::new().unwrap(),
    };

    Server::builder()
        .add_service(ApiServer::new(api))
        .serve(addr)
        .await?;

    Ok(())
}
