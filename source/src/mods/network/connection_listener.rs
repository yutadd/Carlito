use super::super::config_wrapper::config::YAML;
use super::connection;
use super::connection::UNTRUSTED_USERS;
use std::net::TcpListener;
use std::sync::Arc;

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
            user.read_thread();
            UNTRUSTED_USERS.push(user);
            for user in UNTRUSTED_USERS.iter() {
                //user.write("Hi from rust\r\n".to_string());
            }
        }
    }
}
