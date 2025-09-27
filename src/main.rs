use netowiru::tools::pingpong::Server;
use netowiru::tools::pingpong::Ping;
use netowiru::proxy::config::ProxyConfig;

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    sample_proxy().await;
}

async fn sample_proxy() {
    // Bind the listener to the address
    let server = Server::new("127.0.0.1:6379", "echo");
    let server_thread = server.start();
    let client = Ping::new("echo", "127.0.0.1:6379", 20, 10);
    let client_thread = client.start();
    tokio::join!(server_thread, client_thread);

    let config = ProxyConfig::load("test_config.yaml".to_string());
    println!("proxy config : {:?}", config);
    // Build proxy from config mapping 8003 to echo service on 8001

    let proxy = netowiru::proxy::proxy::Proxy::build(config);
    let mut handles = proxy.start();
    let server = Server::new("127.0.0.1:8001", "echo");
    let server_thread = server.start();
    let client = Ping::new("echo", "127.0.0.1:8003", 20, 10);
    let client_thread = client.start();
    tokio::join!(server_thread, client_thread);
    println!("echo traffic through service1 test completed");
    handles.clear();
}