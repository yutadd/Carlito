extern crate yaml_rust;
use crate::mods::console::output::println;
use once_cell::sync::OnceCell;
use std::fs;
use yaml_rust::{Yaml, YamlLoader};

pub static YAML: OnceCell<Yaml> = OnceCell::new();
pub fn init() {
    YAML.set(
        YamlLoader::load_from_str(&fs::read_to_string("Config/config.yml").unwrap()).unwrap()[0]
            .clone(),
    )
    .unwrap();
}
#[test]
fn config_init() {
    println(format!(
        "[config]unsafe config getter:{}",
        YAML.get().unwrap()["network"]["domain"].as_str().unwrap()
    ));
}
