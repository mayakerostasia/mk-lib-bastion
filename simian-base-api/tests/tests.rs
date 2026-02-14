mod things;
pub use things::client::TestClient;
use things::client::{test_paged_call, test_rest_call};

#[tokio::test]
async fn test_rest_client() {
    let client = TestClient::new().await;
    let _ = test_rest_call(&client).await;
    let _ = test_paged_call(&client).await;
    drop(client);
}
