use super::super::certification::key_agent;
use super::super::certification::sign_util;
use crate::mods::block::block;
use crate::mods::block::block::check;
use crate::mods::block::block::BLOCKCHAIN;
use crate::mods::certification::sign_util::TRUSTED_KEY;
use crate::mods::console::output::{println, wprintln};
use crate::mods::poa::blockchain_manager::get_previous_generator;
use crate::mods::poa::blockchain_manager::set_previous_generator;
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
use std::sync::RwLock;
use std::thread;
use std::time::Duration;
use std::{io::BufReader, net::TcpStream};
/**
 * 重複するノード・接続終了したノードの削除ループの実装
 * */
pub struct ConnectionStats {
    pub connection_list: Vec<Connection>,
    pub count: u16,
}
pub static STATS: RwLock<ConnectionStats> = {
    RwLock::new(ConnectionStats {
        connection_list: Vec::new(),
        count: 0,
    })
};
pub struct Connection {
    //クローン後も同じ値を参照するためにスマートポインタを使う必要がある機がする。
    pub is_connected: Arc<RwLock<bool>>, //処理中などにノードが使用不可になったことを判定できるようにisokは必要
    pub id: Arc<RwLock<u16>>,
    pub stream: Arc<TcpStream>,
    pub is_trusted: Arc<RwLock<bool>>,
    pub pubk: Arc<RwLock<Option<PublicKey>>>,
    pub nonce: Arc<RwLock<Option<String>>>,
    pub is_latest: Arc<RwLock<bool>>,
}
impl Clone for Connection {
    fn clone(&self) -> Connection {
        Connection {
            is_connected: self.is_connected.clone(),
            id: self.id.clone(),
            stream: self.stream.clone(),
            is_trusted: self.is_trusted.clone(),
            pubk: self.pubk.clone(),
            nonce: self.nonce.clone(),
            is_latest: self.is_latest.clone(),
        }
    }
}
impl Connection {
    pub fn read_thread(&mut self) {
        let binding = (&self.stream).clone();
        let mut reader = BufReader::new(&*binding);
        loop {
            let mut line = String::new();
            let bytes = match reader.read_line(&mut line) {
                Ok(o) => o,
                Err(e) => {
                    println(format!(
                        "[connection]error on reading input buffer:{}",
                        e.kind()
                    ));
                    break;
                }
            };
            if bytes == 0 || line.trim().len() == 0 {
                *self.is_connected.write().unwrap() = false;
                self.stream.shutdown(Shutdown::Both).unwrap();
                break;
            }
            println(format!("[connection]read_line{}", line));
            let json_obj = json::parse(&line).unwrap();
            if json_obj["type"].eq("hello") {
                println(format!(
                    "[connection]received pubk is {}",
                    json_obj["args"]["pubk"].to_string()
                ));
                *self.pubk.write().unwrap() = Option::Some(
                    PublicKey::from_str(json_obj["args"]["pubk"].as_str().unwrap())
                        .unwrap()
                        .clone(),
                );

                if sign_util::is_host_trusted(self.pubk.read().unwrap().unwrap().to_string()) {
                    let mut rng = rand::thread_rng();
                    let generated_rand = rng.next_u32();
                    self.write(format!(
                        "{{\"type\":\"req_sign\",\"args\":{{\"nonce\":\"{}\"}}}}\r\n",
                        generated_rand
                    ));

                    *self.nonce.write().unwrap() = Option::Some(format!("{}", generated_rand));
                } else {
                    wprintln(format!(
                        "[connection]connection not trusted for wrong pubk."
                    ));
                }
            } else if json_obj["type"].eq("req_sign") {
                let sign = sign_util::create_sign(
                    json_obj["args"]["nonce"].as_str().unwrap().to_string(),
                    *key_agent::SECRET.get().unwrap(),
                );
                self.write(format!(
                    "{{\"type\":\"signed\",\"args\":{{\"sign\":\"{}\"}}}}\r\n",
                    sign.to_string()
                ));
                println(format!("[connection]sign was sent"));
            } else if json_obj["type"].eq("signed") {
                let verify_result;
                verify_result = sign_util::verify_sign(
                    self.nonce.read().unwrap().clone().unwrap(),
                    json_obj["args"]["sign"].as_str().unwrap().to_string(),
                    self.pubk.read().unwrap().unwrap(),
                );

                if verify_result {
                    println(format!("[connection]verifying connection success"));
                    *self.is_trusted.write().unwrap() = true;
                    println(format!(
                        "[connection]is trusted:{}",
                        self.is_trusted.read().unwrap().to_string()
                    ));
                    self.write("{\"type\":\"get_latest\"}\r\n".to_string());

                //すべてのノードに最新のブロックを問い合わせて、最新状態に同期する。
                } else {
                    wprintln(format!("[connection]failed to verify this connection"));
                }
            } else if json_obj["type"].eq("get_latest") {
                let _blockchain = BLOCKCHAIN.read().unwrap();
                if _blockchain.len() > 0 {
                    self.write(format!(
                        "{{\"type\":\"block\",\"args\":{{\"block\":{}}}}}\r\n",
                        _blockchain[_blockchain.len() - 1].dump()
                    ));
                } else {
                    self.write("{\"type\":\"no_block\"}\r\n".to_string());
                }
            } else if json_obj["type"].eq("block") {
                let mut _block = BLOCKCHAIN.write().unwrap();
                println(format!("[connection]BLOCKCHAIN_LEN:{}", _block.len()));
                println(format!(
                    "[connection]received height:{}",
                    json_obj["args"]["block"]["height"].as_usize().unwrap()
                ));
                if json_obj["args"]["block"]["height"].as_usize().unwrap() > _block.len() {
                    let previous = match _block.len() > 0 {
                        true => Message::from_hashed_data::<sha256::Hash>(
                            _block[_block.len() - 1].dump().as_bytes(),
                        )
                        .to_string(),
                        false => block::GENESIS_BLOCK_HASH.to_string(),
                    };
                    if check(json_obj["args"]["block"].clone(), previous) {
                        println(format!(
                            "[connection]Received block is correct and taller than my block"
                        ));
                        for i in 0..TRUSTED_KEY.read().unwrap().len() {
                            println(format!(
                                "[connection]compare trusted_key:{} and {}",
                                TRUSTED_KEY.read().unwrap().get(&(i as isize)).unwrap(),
                                &json_obj["args"]["block"]["author"].to_string()
                            ));
                            if TRUSTED_KEY
                                .read()
                                .unwrap()
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
                        _block.push(json_obj["args"]["block"].clone());
                        println("[connection]New block pushed to my blockchain");
                    } else if check(json_obj["args"]["block"].clone(), "*".to_string()) {
                        println(format!(
                                        "[connection]Received block is correct but it taller than my blockchain for 2 or more."
                                    ));
                    } else {
                        wprintln(format!(
                            "[connection]Received block is taller than my block but not correct"
                        ));
                    }
                } else {
                    wprintln(format!(
                        "[connection]Received block is not taller than my block."
                    ));
                }

                *self.is_latest.write().unwrap() = true;
            } else if json_obj["type"].eq("no_block") {
                *self.is_latest.write().unwrap() = true;
            } else {
                wprintln(format!("[connection]connection received unknown command"));
            }
        }
    }
    pub fn write(&mut self, context: String) {
        if *self.is_connected.read().unwrap() {
            match (&*self.stream).write_all(context.as_bytes()) {
                Err(e) => {
                    println(format!(
                        "[connection]connection aborted due to :{}",
                        e.kind()
                    ));
                    *self.is_connected.write().unwrap() = false;
                }
                Ok(o) => o,
            };
            match (&*self.stream).flush() {
                Err(e) => {
                    println(format!(
                        "[connection]connection aborted due to :{}",
                        e.kind()
                    ));
                    *self.is_connected.write().unwrap() = false;
                }
                Ok(o) => o,
            };
        }
    }
}

pub fn ovserve() {
    loop {
        let mut _stats = STATS.write().unwrap();
        let len = _stats.connection_list.len();
        let mut rem: Vec<usize> = Vec::new();
        for i in 0.._stats.connection_list.len() {
            if !*_stats.connection_list[i].is_connected.read().unwrap() {
                rem.push(i);
            }
        }
        for i in rem {
            _stats.connection_list.remove(i);
        }
        let after_len = _stats.connection_list.len();
        drop(_stats);
        if len - after_len > 0 {
            println(format!(
                "[connection]connection_list's gabage collacted:{}->{}",
                len, after_len
            ));
        }

        thread::sleep(Duration::from_secs(12));
    }
}

pub fn init(stream: Arc<TcpStream>) {
    let mut _stats = STATS.write().unwrap();
    _stats.count += 1;
    let key = key_agent::SECRET
        .get()
        .unwrap()
        .public_key(&sign_util::SECP)
        .to_string();
    let mut con = Connection {
        id: Arc::new(RwLock::new(_stats.count)),
        is_connected: Arc::new(RwLock::new(true)),
        stream: stream,
        is_trusted: Arc::new(RwLock::new(false)),
        pubk: Arc::new(RwLock::new(Option::None)),
        nonce: Arc::new(RwLock::new(Option::None)),
        is_latest: Arc::new(RwLock::new(false)),
    };
    con.write(format!(
        "{{\"type\":\"hello\",\"args\":{{\"pubk\":\"{}\"}}}}\r\n",
        key
    ));
    _stats.connection_list.push(con.clone());
    thread::spawn(move || con.read_thread());
    println("[connection]inited connection instance");
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
