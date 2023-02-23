use super::super::config::config::YAML;
use super::connection;
use crate::mods::console::output::println;
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;

pub fn run() {
    let bind_target;
    bind_target = format!(
        "{}:{}",
        YAML.get().unwrap()["network"]["bind-addr"]
            .as_str()
            .unwrap(),
        YAML.get().unwrap()["network"]["bind-port"]
            .as_i64()
            .unwrap()
    );

    let listener = TcpListener::bind(bind_target).expect("Error: Failed to bind");
    println(format!("[connection_listener]Listening..."));
    for streams in listener.incoming() {
        println(format!("[connection_listener]connection incoming!"));
        let streams = streams.unwrap();
        let user = connection::init(Arc::new(streams));
        let mut user2 = user.clone();
        connection::STATS
            .write()
            .unwrap()
            .connection_list
            .push(user);
        thread::spawn(move || user2.read_thread());
    }
}
