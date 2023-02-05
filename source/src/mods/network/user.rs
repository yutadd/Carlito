use std::io::{Write, Read, BufRead};
use std::{
    sync::Arc,
    thread
};
use std::{
    io::BufReader,
    net::{ TcpStream},
    prelude::*,
    task,
};
pub struct User{
    pub user:Arc<TcpStream>
}
impl User{
    pub fn write(&self,context:String){
        (&*self.user).write_all(context.as_bytes());
        (&*self.user).flush();
    }
}
pub fn init(stream:Arc<TcpStream>)->User{
    let mut stream2=Arc::clone(&stream);
    thread::spawn(move ||{
        let mut reader = BufReader::new(&*stream2);
        println!("");
        loop{
            let mut line=String::new();
            let bytes=reader.read_line(&mut line).unwrap();
            if bytes==0 {
                println!("接続終了");
                break;
            }
            println!("{}",line);
        }
    });
    User{user:stream}
}