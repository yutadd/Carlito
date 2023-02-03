use once_cell::sync::Lazy;
use mods::certification::key_agent;
use mods::config_wrapper::config;
pub static VERSION: Lazy<String>=Lazy::new(|| String::from("0.0.0.13"));         //count of Release.BetaRelease.DevRelease.Commit
pub static NETWORK: Lazy<String>=Lazy::new(|| String::from("seed.yutadd.com")); //IP of dns seed
mod mods;
fn main() {
    println!("Hello, world!{}",VERSION.as_str());
    println!("inited:{:?}",config::init());
    key_agent::init();
    for i in 0..(key_agent::get_key_length()){
        println!("{}",key_agent::get_key(i).unwrap().display_secret());
    }
}
