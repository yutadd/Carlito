extern crate yaml_rust;

use once_cell::sync::OnceCell;
use std::fs;
use yaml_rust::{Yaml, YamlLoader};

pub static YAML: OnceCell<Yaml> = OnceCell::new();
pub fn init() {
    YAML.get_or_init(|| {
        YamlLoader::load_from_str(&fs::read_to_string("Config/config.yml").unwrap()).unwrap()[0]
            .clone()
    });
}
#[test]
fn config_init() {
    use crate::mods::console::output::println;
    init();
    println(format!(
        "[config]config getter:{}",
        YAML.get().unwrap()["network"]["domain"].as_str().unwrap()
    ));
}
