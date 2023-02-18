use crate::mods::console::output::{eprintln, println};
use crate::mods::network::connection;
use chrono::{DateTime, Utc};
use json::{array, object, JsonValue};
use once_cell::sync::Lazy;
use secp256k1::hashes::sha256;
use secp256k1::Message;
use secp256k1::{ecdsa::Signature, PublicKey};
use std::fs::{self, File};
use std::io::prelude::*;
use std::ops::IndexMut;
use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader},
    str::FromStr,
};

use crate::mods::certification::sign_util::create_sign;
use crate::mods::{
    certification::{key_agent, sign_util},
    transaction::transaction,
    transaction::transactions,
};
pub static genesis_block_hash: &str =
    "3F6D388DB566932F70F35D15D9FA88822F40075BDAAA370CCB40536D2FC18C3D";
pub static tx_per_file: usize = 100;
pub static mut BLOCKCHAIN: Lazy<Vec<JsonValue>> = Lazy::new(|| Vec::new());

pub fn check(block: JsonValue, previous_hash: String) -> bool {
    println(format!("[block]dumped_full_block:{}", block.dump()));
    let block_without_sign = object![
        previous_hash:block["previous_hash"].to_string(),
        author:block["author"].to_string(),
        date:block["date"].to_string().parse::<usize>().unwrap(),
        height:block["height"].to_string().parse::<usize>().unwrap(),
        transactions:block["transactions"].clone(),
    ];
    if previous_hash.eq(&block["previous_hash"].to_string()) {
        println(format!("[block]verifying block:{}", block_without_sign));
        let mut any_invalid_ts = false;
        for t in block_without_sign["transactions"].members() {
            println(format!("[block]verifying transaction:{}", t));
            if !transaction::check(t.clone()) {
                any_invalid_ts = true;
                eprintln(format!("[block]invalid transaction:{}", t))
            } else {
                println(format!("[block]perfect transaction:{}", t))
            }
        }
        if !any_invalid_ts {
            sign_util::verify_sign(
                block_without_sign.dump(),
                block["sign"].to_string(),
                PublicKey::from_str(block_without_sign["author"].as_str().unwrap()).unwrap(),
            )
        } else {
            eprintln(format!("[block]threre is invalid transaction"));
            false
        }
    } else {
        eprintln(format!(
            "previous hash:{} not match:{}",
            previous_hash,
            block["previous_hash"].to_string()
        ));
        false
    }
}

pub fn append_block(text: String) {
    let height_str = text.split("-").collect::<Vec<&str>>();
    let height = height_str[3].parse::<usize>().unwrap();
    create_directory_if_not_exists();
    let file_target = get_file_and_index(height).0;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .write(true)
        .open(format!("Blocks/Block-{}.txt", file_target))
        .unwrap();
    writeln!(file, "{}", text).unwrap();
}
pub fn create_directory_if_not_exists() {
    fs::create_dir_all("Blocks/").unwrap();
}
pub fn read_block_from_local() {
    let mut previous = genesis_block_hash.to_string();
    let mut i: usize = 0;
    let mut last_block_height = 0;
    create_directory_if_not_exists();
    loop {
        println(format!("[block]entered a loop"));
        i += 1;
        let f: File;
        match OpenOptions::new()
            .read(true)
            .open(format!("Blocks/Block-{}.json", i))
        {
            Ok(_f) => {
                f = _f;
                println(format!("[block]there is block"))
            }
            Err(e) => {
                println(format!("[block]ERR:{}", e));
                i -= 1;
                break;
            }
        };
        let reader = BufReader::new(f);
        for line in reader.lines() {
            let line = line.unwrap();
            if line.eq("") {
                println(format!("[block]found EOF"));
                break; //読み込み途中で""になったということはブロックはここまでであり、これ以上ブロックファイルも存在しないはずなので、loopも抜ける。
            } else {
                println(format!("[block]readed line:{}", line));
                let _block = json::parse(line.as_str()).unwrap();
                let _prev = previous.clone();
                assert!(check(_block.clone(), _prev.to_string()));
                let hash = Message::from_hashed_data::<sha256::Hash>(_block.dump().as_bytes());
                println(format!("[block]dump block hash:{}", hash.clone()));
                previous = hash.to_string();
                last_block_height = _block["height"].as_usize().unwrap();
                println(format!("[block]height:{}", last_block_height));
                unsafe {
                    BLOCKCHAIN.push(_block);
                }
            }
        }
    }
    assert_eq!(get_file_and_index(last_block_height).0, i);
    unsafe {
        assert!(BLOCKCHAIN.len() > 0);
    }
    unsafe {
        for c in connection::CONNECTION_LIST.iter() {
            c.write("{\"type\":\"get_latest\"}\r\n".to_string());
        }
    }
}
pub fn get_file_and_index(height: usize) -> (usize, usize) {
    assert!(height > 0);
    return ((height / tx_per_file) + 1, height % tx_per_file);
}

