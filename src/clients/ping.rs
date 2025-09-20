use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Ping {
    pub name: String,
    pub address: String,
    pub count: u32,
    pub interval: u64,
}

impl Ping {
    pub fn new(name: &str, address: &str, count: u32, interval: u64) -> Self {
        Ping {
            name: name.to_string(),
            address: address.to_string(),
            count,
            interval,
        }
    }

    pub async fn start(&self) {
        for i in 0..self.count {
            match tokio::net::TcpStream::connect(&self.address).await {
                Ok(socket) => Self::process_ping(socket, i, &self.address, &self.name).await,
                Err(e) => println!("Test {} on {}: Failed to connect ({})", i + 1, self.address, e),
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(self.interval)).await;
        }
    }

    async fn process_ping(mut socket: tokio::net::TcpStream, seq: u32, address: &str, name: &str) {
        let msg = format!("Ping {} to {}\n", seq + 1, address);
        let len:u64 = msg.len() as u64;
        if let Err(e) = socket.write_u64(len).await {
            println!("Failed to send ping length: {}", e);
            return;
        }
        if let Err(e) = socket.write_all(msg.as_bytes()).await {
            println!("Failed to send ping: {}", e);
            return;
        }

        let mut buf = vec![0; 1024];
        match socket.read_u64().await {
            Ok(0) => {
                println!("Connection closed by server");
                return;
            }
            Ok(len) => {
                if len as usize > buf.len() {
                    buf.resize(len as usize, 0);
                }
            }
            Err(e) => {
                println!("Failed to read response length: {}", e);
                return;
            }
        }
        match socket.read(&mut buf).await {
            Ok(n) if n == 0 => println!("Connection closed by server"),
            Ok(n) => {
                let response = String::from_utf8_lossy(&buf[..n]);
                let exp_rep = format!("Echoing back {} bytes from {}\n", len, name);
                if response != exp_rep {
                    println!("Unexpected response: {}, expecting {}", response, exp_rep);
                } else {
                    println!("Test {} on {}: Success", seq + 1, name);
                }
            }
            Err(e) => println!("Failed to read response: {}", e),
        }
    }
}