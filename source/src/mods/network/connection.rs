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
pub static mut CONNECTION_LIST: Lazy<Vec<Connection>> = Lazy::new(|| Vec::new());
pub struct Connection {
    pub isok: bool,
    pub id: u16,
    pub stream: Arc<TcpStream>,
    pub is_trusted: bool,
    pub pubk: Option<PublicKey>,
    pub nonce: Option<String>,
}
impl Clone for Connection {
    fn clone(&self) -> Connection {
        Connection {
            isok: self.isok,
            id: self.id,
            stream: self.stream.clone(),
            is_trusted: self.is_trusted,
            pubk: self.pubk,
            nonce: (&self.nonce).clone(),
        }
    }
}
impl Connection {
    pub fn read_thread(&mut self) {
        let mut reader = BufReader::new(&*self.stream);
        loop {
            let mut line = String::new();
            let bytes = reader.read_line(&mut line).unwrap();
            if bytes == 0 {
                println!("接続終了");
                unsafe {
                    CONNECTION_LIST.remove(get_idx(self.id));
                }
                break;
            } else {
                let json_obj = json::parse(&line).unwrap();
                if json_obj["type"].eq("hello") {
                    println!("received pubk is {}", json_obj["args"]["pubk"]);
                    unsafe {
                        self.pubk = Option::Some(
                            PublicKey::from_str(json_obj["args"]["pubk"].as_str().unwrap())
                                .unwrap()
                                .clone(),
                        );
                        if sign_util::TRUSTED_KEY.contains(&self.pubk.unwrap().to_string()) {
                            let mut rng = rand::thread_rng();
                            let generated_rand = rng.next_u32();
                            self.write(format!(
                                "{{\"type\":\"req_sign\",\"args\":{{\"nonce\":\"{}\"}}}}\r\n",
                                generated_rand
                            ));
                            self.nonce = Option::Some(format!("{}", generated_rand));
                        } else {
                            println!("connection dropped out for wrong pubk.");
                        }
                    }
                } else if json_obj["type"].eq("req_sign") {
                    unsafe {
                        let sign = sign_util::create_sign(
                            json_obj["args"]["nonce"].as_str().unwrap().to_string(),
                            key_agent::SECRET[0],
                        );
                        self.write(format!(
                            "{{\"type\":\"signed\",\"args\":{{\"sign\":\"{}\"}}}}\r\n",
                            sign.to_string()
                        ));
                        println!("sign was sent");
                    }
                } else if json_obj["type"].eq("signed") {
                    unsafe {
                        let verify_result = sign_util::verify_sign(
                            self.nonce.clone().unwrap(),
                            json_obj["args"]["sign"].as_str().unwrap().to_string(),
                            self.pubk.unwrap(),
                        );

                        if verify_result {
                            println!("verifying connection success",);
                            self.is_trusted = true;
                        } else {
                            println!("failed to verify this connection");
                        }
                    }
                } else {
                    println!("connection received unknown command");
                }
            }
            println!("{}", line);
        }
    }
    pub fn write(&self, context: String) {
        (&*self.stream).write_all(context.as_bytes()).unwrap();
        (&*self.stream).flush().unwrap();
    }
}
fn get_idx(id: u16) -> usize {
    unsafe {
        for idx in 0..CONNECTION_LIST.len() {
            if CONNECTION_LIST[idx].id == id {
                return idx;
            }
        }
        CONNECTION_LIST.len()
    }
}
pub fn init(stream: Arc<TcpStream>) -> Connection {
    *COUNT.lock().unwrap() += 1;
    unsafe {
        let key = key_agent::SECRET[0]
            .public_key(&sign_util::SECP)
            .to_string();
        (&*stream)
            .write_all(
                format!(
                    "{{\"type\":\"hello\",\"args\":{{\"pubk\":\"{}\"}}}}\r\n",
                    key
                )
                .as_bytes(),
            )
            .unwrap();
        (&*stream).flush().unwrap();

        println!(
            "secret:{}\nsent pubk:{}",
            key_agent::SECRET[0].display_secret(),
            key
        )
    }
    Connection {
        id: *COUNT.lock().unwrap(),
        isok: true,
        stream: stream,
        is_trusted: false,
        pubk: Option::None,
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
