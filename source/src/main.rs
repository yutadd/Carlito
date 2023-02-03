use once_cell::sync::Lazy;
pub static VERSION: Lazy<String>=Lazy::new(|| String::from("0.0.0.12"));         //count of Release.BetaRelease.DevRelease.Commit
pub static NETWORK: Lazy<String>=Lazy::new(|| String::from("seed.yutadd.com")); //IP of dns seed
mod mods;
fn main() {
    println!("Hello, world!{}",VERSION.as_str());
    println!("inited:{:?}",mods::config_wrapper::config::init());
    mods::certification::key_agent::init();
}
