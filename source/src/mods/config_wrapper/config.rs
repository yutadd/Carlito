extern crate yaml_rust;
use once_cell::sync::Lazy;
use std::fs;
use yaml_rust::{Yaml, YamlLoader};

pub static mut YAML: Lazy<Yaml> = Lazy::new(|| {
    YamlLoader::load_from_str(&fs::read_to_string("Config/config.yml").unwrap()).unwrap()[0].clone()
});

#[test]
fn config_init() {
    unsafe {
        println!(
            "unsafe config getter:{}",
            YAML["network"]["domain"].as_str().unwrap()
        );
    }
}
