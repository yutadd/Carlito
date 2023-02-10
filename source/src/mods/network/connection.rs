use super::super::certification::key_agent;
use super::super::certification::sign_util;
use once_cell::sync::Lazy;
use rand::prelude::*;
use secp256k1::ecdsa::Signature;
use secp256k1::PublicKey;
use secp256k1::XOnlyPublicKey;
use std::io::{BufRead, Write};
use std::str::FromStr;
use std::sync::Mutex;
use std::{io::BufReader, net::TcpStream};
use std::{sync::Arc, thread};
static COUNT: Lazy<Mutex<u16>> = Lazy::new(|| Mutex::new(0));
pub static mut UNTRUSTED_USERS: Lazy<Vec<Connection>> = Lazy::new(|| Vec::new());
pub static mut TRUSTED_USERS: Lazy<Vec<Connection>> = Lazy::new(|| Vec::new());
pub struct Connection {
    pub isok: bool,
    pub id: u16,
    pub user: Arc<TcpStream>,
    pub pubk: Option<PublicKey>,
    pub is_inbound: bool,
    pub nonce: Option<String>,
}
impl Connection {
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
                        UNTRUSTED_USERS.remove(get_idx(id));
                    }
                    break;
                } else {
                    let json_obj = json::parse(&line).unwrap();
                    if json_obj["type"].eq("hello") {
                        println!("received pubk is {}", json_obj["args"]["pubk"]);
                        unsafe {
                            UNTRUSTED_USERS[get_idx(id)].pubk = Option::Some(
                                PublicKey::from_str(json_obj["args"]["pubk"].as_str().unwrap())
                                    .unwrap()
                                    .clone(),
                            );
                            let mut rng = rand::thread_rng();
                            let generated_rand = rng.next_u32();
                            UNTRUSTED_USERS[get_idx(id)].write(format!(
                                "{{\"type\":\"req_sign\",\"args\":{{\"nonce\":\"{}\"}}}}\r\n",
                                generated_rand
                            ));
                            UNTRUSTED_USERS[get_idx(id)].nonce =
                                Option::Some(format!("{}", generated_rand));
                        }
                    } else if json_obj["type"].eq("req_sign") {
                        unsafe {
                            let sign = sign_util::create_sign(
                                json_obj["args"]["nonce"].as_str().unwrap().to_string(),
                                key_agent::SECRET[0],
                            );
                            UNTRUSTED_USERS[get_idx(id)].write(format!(
                                "{{\"type\":\"signed\",\"args\":{{\"sign\":\"{}\"}}}}\r\n",
                                sign.to_string()
                            ));
                        }
                    } else if json_obj["type"].eq("signed") {
                        unsafe {
                            let verify_result = sign_util::verify_sign(
                                UNTRUSTED_USERS[get_idx(id)].nonce.clone().unwrap(),
                                json_obj["args"]["sign"].as_str().unwrap().to_string(),
                                UNTRUSTED_USERS[get_idx(id)].pubk.unwrap(),
                            );
                            println!("verify result:{}", verify_result)
                        }
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
fn get_idx(id: u16) -> usize {
    unsafe {
        for idx in 0..UNTRUSTED_USERS.len() {
            if UNTRUSTED_USERS[idx].id == id {
                return idx;
            }
        }
        UNTRUSTED_USERS.len()
    }
}
pub fn init(stream: Arc<TcpStream>, is_inbound: bool) -> Connection {
    *COUNT.lock().unwrap() += 1;
    if !is_inbound {
        unsafe {
            let key = key_agent::SECRET[0]
                .public_key(&sign_util::SECP)
                .to_string();
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
    Connection {
        user: stream,
        id: *COUNT.lock().unwrap(),
        isok: true,
        pubk: Option::None,
        is_inbound: is_inbound,
        nonce: Option::None,
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
