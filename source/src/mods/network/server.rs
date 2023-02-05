use std::sync::Arc;
use once_cell::sync::Lazy;
use super::user;
use std::net::TcpListener;
use super::user::User;
pub static mut users:Lazy<Vec<User>>=Lazy::new(||Vec::new());
pub fn run(){
    let listener = TcpListener::bind("0.0.0.0:7777").expect("Error: Failed to bind");
    println!("Listening...");
    for streams in listener.incoming() {
        println!("connection incoming!");
        let streams=streams.unwrap();
        unsafe{
            users.push(user::init(Arc::new(streams)));
            for user in users.iter(){
                user.write("Hi from rust\r\n".to_string());
            }
        }
        
    }
}