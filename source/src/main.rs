use mods::config_wrapper::config::Elements;
use once_cell::sync::OnceCell;
use mods::certification::key_agent;
use mods::certification::sign_util;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::thread;
use mods::config_wrapper::config;
use mods::network::server;
use mods::network::dns_seed;
use std::io::stdin;
pub static mut CONFIG: OnceCell<Mutex<config::Elements>>=OnceCell::new();         //count of Release.BetaRelease.DevRelease.Commit
mod mods;

fn main() {
    println!("Initializing...");
    key_agent::init();
    sign_util::init();
    unsafe{
        CONFIG=OnceCell::from(Mutex::new(config::init()));
    }
    thread::spawn(||{
        server::run();
        println!("thread-Inited");
        
    });
    dns_seed::init();
    println!("Inited");
    loop{
        let line=&mut String::new();
        stdin().read_line(line).unwrap();
        println!("your input:{}",line);
    }
}
fn get_config()->MutexGuard<'static, Elements, >{
    unsafe{
        CONFIG.get_or_init(||Mutex::new(config::Elements::new())).lock().unwrap()
    }
}
#[test]
fn main_access_config(){
    println!("Hello, world!{}",get_config().version);
}
