pub mod servers {
    pub mod proxy;
    pub use proxy as proxy_server;
    pub mod echo;
}

pub mod clients {
    pub mod ping;
    pub mod proxy;
    pub use proxy as proxy_client;
}