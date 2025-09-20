use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};

pub struct Server {
    addr: String,
    name: String,
}

impl Server {
    pub fn new(addr: &str, name: &str) -> Self {
        Server {
            addr: addr.to_string(),
            name: name.to_string(), 
        }
    }


    pub async fn start(&self) {
        let addr = self.addr.clone();
        let name = self.name.clone();
        tokio::spawn(async move {
            // Bind the listener to the address
            let listener = TcpListener::bind(addr).await.unwrap();
            eprintln!("Server listening on port 6379");
            loop {
                // The second item contains the IP and port of the new connection.
                let (socket, _) = listener.accept().await.unwrap();
                let name = name.clone();
                tokio::spawn(async move {
                    Self::process(socket, &name).await;
                });
            }
        });
    }


    async fn process(mut socket: TcpStream, name: &str) {
        loop {  
            match socket.read_u64().await {
                Ok(0)   => {
                    eprintln!("Connection closed by client");
                    break;
                },
                Ok(len) => {
                    eprintln!("Length: {}", len);
                    Self::process_message(len, &mut socket, name).await;
                },
                Err(e) => {
                    eprintln!("Connection closed by client");
                    break;
                }
            }
        }
    }   
    
    async fn process_message(len: u64, socket: &mut TcpStream, name: &str) {
        let mut buf = vec![0; len as usize];
        match socket.read_exact(&mut buf).await {
            Ok(0)   => {
                eprintln!("Not received any data ! Connection closed by client (len expected: {})", len);
                return;
            },
            Ok(n) => {
                assert_eq!(n as u64, len);
                eprintln!("Msg recieved : {}", String::from_utf8_lossy(&buf));
            },
            Err(e) => {
                eprintln!("Server Failed to read length: {}", e);
            }
        }        
        let msg = format!("Echoing back {} bytes from {}\n", len, name);
        let len:u64 = msg.len() as u64;
        if let Err(e) = socket.write_u64(len).await {
            eprintln!("Failed to send echo length: {}", e);
            return;
        }
        if let Err(e) = socket.write_all(msg.as_bytes()).await {
            eprintln!("Failed to send echo: {}", e);
            return;
        }
    }
}