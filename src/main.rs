use netowiru::tools::pingpong::Server;
use netowiru::tools::pingpong::Ping;
use netowiru::proxy::config::ProxyConfig;

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let server = Server::new("127.0.0.1:6379", "echo");
    let server_thread = server.start();
    let client = Ping::new("echo", "127.0.0.1:6379", 20, 10);
    let client_thread = client.start();
    tokio::join!(server_thread, client_thread);

    let config = ProxyConfig::new("test_config.yaml".to_string());
    println!("proxy config : {:?}", config);
}

