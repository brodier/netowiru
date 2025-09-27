use std::fs;
use std::collections::HashMap;
use yaml_rust2::{Yaml, YamlLoader};

#[derive(Debug)]
pub struct ProxyConfig {
    pub incomming: IncommingConfig, // frontend map from service name to address
    pub outgoing: HashMap<String, OutgoingConfig>,   // backend map from local address to (service name, 
}

#[derive(Debug)]
pub struct IncommingConfig {
    pub address: String,
    pub services: HashMap<String, String>, // map from service name to address
}

#[derive(Debug)]
pub struct OutgoingConfig {
    pub local_address: String,
    pub proxy_address: String,
}
// Load hashmap from config file with yaml format
impl ProxyConfig {
    pub fn load(config_file:String) -> Self {
        let yaml_config = fs::read_to_string(config_file).unwrap();        
        let yaml_config = YamlLoader::load_from_str(&yaml_config).unwrap();
        let yaml_config = &yaml_config[0];
        let incomming_config = &yaml_config["incomming"];
        let address = incomming_config["address"].as_str().unwrap().to_string();
        let services = Self::load_map(&incomming_config["services"]);
        let incomming = IncommingConfig { address, services };
        let mut outgoing = HashMap::new();
        let outgoing_config = yaml_config["outgoing"].as_hash().unwrap();
        for (key, value) in outgoing_config {
            let service_name = key.as_str().unwrap().to_string();
            let local_address = value["local_address"].as_str().unwrap().to_string();
            let proxy_address = value["proxy_address"].as_str().unwrap().to_string();
            outgoing.insert(service_name, OutgoingConfig { local_address, proxy_address });
        }
        ProxyConfig {incomming, outgoing,}
    }

    fn load_map(yaml_map: &Yaml) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (key, value) in yaml_map.as_hash().unwrap() {
            let key = key.as_str().unwrap().to_string();
            let value = value.as_str().unwrap().to_string();
            map.insert(key, value);
        }
        map
    }

}
