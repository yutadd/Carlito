use mods::certification::key_agent;
use mods::certification::sign_util;
use mods::network::dns_seed;
use mods::network::server;
use std::io::stdin;
use std::thread;
mod mods;

fn main() {
    println!("Initializing...");
    key_agent::init();
    sign_util::init();
    thread::spawn(|| {
        server::run();
        println!("thread-Inited");
    });
    dns_seed::init();
    println!("Inited");
    loop {
        let line = &mut String::new();
        stdin().read_line(line).unwrap();
        println!("your input:{}", line);
    }
}
