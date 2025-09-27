use std::{collections::HashMap, io, sync::Arc};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, task::JoinHandle};

use crate::proxy::config::ProxyConfig;

pub struct Client {
    pub name: String,
    pub local_address: String,
    pub proxy_address: String
}

impl Client {
    pub fn new(name: &str, local_address: &str, proxy_address: &str) -> Self {
        Client {
            name: name.to_string(),
            local_address: local_address.to_string(),
            proxy_address: proxy_address.to_string(),
        }
    }

    // TODO accept outgoing connections from clients and join them to the backend connections
    pub fn start(with: &Arc<Self>) -> JoinHandle<()> {
        let client = Arc::clone(with);
        tokio::spawn(async move {
            loop {
                // The second item contains the IP and port of the new connection.
                let listener = TcpListener::bind(&client.local_address).await.unwrap();
                let (socket, _) = listener.accept().await.unwrap();
                let with_client = Arc::clone(&client);
                tokio::spawn(async move {
                    if let Ok((frontend, backend )) = with_client.connect(socket).await {
                        join_connections(frontend, backend).await;
                    }
                });
            }
        })
    }   

    // Complete outgoing connection to the proxy server
    pub async fn connect(&self, frontend: TcpStream) -> io::Result<(TcpStream,TcpStream)> {
        let mut backend = tokio::net::TcpStream::connect(&self.proxy_address).await?;
        let len:u64 = self.name.len() as u64;
        backend.write_u64(len).await?;
        backend.write_all(self.name.as_bytes()).await?;
        let mut buf = vec![0; (len + 8) as usize];
        if backend.read_u64().await? != len {
            println!("Service name length mismatch on connecting to remote proxy");
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Service name length mismatch"));
        }
        backend.read_exact(&mut buf[..len as usize]).await?;
        let response = String::from_utf8_lossy(&buf[..len as usize]);
        if response != self.name {
            println!("Unexpected reply on proxy connection {}, expecting {}", response, self.name);
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Service name length mismatch"));
        } 
        println!("Proxy connected to service {}", self.name);
        Ok((frontend,backend))
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

    // Accepting incomming connections from proxy clients
    pub fn start(with: &Arc<Self>) -> JoinHandle<()> {
        let server = Arc::clone(with);
        tokio::spawn(async move {
            // Bind the listener to the address
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
        })
    }

    // Complete incoming connection from the proxy client
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




async fn join_connections(mut frontend: TcpStream,mut backend: TcpStream) {
    tokio::spawn(async move {
        match tokio::io::copy_bidirectional(&mut frontend,&mut backend).await  {
            Ok((a,b)) => println!("send {} bytes, recieved {} bytes", a, b),
            Err(e) => eprintln!("Error in connection: {}", e),
        }
    });
}


pub struct Proxy {
    server: Arc<Server>,
    clients: Vec<Arc<Client>>,
}

impl Proxy {
    pub fn build(config: ProxyConfig) -> Self {
        let server = Server::new(&config.incomming.address, &config.incomming.services);
        let mut clients = Vec::new();
        for (service_name, outgoing) in config.outgoing {
            clients.push(Arc::new(Client::new(&service_name, &outgoing.local_address, &outgoing.proxy_address)));
        }
        Proxy { server: Arc::new(server), clients }
    }

    pub fn start(&self) -> Vec<JoinHandle<()>> {
        let mut handles = Vec::new();
        handles.push(Server::start(&self.server));
        for client in self.clients.iter() {
            handles.push(Client::start(client));
        }
        handles
    }

}