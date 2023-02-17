use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use secp256k1::{ecdsa::Signature, PublicKey};
use std::fs::{self, File};
use std::io::prelude::*;
use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader},
    str::FromStr,
};

use crate::mods::{
    certification::{key_agent, sign_util},
    transaction::transactions,
};
pub static genesis_block_hash: &str =
    "3F6D388DB566932F70F35D15D9FA88822F40075BDAAA370CCB40536D2FC18C3D";
pub static tx_per_file: usize = 100;
//pub static mut BLOCKCHAIN: Lazy<Vec<Block>> = Lazy::new(|| Vec::new());

/*pub fn check(&self) -> bool {
    if (self.height == 1 && self.previous_hash.eq(&genesis_block_hash.to_string()))
        || (/*TODO: impl later*/false)
    //Must be implemented to check if a block is connected to the previous block.
    {
        if sign_util::verify_sign(
            self.transaction_data.clone(),
            self.sign.to_string(),
            self.author,
        ) {
            for t in self.tx.iter() {
                if !t.check() {
                    return false;
                }
            }
            return true;
        }
    }
    false
}*/

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
pub fn read_block_from_local() -> usize {
    let mut i: usize = 0;
    let mut last_block_height = 0;
    create_directory_if_not_exists();
    loop {
        println!("entered a loop");
        i += 1;
        let f: File;
        match OpenOptions::new()
            .read(true)
            .open(format!("Blocks/Block-{}.txt", i))
        {
            Ok(_f) => f = _f,
            Err(e) => {
                println!("ERR:{}", e);
                break;
            }
        };
        let reader = BufReader::new(f);
        for line in reader.lines() {
            let line = line.unwrap();
            if line.eq("") {
                println!("found EOF");
                break; //読み込み途中で""になったということはブロックはここまでであり、これ以上ブロックファイルも存在しないはずなので、loopも抜ける。
            } else {
                println!("readed line:{}", line);
                last_block_height = 1;
                /*let _block = from_str(line);
                assert!(_block.check());

                println!("height:{}", last_block_height);
                unsafe {
                    BLOCKCHAIN.push(_block);
                }*/
            }
        }
    }
    assert_eq!(get_file_and_index(last_block_height).0, i - 1);
    i
}
pub fn get_file_and_index(height: usize) -> (usize, usize) {
    assert!(height > 0);
    return ((height / tx_per_file) + 1, height % tx_per_file);
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
