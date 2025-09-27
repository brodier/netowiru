use netowiru::tools::pingpong::Server;
use netowiru::tools::pingpong::Ping;
use netowiru::proxy::config::ProxyConfig;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    /// The pattern to look for
    cmd: String,
    /// The path to the file to read
    #[arg(default_value = "app.yml")]
    conf: PathBuf,
}


#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    println!("pattern: {:?}, path: {:?}", cli.cmd, cli.conf);
    let result = tokio::spawn(async move {  
        match cli.cmd.as_str() {
            "ping" => send_echo(cli.conf).await,
            "echo" => wait_echo(cli.conf).await,
            "proxy" => run_proxy(cli.conf).await,
            "sample" => sample_proxy(cli.conf).await,
            _ => println!("unknown command"),
        }; 
    });
    let _ = tokio::join!(result);
}

async fn send_echo(_conf: PathBuf) {
    let client = Ping::new("echo", "127.0.0.1:8003", 20, 10);
    client.start().await
}

async fn wait_echo(_conf: PathBuf) {
    Server::new("127.0.0.1:8001", "echo").start().await;
}

async fn run_proxy(conf: PathBuf) {
    let config = ProxyConfig::load(conf.to_str().unwrap());
    println!("proxy config : {:?}", config);
    // Build proxy from config mapping 8003 to echo service on 8001

    let proxy = netowiru::proxy::proxy::Proxy::build(config);
    proxy.start().await;
}

async fn sample_proxy(conf: PathBuf) {
    let _proxy_task = tokio::spawn(run_proxy(conf.clone()));
    let _server_task = tokio::spawn(wait_echo(conf.clone()));
    send_echo(conf).await;
}