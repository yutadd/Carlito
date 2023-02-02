use once_cell::sync::Lazy;
static VERSION: Lazy<String>=Lazy::new(|| String::from("0.0.0.10"));         //count of Release.BetaRelease.DevRelease.Commit
static NETWORK: Lazy<String>=Lazy::new(|| String::from("seed.yutadd.com")); //IP of dns seed
mod mods;
fn main() {
    println!("Hello, world!{}",VERSION.as_str());
    println!("inited:{:?}",mods::config_wrapper::config::init());
    mods::certification::key_manager::init();
}
