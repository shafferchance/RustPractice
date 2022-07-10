use simple_rust_web_app::net::server::Server;

#[async_std::main]
async fn main() {
    let listener = Server::new("127.0.0.1", "7878").await;

    listener.listen().await;
}
