use ::proto::{api_client::ApiClient, Sentence};
use proto::TokenList;
use tokio::runtime::{Builder, Runtime};
use tonic::codegen::StdError;

pub struct BlockingClient {
    client: ApiClient<tonic::transport::Channel>,
    rt: Runtime,
}

impl BlockingClient {
    pub fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
    where
        D: std::convert::TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        let rt = Builder::new_multi_thread().enable_all().build().unwrap();
        let client = rt.block_on(ApiClient::connect(dst))?;

        Ok(Self { client, rt })
    }

    pub fn analyze(
        &mut self,
        // text: &str
        request: impl tonic::IntoRequest<Sentence>,
    ) -> Result<tonic::Response<TokenList>, tonic::Status> {
        self.rt.block_on(self.client.analyze(request))
    }
}
