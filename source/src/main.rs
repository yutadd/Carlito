use mods::config_wrapper::config::Elements;
use once_cell::sync::OnceCell;
use mods::certification::key_agent;
use mods::certification::sign_util;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::thread;
use mods::config_wrapper::config;
use mods::network::server;
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
        println!("Inited");
        
    });
    loop{
        let mut line=&mut String::new();
        stdin().read_line(line);
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
