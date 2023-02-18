use crate::mods::block::block::check;
use crate::mods::block::block::BLOCKCHAIN;
use crate::mods::config::config;
use crate::mods::PoA::blockchain_manager::PREVIOUS_GENERATOR;

use super::super::certification::key_agent;
use super::super::certification::sign_util;
use crate::mods::console::output::{eprintln, println};
use once_cell::sync::Lazy;
use rand::prelude::*;
use secp256k1::PublicKey;
use std::io::{BufRead, Write};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::{io::BufReader, net::TcpStream};
static COUNT: Lazy<Mutex<u16>> = Lazy::new(|| Mutex::new(0));
pub static mut CONNECTION_LIST: Lazy<Vec<Connection>> = Lazy::new(|| Vec::new());
pub struct Connection {
    pub isok: bool, //処理中などにノードが使用不可になったことを判定できるようにisokは必要
    pub id: u16,
    pub stream: Arc<TcpStream>,
    pub is_trusted: bool,
    pub pubk: Option<PublicKey>,
    pub nonce: Option<String>,
    pub is_latest: bool,
}
impl Clone for Connection {
    fn clone(&self) -> Connection {
        Connection {
            isok: self.isok.clone(),
            id: self.id,
            stream: self.stream.clone(),
            is_trusted: self.is_trusted.clone(),
            pubk: self.pubk,
            nonce: (&self.nonce).clone(),
            is_latest: self.is_latest.clone(),
        }
    }
}
impl Connection {
    pub fn read_thread(&self) {
        let mut reader = BufReader::new(&*self.stream);
        loop {
            let mut line = String::new();
            let bytes = reader.read_line(&mut line).unwrap();
            if bytes == 0 {
                println(format!("[connection]接続終了"));
                unsafe {
                    CONNECTION_LIST.remove(get_idx(self.id));
                }
                break;
            } else {
                println(format!("[connection]read_line{}", line));
                let json_obj = json::parse(&line).unwrap();
                if json_obj["type"].eq("hello") {
                    println(format!(
                        "[connection]received pubk is {}",
                        json_obj["args"]["pubk"]
                    ));
                    unsafe {
                        CONNECTION_LIST[get_idx(self.id)].pubk = Option::Some(
                            PublicKey::from_str(json_obj["args"]["pubk"].as_str().unwrap())
                                .unwrap()
                                .clone(),
                        );
                    }
                    unsafe {
                        if sign_util::is_host_trusted(
                            CONNECTION_LIST[get_idx(self.id)].pubk.unwrap().to_string(),
                        ) {
                            let mut rng = rand::thread_rng();
                            let generated_rand = rng.next_u32();
                            self.write(format!(
                                "{{\"type\":\"req_sign\",\"args\":{{\"nonce\":\"{}\"}}}}\r\n",
                                generated_rand
                            ));

                            CONNECTION_LIST[get_idx(self.id)].nonce =
                                Option::Some(format!("{}", generated_rand));
                        } else {
                            println(format!(
                                "[connection]connection dropped out for wrong pubk."
                            ));
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
                        println(format!("[connection]sign was sent"));
                    }
                } else if json_obj["type"].eq("signed") {
                    let verify_result;
                    unsafe {
                        verify_result = sign_util::verify_sign(
                            CONNECTION_LIST[get_idx(self.id)].nonce.clone().unwrap(),
                            json_obj["args"]["sign"].as_str().unwrap().to_string(),
                            CONNECTION_LIST[get_idx(self.id)].pubk.unwrap(),
                        );
                    }
                    if verify_result {
                        println(format!("[connection]verifying connection success"));
                        unsafe {
                            CONNECTION_LIST[get_idx(self.id)].is_trusted = true;
                        }
                        unsafe {
                            println(format!(
                                "[connection]is trusted:{}",
                                CONNECTION_LIST[get_idx(self.id)].is_trusted.to_string()
                            ));
                            CONNECTION_LIST[get_idx(self.id)]
                                .write("{\"type\":\"get_latest\"}\r\n".to_string());
                        }

                    //すべてのノードに最新のブロックを問い合わせて、最新状態に同期する。
                    } else {
                        println(format!("[connection]failed to verify this connection"));
                    }
                } else if json_obj["type"].eq("get_latest") {
                    unsafe {
                        if BLOCKCHAIN.len() > 0 {
                            self.write(format!(
                                "{{\"type\":\"block\",\"args\":{{\"block\":{}}}}}\r\n",
                                BLOCKCHAIN[BLOCKCHAIN.len() - 1].dump()
                            ));
                        }
                    }
                } else if json_obj["type"].eq("block") {
                    unsafe {
                        if BLOCKCHAIN.len() > 0 {
                            if json_obj["args"]["block"]["height"].as_usize().unwrap()
                                > BLOCKCHAIN[BLOCKCHAIN.len() - 1]["height"]
                                    .as_usize()
                                    .unwrap()
                            {
                                if check(json_obj["args"]["block"].clone(), "*".to_string()) {
                                    println(format!(
                                "[connection]Received block is correct and taller than my block"
                            ));
                                } else {
                                    eprintln(format!("[connection]Received block is taller than my block but not correct"));
                                }
                            } else {
                                eprintln(format!(
                                    "[connection]Received block is not taller than my block."
                                ));
                            }
                        }
                    }
                    unsafe {
                        CONNECTION_LIST[get_idx(self.id)].is_latest = true;
                    }
                } else {
                    println(format!("[connection]connection received unknown command"));
                }
            }
        }
    }
    pub fn write(&self, context: String) {
        (&*self.stream).write_all(context.as_bytes()).unwrap();
        (&*self.stream).flush().unwrap();
    }
}

pub fn is_all_connected() -> bool {
    unsafe {
        for tk in sign_util::TRUSTED_KEY.values() {
            if !tk.eq(config::YAML["docker"]["own-pubk"].as_str().unwrap()) {
                let mut aru = false;
                for c in CONNECTION_LIST.iter() {
                    if c.is_trusted {
                        if c.pubk.unwrap().to_string().eq(tk) {
                            aru = true;
                            break;
                        }
                    }
                }
                if !aru {
                    println(format!("[connection]there is not connected node"));
                    println(format!("[connection]not connected node:{}", tk));
                    return false;
                } else {
                    println(format!("[connection]ok, already connected"));
                }
            }
        }
        return true;
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

        println(format!(
            "[connection]secret:{}\n[connection]sent pubk:{}",
            key_agent::SECRET[0].display_secret(),
            key
        ))
    }
    Connection {
        id: *COUNT.lock().unwrap(),
        isok: true,
        stream: stream,
        is_trusted: false,
        pubk: Option::None,
        nonce: Option::None,
        is_latest: false,
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
