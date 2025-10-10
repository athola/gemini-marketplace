use std::net::TcpListener;

use axum::Server;
use gemini_marketplace::marketplace::api::server::ApiServer;
use tokio::task::JoinHandle;

async fn spawn_server() -> (String, JoinHandle<()>) {
    let server = ApiServer::new();
    let router = server.router();

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
    let addr = listener.local_addr().expect("address");
    let handle = tokio::spawn(async move {
        Server::from_tcp(listener)
            .expect("server")
            .serve(router.into_make_service())
            .await
            .unwrap();
    });

    (format!("http://{addr}"), handle)
}

#[tokio::test]
async fn status_endpoint_returns_not_implemented() {
    let (base, handle) = spawn_server().await;
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/marketplace/status", base))
        .send()
        .await
        .expect("request succeeds");

    assert_eq!(response.status(), reqwest::StatusCode::NOT_IMPLEMENTED);

    handle.abort();
}
