use std::sync::Arc;
use super::user;
use std::net::TcpListener;
use super::user::USERS;

pub fn run(){
    let listener = TcpListener::bind("0.0.0.0:7777").expect("Error: Failed to bind");
    println!("Listening...");
    for streams in listener.incoming() {
        println!("connection incoming!");
        let streams=streams.unwrap();
        unsafe{
            let user=user::init(Arc::new(streams));
            user.read_thread();
            USERS.push(user);
            for user in USERS.iter(){
                user.write("Hi from rust\r\n".to_string());
            }
        }
        
    }
}