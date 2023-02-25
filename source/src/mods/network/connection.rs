use super::super::certification::key_agent;
use super::super::certification::sign_util;
use crate::mods::block::block;
use crate::mods::block::block::check;
use crate::mods::block::block::BLOCKCHAIN;
use crate::mods::certification::sign_util::TRUSTED_KEY;
use crate::mods::console::output::{eprintln, println, wprintln};
use crate::mods::poa::blockchain_manager;
use crate::mods::transaction::transaction;
use crate::mods::transaction::transaction::create_transaction;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use rand::prelude::*;
use secp256k1::hashes::sha256;
use secp256k1::Message;
use secp256k1::PublicKey;
use std::io::{BufRead, Write};
use std::net::IpAddr;
use std::net::Shutdown;
use std::str::FromStr;
use std::sync::Arc;
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
    pub addr: IpAddr, //固定
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
            addr: self.addr.clone(),
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
                    eprintln(format!(
                        "[connection{}]error on reading input buffer:{}",
                        self.addr,
                        e.kind()
                    ));
                    break;
                }
            };
            if bytes == 0 || line.trim().len() == 0 {
                *self.is_connected.write().unwrap() = false;
                drop(&self.is_connected);
                self.stream.shutdown(Shutdown::Both).unwrap();
                break;
            }
            let json_obj = json::parse(&line).unwrap();
            if json_obj["type"].eq("hello") {
                drop({
                    let _id = self.id.read().unwrap();
                    _id
                });
                println(format!(
                    "[connection{}]received pubk is {}",
                    self.addr,
                    json_obj["args"]["pubk"].to_string()
                ));
                let mut _pubk = self.pubk.write().unwrap();
                let pubk_copy = *_pubk.insert(
                    PublicKey::from_str(json_obj["args"]["pubk"].as_str().unwrap())
                        .unwrap()
                        .clone(),
                );
                drop(_pubk);

                if sign_util::is_host_trusted(pubk_copy.to_string()) {
                    let mut rng = rand::thread_rng();
                    let generated_rand = rng.next_u32();
                    self.write(format!(
                        "{{\"type\":\"req_sign\",\"args\":{{\"nonce\":\"{}\"}}}}\r\n",
                        generated_rand
                    ));

                    let _ = *self
                        .nonce
                        .write()
                        .unwrap()
                        .insert(format!("{}", generated_rand));
                } else {
                    wprintln(format!(
                        "[connection{}]connection not trusted for wrong pubk.",
                        self.addr,
                    ));
                }
            } else if json_obj["type"].eq("req_sign") {
                drop({
                    let _id = self.id.read().unwrap();
                    _id
                });
                let sign = sign_util::create_sign(
                    json_obj["args"]["nonce"].as_str().unwrap().to_string(),
                    *key_agent::SECRET.get().unwrap(),
                );
                self.write(format!(
                    "{{\"type\":\"signed\",\"args\":{{\"sign\":\"{}\"}}}}\r\n",
                    sign.to_string()
                ));
                println(format!("[connection{}]sign was sent", self.addr,));
            } else if json_obj["type"].eq("signed") {
                let verify_result;
                let pubk_copy;
                drop({
                    let _id = self.id.read().unwrap();
                    _id
                });
                drop({
                    let _pubk = self.pubk.read().unwrap();
                    pubk_copy = (*_pubk).unwrap().clone();
                    _pubk
                });
                verify_result = sign_util::verify_sign(
                    self.nonce.read().unwrap().clone().unwrap(),
                    json_obj["args"]["sign"].as_str().unwrap().to_string(),
                    pubk_copy,
                );

                if verify_result {
                    println(format!(
                        "[connection{}]verifying connection success",
                        self.addr,
                    ));
                    let mut _is_trusted = self.is_trusted.write().unwrap();
                    *_is_trusted = true;
                    println(format!(
                        "[connection{}]is trusted:{}",
                        self.addr,
                        _is_trusted.to_string()
                    ));
                    drop(_is_trusted);
                    self.write("{\"type\":\"get_latest\"}\r\n".to_string());
                //すべてのノードに最新のブロックを問い合わせて、最新状態に同期する。
                } else {
                    wprintln(format!(
                        "[connection{}]failed to verify this connection",
                        self.addr,
                    ));
                }
            } else if json_obj["type"].eq("get_latest") {
                let blockchain_copy;
                drop({
                    let _blockchain = BLOCKCHAIN.read().unwrap();
                    blockchain_copy = (*_blockchain).clone();
                    _blockchain
                });
                if blockchain_copy.len() > 0 {
                    self.write(format!(
                        "{{\"type\":\"block\",\"args\":{{\"block\":{}}}}}\r\n",
                        blockchain_copy[blockchain_copy.len() - 1].dump()
                    ));
                } else {
                    self.write("{\"type\":\"no_block\"}\r\n".to_string());
                }
            } else if json_obj["type"].eq("block") {
                let blockchain_copy;
                {
                    let _id = self.id.read().unwrap();
                }
                let trusted_copy;
                {
                    let _trusted = TRUSTED_KEY.read().unwrap();
                    trusted_copy = (*_trusted).clone();
                }
                {
                    let _blockchain = BLOCKCHAIN.read().unwrap();
                    blockchain_copy = (*_blockchain).clone();
                }
                if json_obj["args"]["block"]["height"].as_usize().unwrap() > blockchain_copy.len() {
                    let previous = match blockchain_copy.len() > 0 {
                        true => Message::from_hashed_data::<sha256::Hash>(
                            blockchain_copy[blockchain_copy.len() - 1].dump().as_bytes(),
                        )
                        .to_string(),
                        false => block::GENESIS_BLOCK_HASH.to_string(),
                    };
                    if check(json_obj["args"]["block"].clone(), previous) {
                        println(format!(
                            "[connection{}]Received block is correct and taller than my block",
                            self.addr,
                        ));
                        for i in 0..trusted_copy.len() {
                            println(format!(
                                "[connection{}]compare trusted_key:{} and {}",
                                self.addr,
                                trusted_copy.get(&(i as isize)).unwrap(),
                                &json_obj["args"]["block"]["author"].to_string()
                            ));
                            let mut _stats = blockchain_manager::STATS.write().unwrap();
                            if TRUSTED_KEY
                                .read()
                                .unwrap()
                                .get(&(i as isize))
                                .unwrap()
                                .eq(&json_obj["args"]["block"]["author"].to_string())
                            {
                                _stats.previous_generator = i as isize;
                                break;
                            }

                            drop(_stats);
                        }

                        let stats_copy;
                        let _stats = blockchain_manager::STATS.read().unwrap();
                        stats_copy = (*_stats).clone();
                        drop(_stats);
                        println(format!(
                            "[connection{}]previous_generator:{}",
                            self.addr, stats_copy.previous_generator
                        ));
                        {
                            let mut _blockchain = BLOCKCHAIN.write().unwrap();
                            _blockchain.push(json_obj["args"]["block"].clone());
                            println(format!(
                                "[connection{}]New block pushed to my blockchain",
                                self.addr
                            ));
                        }
                    } else if check(json_obj["args"]["block"].clone(), "*".to_string()) {
                        wprintln(format!(
                                        "[connection{}]Received block is correct but it taller than my blockchain for 2 or more.",self.addr,
                                    ));
                        self.write(format!(
                            "{{\"type\":\"fetch\",\"args\":{{\"from\":{}}}}}\r\n",
                            blockchain_copy.len()
                        ));
                        println(format!("[connection{}]sent fetch", self.addr,))
                    } else {
                        wprintln(format!(
                            "[connection{}]Received block is taller than my block but not correct",
                            self.addr,
                        ));
                    }
                } else {
                    wprintln(format!(
                        "[connection{}]Received block is not taller than my block.",
                        self.addr,
                    ));
                }

                *self.is_latest.write().unwrap() = true;
            } else if json_obj["type"].eq("fetch") {
                println("[connection]received fetch_request");
                let request_index = json_obj["args"]["from"].as_usize().unwrap();
                let blockchain_copy;
                {
                    let _blockchain = BLOCKCHAIN.read().unwrap();
                    blockchain_copy = (*_blockchain).clone();
                }
                println("[connection]locked blockchain");
                if request_index < blockchain_copy.len() {
                    for i in request_index..blockchain_copy.len() {
                        self.write(format!(
                            "{{\"type\":\"block\",\"args\":{{\"block\":{}}}}}\r\n",
                            blockchain_copy[i].dump()
                        ));
                    }
                }
                println("[connection]sent blockchain");
            } else if json_obj["type"].eq("transaction") {
                let transaction = json_obj["args"]["transaction"].clone();
                let verify = transaction::check(&transaction);
                if verify {
                    println(format!(
                        "[connection{}]verifying transaction success",
                        self.addr
                    ));
                    if transaction["content_type"].as_str().unwrap().eq("layer_0") {
                        let decoded_content = STANDARD_NO_PAD
                            .decode(transaction["content_b64"].as_str().unwrap())
                            .unwrap();
                        let content = String::from_utf8(decoded_content).unwrap();
                        let parsed_content = json::parse(&content).unwrap();
                        {
                            println(format!(
                                "[connection{}]received transaction content:{}",
                                self.addr, content
                            ));
                        }
                        if parsed_content["action"].to_string().eq("ping") {
                            println(format!("[connection{}]received ping", self.addr));
                            let transaction = create_transaction(
                                "layer_0".to_string(),
                                STANDARD_NO_PAD.encode("{\"action\":\"pong\"}".to_string()),
                            )
                            .unwrap();
                            if transaction::check(&transaction) {
                                self.write(format!(
                                    "{{\"type\":\"transaction\",\"args\":{{\"transaction\":{}}}}}\r\n",
                                    transaction.dump()
                                ));
                            }
                        } else if parsed_content["action"].to_string().eq("pong") {
                            println(format!("[connection{}]received pong", self.addr));
                        }
                    }
                }
            } else if json_obj["type"].eq("no_block") {
                *self.is_latest.write().unwrap() = true;
            } else {
                wprintln(format!(
                    "[connection{}]connection received unknown command",
                    self.addr,
                ));
            }
        }
    }
    pub fn write(&mut self, context: String) {
        if *self.is_connected.read().unwrap() {
            match (&*self.stream).write_all(context.as_bytes()) {
                Err(e) => {
                    println(format!(
                        "[connection{}]connection aborted due to :{}",
                        self.addr,
                        e.kind()
                    ));
                    *self.is_connected.write().unwrap() = false;
                }
                Ok(o) => o,
            };
            match (&*self.stream).flush() {
                Err(e) => {
                    println(format!(
                        "[connection{}]connection aborted due to :{}",
                        self.addr,
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
        let _cl;
        {
            let _stats = STATS.read().unwrap();
            _cl = _stats.connection_list.clone();
        }

        let mut rem: Vec<usize> = Vec::new();
        for i in 0.._cl.len() {
            if !*_cl[i].is_connected.read().unwrap() {
                rem.push(i);
            }
        }
        drop(_cl);
        let mut _stats = STATS.write().unwrap();
        let len = _stats.connection_list.len();
        for i in rem {
            println(format!(
                "[connection{}]was closed",
                _stats.connection_list[i].addr
            ));
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
    let ip = (&stream).peer_addr().unwrap().ip();
    let mut con = Connection {
        id: Arc::new(RwLock::new(_stats.count)),
        is_connected: Arc::new(RwLock::new(true)),
        stream: stream,
        is_trusted: Arc::new(RwLock::new(false)),
        pubk: Arc::new(RwLock::new(Option::None)),
        nonce: Arc::new(RwLock::new(Option::None)),
        is_latest: Arc::new(RwLock::new(false)),
        addr: ip,
    };
    con.write(format!(
        "{{\"type\":\"hello\",\"args\":{{\"pubk\":\"{}\"}}}}\r\n",
        key
    ));
    _stats.connection_list.push(con.clone());
    drop(_stats);
    thread::spawn(move || con.read_thread());
    println(format!("[connection{}]inited connection instance", ip));
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