#[test]
pub fn parsing_json() {
    key_agent::init();
    sign_util::init();
    let example_transaction = object![
        author:"026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908".to_string(),
        date:1676449733,
        text_b64:"QURERiBwYXRoL3RvL2ZpbGUgdXNlcjAx".to_string(),
        sign:"3045022100c4d6d23647dcbdbd1bf9f7abdbd2c427e6d0b732db4633f9fa6ceecdaa5f317b022013c8aba9606e48a5be1eebad06475fb5baeb1e92cd4059c10ee6507c9d38587a"
    ];
    let example_transactions = array![example_transaction];
    let example_block = object![
        previous_hash:"3F6D388DB566932F70F35D15D9FA88822F40075BDAAA370CCB40536D2FC18C3D".to_string(),
        author:"026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908".to_string(),
        date:1676449733,
        height:1,
        transactions:example_transactions,
    ];
    let dumped_json = example_block.dump();
    println(format!("[block]dumped_block:{}", dumped_json));
    unsafe {
        println(format!(
            "[block]created_block_sign:{}",
            create_sign(dumped_json, key_agent::SECRET[0])
        ))
    }
    let check_result=check(json::parse("{
        \"previous_hash\":\"3F6D388DB566932F70F35D15D9FA88822F40075BDAAA370CCB40536D2FC18C3D\",
        \"author\":\"026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908\",
        \"date\":1676449733,
        \"height\":1,
        \"transactions\":[{
            \"author\":\"026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908\",
            \"date\":1676449733,
            \"text_b64\":\"QURERiBwYXRoL3RvL2ZpbGUgdXNlcjAx\",
            \"sign\":\"3045022100c4d6d23647dcbdbd1bf9f7abdbd2c427e6d0b732db4633f9fa6ceecdaa5f317b022013c8aba9606e48a5be1eebad06475fb5baeb1e92cd4059c10ee6507c9d38587a\"}
            ],
        \"sign\":\"304402200ca1d60f83187635da209bc7521dbd96fc896b740e8c3589c4462c2ce2ca70ac02206488ec6b7ea6d55eecd32a70ec5b89538e235e5452b09d4b254911b7d9d913cd\"}").unwrap(),"3F6D388DB566932F70F35D15D9FA88822F40075BDAAA370CCB40536D2FC18C3D".to_string());
    println(format!("[block]check_example_block:{}", check_result));
    assert!(check_result);
}

#[test]
pub fn get_index() {
    assert_eq!(get_file_and_index(1).0, 1);
    assert_eq!(get_file_and_index(99).0, 1);
    assert_eq!(get_file_and_index(100).0, 2);
    assert_eq!(get_file_and_index(1).1, 1);
    assert_eq!(get_file_and_index(99).1, 99);
    assert_eq!(get_file_and_index(100).1, 0);
}
#[test]
pub fn _append_block() {
    //append_block("previoushash-pubk-time-10".to_string());
}
#[test]
pub fn read_block() {
    read_block_from_local();
}
