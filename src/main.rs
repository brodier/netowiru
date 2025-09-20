use netowiru::servers::echo::Server;
use netowiru::clients::ping::Ping;
#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let server = Server::new("127.0.0.1:6379");
    let server_thread = server.start();
    let client = Ping::new("127.0.0.1:6379", 5, 1000);
    let client_thread = client.start();
    tokio::join!(server_thread, client_thread);
}

