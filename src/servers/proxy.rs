use std::collections::HashMap;

struct Proxy {
    name: String,
    frontend: String,
    backends: HashMap<String, String>,
}

impl Proxy {

    fn new(name: &str, frontend: &str, backends: HashMap<String,String>) -> Self {
        let backends = backends.clone();
        Proxy {
            name: name.to_string(),
            frontend: frontend.to_string(),
            backends,
        }
    }

    async fn start(&self) {
        // Start listening on the frontend address
        let listener = tokio::net::TcpListener::bind(&self.frontend).await.unwrap();
        eprintln!("Proxy {} listening on {}", self.name, self.frontend);

        loop {
            let (frontend_socket, _) = listener.accept().await.unwrap();
            // TODO : Upgrade tcp to websocket and retrieve service header to select backend
            let backend_addr = self.backends.get("default").unwrap().to_string();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(frontend_socket, &backend_addr).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }
    }

    async fn handle_connection(mut frontend: tokio::net::TcpStream, backend_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        // upgrade the connection 
        // Connect to the backend server
        let mut backend = tokio::net::TcpStream::connect(backend_addr).await?;

        // Split the frontend and backend sockets into read/write halves
        let (mut fr, mut fw) = frontend.split();
        let (mut br, mut bw) = backend.split();

        // Create tasks to forward data in both directions
        let client_to_server = tokio::io::copy(&mut fr, &mut bw);
        let server_to_client = tokio::io::copy(&mut br, &mut fw);

        // Run both tasks concurrently
        tokio::try_join!(client_to_server, server_to_client)?;

        Ok(())
    }   
}