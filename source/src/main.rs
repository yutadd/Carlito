use once_cell::sync::Lazy;
use mods::certification::key_agent;
use mods::certification::sign_util;
use mods::config_wrapper::config;
pub static VERSION: Lazy<String>=Lazy::new(|| String::from("0.0.0.16"));         //count of Release.BetaRelease.DevRelease.Commit
pub static NETWORK: Lazy<String>=Lazy::new(|| String::from("seed.yutadd.com")); //IP of dns seed
mod mods;
fn main() {
    println!("Hello, world!{}",VERSION.as_str());
    println!("inited:{:?}",config::init());
    key_agent::init();
    sign_util::init();
    println!("test_signeture");
    println!("{}",sign_util::is_host_trusted("f0cd5f5b47d983c4c5c173444e577bcffda3884f6f53b03cf5f97b5ed25d692f".to_string()));
    let sign=sign_util::create_sign("HelloWorld".to_string(), *key_agent::get_key(0).unwrap());
    println!("{}",sign);
    println!("{}",sign_util::verify_sign("HelloWorld".to_string(), sign.to_string(), key_agent::get_key(0).unwrap().public_key(&sign_util::SECP)));
    println!("{}",sign_util::verify_sign("HelloWorld01".to_string(), sign.to_string(), key_agent::get_key(0).unwrap().public_key(&sign_util::SECP)));
    println!("end_test_signeture");
    for i in 0..(key_agent::get_key_length()){
        println!("{}",key_agent::get_key(i).unwrap().display_secret());
    }

}
