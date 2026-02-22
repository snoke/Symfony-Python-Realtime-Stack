mod services;

#[tokio::main]
async fn main() {
    services::app::run().await;
}
