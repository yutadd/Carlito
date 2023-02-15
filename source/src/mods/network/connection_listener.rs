use super::super::config::config::YAML;
use super::connection;
use super::connection::CONNECTION_LIST;
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;

pub fn run() {
    let bind_target;
    unsafe {
        bind_target = format!(
            "{}:{}",
            YAML["network"]["bind-addr"].as_str().unwrap(),
            YAML["network"]["bind-port"].as_i64().unwrap()
        );
    }
    let listener = TcpListener::bind(bind_target).expect("Error: Failed to bind");
    println!("Listening...");
    for streams in listener.incoming() {
        println!("connection incoming!");
        let streams = streams.unwrap();
        unsafe {
            let user = connection::init(Arc::new(streams));
            let mut user2 = user.clone();
            CONNECTION_LIST.push(user);
            thread::spawn(move || user2.read_thread());
        }
    }
}
