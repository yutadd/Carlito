use super::super::certification::key_agent;
use super::super::certification::sign_util;
use crate::mods::block::block;
use crate::mods::block::block::check;
use crate::mods::block::block::BLOCKCHAIN;
use crate::mods::certification::sign_util::TRUSTED_KEY;
use crate::mods::config::config;
use crate::mods::console::output::{eprintln, println, wprintln};
use crate::mods::PoA::blockchain_manager::get_previous_generator;
use crate::mods::PoA::blockchain_manager::set_previous_generator;
use async_std::sync;
use once_cell::sync::Lazy;
use rand::prelude::*;
use secp256k1::hashes::sha256;
use secp256k1::Message;
use secp256k1::PublicKey;
use std::io::{BufRead, Write};
use std::net::Shutdown;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::{io::BufReader, net::TcpStream};
static COUNT: Lazy<Mutex<u16>> = Lazy::new(|| Mutex::new(0));
pub static mut CONNECTION_LIST: Lazy<Vec<Connection>> = Lazy::new(|| Vec::new());
pub static IS_BLOCKED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

pub struct Connection {
    pub is_connected: bool, //処理中などにノードが使用不可になったことを判定できるようにisokは必要
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
            is_connected: self.is_connected.clone(),
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
            if bytes == 0 || !self.is_connected {
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
                        json_obj["args"]["pubk"].to_string()
                    ));
                    let mut is_already_connected = false;
                    unsafe {
                        for c in CONNECTION_LIST.iter() {
                            if c.pubk.is_some() {
                                if c.pubk
                                    .unwrap()
                                    .to_string()
                                    .eq(&json_obj["args"]["pubk"].to_string())
                                {
                                    self.stream.shutdown(Shutdown::Both).unwrap();
                                    CONNECTION_LIST[get_idx(self.id)].is_connected = false;
                                    is_already_connected = true;
                                    break;
                                }
                            }
                        }
                        if !is_already_connected {
                            CONNECTION_LIST[get_idx(self.id)].pubk = Option::Some(
                                PublicKey::from_str(json_obj["args"]["pubk"].as_str().unwrap())
                                    .unwrap()
                                    .clone(),
                            );

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
                                wprintln(format!(
                                    "[connection]connection not trusted for wrong pubk."
                                ));
                            }
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
                        wprintln(format!("[connection]failed to verify this connection"));
                    }
                } else if json_obj["type"].eq("get_latest") {
                    unsafe {
                        if BLOCKCHAIN.read().unwrap().len() > 0 {
                            self.write(format!(
                                "{{\"type\":\"block\",\"args\":{{\"block\":{}}}}}\r\n",
                                BLOCKCHAIN.read().unwrap()[BLOCKCHAIN.read().unwrap().len() - 1]
                                    .dump()
                            ));
                        } else {
                            self.write("{\"type\":\"no_block\"}\r\n".to_string());
                        }
                    }
                } else if json_obj["type"].eq("block") {
                    unsafe {
                        if !*IS_BLOCKED.lock().unwrap() {
                            *IS_BLOCKED.lock().unwrap() = true;

                            println(format!(
                                "[connection]BLOCKCHAIN_LEN:{}",
                                BLOCKCHAIN.read().unwrap().len()
                            ));
                            println(format!(
                                "[connection]received height:{}",
                                json_obj["args"]["block"]["height"].as_usize().unwrap()
                            ));
                            if json_obj["args"]["block"]["height"].as_usize().unwrap()
                                > BLOCKCHAIN.read().unwrap().len()
                            {
                                let previous = match BLOCKCHAIN.read().unwrap().len() > 0 {
                                    true => Message::from_hashed_data::<sha256::Hash>(
                                        BLOCKCHAIN.read().unwrap()
                                            [BLOCKCHAIN.read().unwrap().len() - 1]
                                            .dump()
                                            .as_bytes(),
                                    )
                                    .to_string(),
                                    false => block::GENESIS_BLOCK_HASH.to_string(),
                                };
                                if check(json_obj["args"]["block"].clone(), previous) {
                                    println(format!(
                                "[connection]Received block is correct and taller than my block"
                            ));
                                    for i in 0..TRUSTED_KEY.len() {
                                        println(format!(
                                            "[connection]compare trusted_key:{} and {}",
                                            TRUSTED_KEY.get(&(i as isize)).unwrap(),
                                            &json_obj["args"]["block"]["author"].to_string()
                                        ));
                                        if TRUSTED_KEY
                                            .get(&(i as isize))
                                            .unwrap()
                                            .eq(&json_obj["args"]["block"]["author"].to_string())
                                        {
                                            set_previous_generator(i as isize);
                                            break;
                                        }
                                    }
                                    println(format!(
                                        "[connection]previous_generator:{}",
                                        get_previous_generator()
                                    ));
                                    BLOCKCHAIN
                                        .write()
                                        .unwrap()
                                        .push(json_obj["args"]["block"].clone());
                                    println("[connection]New block pushed to my blockchain");
                                } else if check(json_obj["args"]["block"].clone(), "*".to_string())
                                {
                                    println(format!(
                                        "[connection]Received block is correct but it taller than my blockchain for 2 or more."
                                    ));
                                } else {
                                    wprintln(format!("[connection]Received block is taller than my block but not correct"));
                                }
                            } else {
                                wprintln(format!(
                                    "[connection]Received block is not taller than my block."
                                ));
                            }
                            *IS_BLOCKED.lock().unwrap() = false;
                        } else {
                            println("[connection]_blocked thread.")
                        }
                    }
                    unsafe {
                        CONNECTION_LIST[get_idx(self.id)].is_latest = true;
                    }
                } else if json_obj["type"].eq("no_block") {
                    unsafe {
                        CONNECTION_LIST[get_idx(self.id)].is_latest = true;
                    }
                } else {
                    wprintln(format!("[connection]connection received unknown command"));
                }
            }
        }
    }
    pub fn write(&self, context: String) {
        if self.is_connected {
            loop {
                if !*IS_BLOCKED.lock().unwrap() {
                    break;
                }
                println("[connection]blocking thread");
                thread::sleep(Duration::from_secs(1));
            }
            (&*self.stream).write_all(context.as_bytes()).unwrap();
            (&*self.stream).flush().unwrap();
        }
    }
}
/*pub fn is_all_connected() -> bool {
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
                    wprintln(format!("[connection]there are some not connected node"));
                    wprintln(format!("[connection]not connected node:{}", tk));
                    return false;
                }
            }
        }
        return true;
    }
}*/
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
        is_connected: true,
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
