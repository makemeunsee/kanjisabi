use morph_client::AsyncClient;
use proto::Sentence;
use tests::test_with_server;

#[tokio::test]
async fn analyze() {
    let test = |api_addr| async move {
        let mut client = AsyncClient::connect(api_addr).await.unwrap();

        let morphemes = client
            .analyze(Sentence {
                sentence: "。".to_owned(),
            })
            .await
            .unwrap()
            .into_inner()
            .morphemes;
        assert_eq!(morphemes.len(), 1);
        assert_eq!(morphemes[0].category, proto::Category::Sign as i32);
    };

    test_with_server(test).await.unwrap();
}
