use std::{collections::HashMap, io, sync::Arc};

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};

pub struct Client {
    pub name: String,
    pub address: String
}

impl Client {
    pub fn new(name: &str, address: &str,) -> Self {
        Client {
            name: name.to_string(),
            address: address.to_string(),
        }
    }

    pub async fn connect(&self, frontend: TcpStream) -> io::Result<(TcpStream,TcpStream)> {
            let backend = tokio::net::TcpStream::connect(&self.address).await?;
            Self::complete_connect(frontend, backend, &self.name).await;
    }

    async fn complete_connect(mut socket: TcpStream, stream: TcpStream, name: &str) -> io::Result<(TcpStream,TcpStream)> {
        let len:u64 = name.len() as u64;
        socket.write_u64(len).await?;
        socket.write_all(name.as_bytes()).await?;
        let mut buf = vec![0; (len + 8) as usize];
        if socket.read_u64().await? != len {
            println!("Service name length mismatch on connecting to remote proxy");
            return;
        }
        socket.read_exact(&mut buf[..len as usize]).await?;
        let response = String::from_utf8_lossy(&buf[..len as usize]);
        if response != name {
            println!("Unexpected reply on proxy connection {}, expecting {}", response, name);
            return;
        } else {
            // proxy connected to remote service
        match socket.read_u64().await {
            Ok(0) => {
                println!("Failed to read response length: on connecting to remote proxy");
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
                if response != name {
                    println!("Unexpected reply on proxy connection {}, expecting {}", response, name);
                    return;
                } else {
                    // proxy connected to remote service
                    let frontend = stream;
                    let backend = socket;
                    println!("Proxy connected to remote service {}", name);
                    join_connections(frontend, backend).await;
                }                        
            },
            Err(e) => println!("Failed to read response: {}", e),
        }
    }

}

pub struct Server {
    addr: String,
    incomming: HashMap<String, String>, // frontend map from service name to address
}

impl Server {
    pub fn new(addr: &str, incomming: &HashMap<String,String>) -> Self {
        Server {
            addr: addr.to_string(),
            incomming: incomming.clone(), 
        }
    }


    pub async fn start(with:Self) {
        tokio::spawn(async move {
            // Bind the listener to the address
            let server = Arc::new(with);
            let listener = TcpListener::bind(server.addr.clone()).await.unwrap();
            eprintln!("Server listening on port 6379");
            loop {
                // The second item contains the IP and port of the new connection.
                let (socket, _) = listener.accept().await.unwrap();
                let with_server = Arc::clone(&server);
                tokio::spawn(async move {
                    if let Ok((frontend, backend )) = Self::complete_connect(socket, with_server).await {
                        join_connections(frontend, backend).await;
                    }
                });
            }
        });
    }


    async fn complete_connect(mut socket: TcpStream, with_server:Arc<Self>) -> io::Result<(TcpStream,TcpStream)> {
        let service_name_len = socket.read_u64().await? as usize;
        if service_name_len == 0 {
            eprintln!("Connection closed by client");
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Connection closed by client"));
        }
        
        let mut buf = vec![0; service_name_len];
        let recieved_len =  socket.read_exact(&mut buf).await?;
        if recieved_len != service_name_len {
            eprintln!("Service name length mismatch: expected {}, got {}", service_name_len, recieved_len);
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Service name length mismatch"));
        } 
        let service_name = String::from_utf8_lossy(&buf).to_string();
        if !with_server.incomming.contains_key(&service_name) {
            eprintln!("Service name {} not found in incomming map", service_name);
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unknown service name"));
        }

        // 1. connect to the corresponding backend service
        let backend_address = with_server.incomming.get(&service_name).unwrap().to_string();
        let backend = tokio::net::TcpStream::connect(backend_address).await?;
        // 2. confirm to client that the proxy is connected to the backend service by echoing back the service name
        socket.write_u64(service_name_len as u64).await?;
        socket.write_all(service_name.as_bytes()).await?;
        eprintln!("Proxy connected to service {}", service_name);
        Ok((socket,backend))
    }   
    
}

async fn join_connections(frontend: TcpStream, backend: TcpStream) {
    tokio::spawn(async move {
        let (mut r1, mut w1) = frontend.into_split();
        let (mut r2, mut w2) = backend.into_split();
        let _client_to_server = tokio::io::copy(&mut r1, &mut w2);
        let _server_to_client = tokio::io::copy(&mut r2, &mut w1);
        // tokio::pin!(client_to_server);
        // tokio::pin!(server_to_client);
    });
}
