use super::super::certification::key_agent;
use super::super::certification::sign_util;
use once_cell::sync::Lazy;
use std::io::{BufRead, Write};
use std::sync::Mutex;
use std::{io::BufReader, net::TcpStream};
use std::{sync::Arc, thread};
static COUNT: Lazy<Mutex<u16>> = Lazy::new(|| Mutex::new(0));
pub static mut UNTRUSTED_USERS: Lazy<Vec<User>> = Lazy::new(|| Vec::new());
pub static mut TRUSTED_USERS: Lazy<Vec<User>> = Lazy::new(|| Vec::new());
pub struct User {
    pub isok: bool,
    pub id: u16,
    pub user: Arc<TcpStream>,
    pub is_inbound: bool,
}
impl User {
    pub fn read_thread(&self) {
        let stream2 = Arc::clone(&self.user);
        let id = self.id;
        thread::spawn(move || {
            let mut reader = BufReader::new(&*stream2);
            println!("");
            loop {
                let mut line = String::new();
                let bytes = reader.read_line(&mut line).unwrap();
                if bytes == 0 {
                    println!("接続終了");
                    let mut remove = 0;
                    let _aru = false;
                    unsafe {
                        for idx in 0..UNTRUSTED_USERS.len() {
                            if UNTRUSTED_USERS[idx].id == id {
                                remove = idx;
                            }
                        }
                        UNTRUSTED_USERS.remove(remove);
                    }
                    break;
                } else {
                    let json_obj = json::parse(&line).unwrap();
                    if json_obj["type"].eq("hello") {
                        println!("received pubk is {}", json_obj["args"]["pubk"]);
                    }
                }
                println!("{}", line);
            }
        });
    }
    pub fn write(&self, context: String) {
        (&*self.user).write_all(context.as_bytes()).unwrap();
        (&*self.user).flush().unwrap();
    }
}
pub fn init(stream: Arc<TcpStream>, is_inbound: bool) -> User {
    *COUNT.lock().unwrap() += 1;
    if !is_inbound {
        unsafe {
            let key = key_agent::SECRET[0].x_only_public_key(&sign_util::SECP).0;
            (&*stream)
                .write_all(
                    format!(
                        "{{\"type\":\"hello\",\"args\":{{\"pubk\":\"{}\"}}}}\r\n",
                        key.to_string()
                    )
                    .as_bytes(),
                )
                .unwrap();
            (&*stream).flush().unwrap();
        }
    }
    User {
        user: stream,
        id: *COUNT.lock().unwrap(),
        isok: true,
        is_inbound: is_inbound,
    }
}
#[test]
fn parsing_json() {
    let json_str = format!(
        "{{\"type\":\"hello\",\"args\":{{\"secret\":\"{}\"}}}}\r\n",
        "key_dummy"
    );
    let json_obj = json::parse(&json_str).unwrap();
    assert_eq!(json_obj["type"], "hello");
    assert_eq!(json_obj["args"]["secret"], "key_dummy");
}
