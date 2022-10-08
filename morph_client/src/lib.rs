use std::net::ToSocketAddrs;

use anyhow::Result;
use proto::{api_client::ApiClient, Analysis, Sentence};
use tokio::runtime::{Builder, Runtime};

pub struct BlockingClient {
    client: ApiClient<tonic::transport::Channel>,
    rt: Runtime,
}

impl BlockingClient {
    pub fn connect(addr: impl ToSocketAddrs) -> Result<Self> {
        let addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow::anyhow!("failed to resolve `addr`"))?;

        let rt = Builder::new_multi_thread().enable_all().build().unwrap();
        let client = rt.block_on(ApiClient::connect(format!("http://{}", addr)))?;

        Ok(Self { client, rt })
    }

    pub fn analyze(
        &mut self,
        request: impl tonic::IntoRequest<Sentence>,
    ) -> Result<tonic::Response<Analysis>, tonic::Status> {
        self.rt.block_on(self.client.analyze(request))
    }
}
pub struct AsyncClient {
    client: ApiClient<tonic::transport::Channel>,
}

impl AsyncClient {
    pub async fn connect(addr: impl ToSocketAddrs) -> Result<Self> {
        let addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow::anyhow!("failed to resolve `addr`"))?;

        let client = ApiClient::connect(format!("http://{}", addr)).await?;

        Ok(Self { client })
    }

    pub async fn analyze(
        &mut self,
        request: impl tonic::IntoRequest<Sentence>,
    ) -> Result<tonic::Response<Analysis>, tonic::Status> {
        self.client.analyze(request).await
    }
}
