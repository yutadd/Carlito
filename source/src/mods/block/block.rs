use std::str::FromStr;

use chrono::{DateTime, Utc};
use secp256k1::{ecdsa::Signature, PublicKey};

use crate::mods::{
    certification::{key_agent, sign_util},
    transaction::{transaction::Transaction, transactions},
};

pub struct Block {
    pub author: PublicKey,
    pub date: i64,
    pub tx: Vec<Transaction>,
    pub sign: Signature,
    pub raw_data: String,
    pub transaction_data: String,
}
impl Block {
    pub fn check(&self) -> bool {
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
        false
    }
}
pub fn from_str(raw_data: String) -> Block {
    let rawdata_copy = raw_data.clone();
    let block_sign: Vec<&str> = raw_data.split("^").collect();
    let params: Vec<&str> = block_sign[0].split("-").collect();
    Block {
        author: PublicKey::from_str(params[0]).unwrap(),
        date: params[1].parse::<i64>().unwrap(),
        tx: transactions::from_str(params[2].to_string()),
        sign: Signature::from_str(block_sign[1]).unwrap(),
        raw_data: rawdata_copy,
        transaction_data: block_sign[0].to_string(),
    }
}
#[test]
pub fn parse_block() {
    key_agent::init();
    let text="026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908-1676449733-026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908,1676449733,QURERiBwYXRoL3RvL2ZpbGUgdXNlcjAx@3044022079e97bfe62f56c4d2123c67cff3f3e70e4444491ae4572132f996e8dce912ef9022056cc1a21c6ead5a7b121b67123da4152328ee27914dd4fdf47b8c289d902895b;026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908,1676449733,QURERiBwYXRoL3RvL2ZpbGUgdXNlcjAx@3044022079e97bfe62f56c4d2123c67cff3f3e70e4444491ae4572132f996e8dce912ef9022056cc1a21c6ead5a7b121b67123da4152328ee27914dd4fdf47b8c289d902895b";
    unsafe {
        let signed = sign_util::create_sign(text.to_string(), key_agent::SECRET[0]);
        let blk = from_str(format!("{}^{}", text, signed.to_string()));
        println!("block_check:{}", blk.check());
    }
}
