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
static COUNT: Lazy<Mutex<u16>> = Lazy::new(|| Mutex::new(0));
pub static IS_BLOCKED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
/**
 * 重複するノード・接続終了したノードの削除ループの実装
 * */
pub struct ConnectionStats {
    pub connection_list: Vec<Connection>,
    pub count: u16,
}
pub static STATS: Lazy<RwLock<ConnectionStats>> = Lazy::new(|| {
    RwLock::new(ConnectionStats {
        connection_list: Vec::new(),
        count: 0,
    })
});
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
                self.is_connected = false;
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
                self.pubk = Option::Some(
                    PublicKey::from_str(json_obj["args"]["pubk"].as_str().unwrap())
                        .unwrap()
                        .clone(),
                );

                if sign_util::is_host_trusted(self.pubk.unwrap().to_string()) {
                    let mut rng = rand::thread_rng();
                    let generated_rand = rng.next_u32();
                    self.write(format!(
                        "{{\"type\":\"req_sign\",\"args\":{{\"nonce\":\"{}\"}}}}\r\n",
                        generated_rand
                    ));

                    self.nonce = Option::Some(format!("{}", generated_rand));
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
                    self.nonce.clone().unwrap(),
                    json_obj["args"]["sign"].as_str().unwrap().to_string(),
                    self.pubk.unwrap(),
                );

                if verify_result {
                    println(format!("[connection]verifying connection success"));
                    self.is_trusted = true;
                    println(format!(
                        "[connection]is trusted:{}",
                        self.is_trusted.to_string()
                    ));
                    self.write("{\"type\":\"get_latest\"}\r\n".to_string());

                //すべてのノードに最新のブロックを問い合わせて、最新状態に同期する。
                } else {
                    wprintln(format!("[connection]failed to verify this connection"));
                }
            } else if json_obj["type"].eq("get_latest") {
                if BLOCKCHAIN.read().unwrap().len() > 0 {
                    self.write(format!(
                        "{{\"type\":\"block\",\"args\":{{\"block\":{}}}}}\r\n",
                        BLOCKCHAIN.read().unwrap()[BLOCKCHAIN.read().unwrap().len() - 1].dump()
                    ));
                } else {
                    self.write("{\"type\":\"no_block\"}\r\n".to_string());
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
                            } else if check(json_obj["args"]["block"].clone(), "*".to_string()) {
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
                self.is_latest = true;
            } else if json_obj["type"].eq("no_block") {
                self.is_latest = true;
            } else {
                wprintln(format!("[connection]connection received unknown command"));
            }
        }
    }
    pub fn write(&mut self, context: String) {
        if self.is_connected {
            match (&*self.stream).write_all(context.as_bytes()) {
                Err(e) => {
                    println(format!(
                        "[connection]connection aborted due to :{}",
                        e.kind()
                    ));
                    self.is_connected = false;
                }
                Ok(o) => o,
            };
            match (&*self.stream).flush() {
                Err(e) => {
                    println(format!(
                        "[connection]connection aborted due to :{}",
                        e.kind()
                    ));
                    self.is_connected = false;
                }
                Ok(o) => o,
            };
        }
    }
}
/**pub fn is_all_connected() -> bool {
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
pub fn ovserve() {
    loop {
        let mut _stats = STATS.write().unwrap();
        let len = _stats.connection_list.len();
        let mut rem: Vec<usize> = Vec::new();
        for i in 0.._stats.connection_list.len() {
            if !_stats.connection_list[i].is_connected {
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

pub fn init(stream: Arc<TcpStream>) -> Connection {
    *COUNT.lock().unwrap() += 1;
    let key = key_agent::SECRET
        .get()
        .unwrap()
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
        key_agent::SECRET.get().unwrap().display_secret(),
        key
    ));

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
