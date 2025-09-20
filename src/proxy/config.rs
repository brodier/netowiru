use std::fs;
use std::collections::HashMap;
use yaml_rust2::YamlLoader;

#[derive(Debug)]
pub struct ProxyServiceToAddress {
    local_address: String,
    remote_address: String,
}

#[derive(Debug)]
pub struct ProxyConfig {
    local_address: String,
    incomming: HashMap<String, String>, // frontend map from service name to address
    outgoing: HashMap<String, ProxyServiceToAddress>,   // backend map from local address to (service name, 
}

// Load hashmap from config file with yaml format
impl ProxyConfig {
    pub fn new(config_file:String) -> Self {

        
        let yaml_config = fs::read_to_string(config_file).unwrap();        
        let yaml_config = YamlLoader::load_from_str(&yaml_config).unwrap();
        let yaml_config = &yaml_config[0];
        let local_address = yaml_config["local_address"].as_str().unwrap().to_string();
        let mut incomming: HashMap<String, String> = HashMap::new();
        let mut outgoing: HashMap<String, ProxyServiceToAddress> = HashMap::new();
        for (service_name, address) in yaml_config["incomming"].as_hash().unwrap() {
            incomming.insert(service_name.as_str().unwrap().to_string(), address.as_str().unwrap().to_string());
        }
        for (service_name, proxy_addresses) in yaml_config["outgoing"].as_hash().unwrap() {
            let service_name = service_name.as_str().unwrap().to_string();
            let local_address = proxy_addresses["local_address"].as_str().unwrap().to_string();
            let remote_address = proxy_addresses["remote_address"].as_str().unwrap().to_string();
            outgoing.insert(service_name, ProxyServiceToAddress {local_address,remote_address,}); 
        }
        ProxyConfig {
            local_address,
            incomming,
            outgoing,}
    }

    pub fn get_local_address(&self) -> String {
        self.local_address.clone()
    }

    pub fn get_incomming(&self) -> HashMap<String, String> {
        self.incomming.clone()
    }

    pub fn get_outgoing(&self) -> &HashMap<String, ProxyServiceToAddress> {
        &self.outgoing
    }
}
