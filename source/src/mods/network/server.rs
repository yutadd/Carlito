use std::{
    sync::Arc,
    thread
};
use crate::mods::network::user;
use std::{
    io::BufReader,
    net::{TcpListener, TcpStream},
    prelude::*
};
pub fn run(){
    let listener = TcpListener::bind("0.0.0.0:7777").expect("Error: Failed to bind");
    println!("Listening...");
    for streams in listener.incoming() {
        println!("connection incoming!");
        let streams=streams.unwrap();
        user::init(Arc::new(streams)).write("Hello from rust!\n".to_string());
    }
}