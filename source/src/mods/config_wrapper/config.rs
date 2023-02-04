extern crate yaml_rust;
use yaml_rust::{YamlLoader};
use std::fs;
pub struct Elements{
    pub network:String,
    pub version:String,
}
impl Elements {
    pub fn new() -> Self {
        Self { network:"seed.yutadd.com".to_string() , version: "0.0.0.16".to_string() }
    }
}
pub fn init()->Elements{
    let s=fs::read_to_string("Config/config.yml").unwrap();
    let doc = YamlLoader::load_from_str(&s).unwrap();
    let doc=&doc[0];
    let elm=Elements{
        network:doc["network"].as_str().unwrap().to_string(),
        version:"0.0.0.17".to_string()
    };
    elm
    // Test
    //assert_eq!(doc["foo"][0].as_str().unwrap(), "list1");
}
#[test]
fn config_init(){
    println!("unsafe config getter:{}",init().network);
}