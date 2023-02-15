use std::str::FromStr;

use chrono::{DateTime, Utc};
use secp256k1::{ecdsa::Signature, PublicKey};

use crate::mods::{
    certification::{key_agent, sign_util},
    transaction::{transaction::Transaction, transactions},
};
pub static genesis_block_hash: &str =
    "3F6D388DB566932F70F35D15D9FA88822F40075BDAAA370CCB40536D2FC18C3D";
pub struct Block {
    pub previous_hash: String,
    pub author: PublicKey,
    pub date: i64,
    pub height: usize,
    pub tx: Vec<Transaction>,
    pub sign: Signature,
    pub raw_data: String,
    pub transaction_data: String,
}
impl Block {
    pub fn check(&self) -> bool {
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
    }
}
pub fn from_str(raw_data: String) -> Block {
    let rawdata_copy = raw_data.clone();
    let block_sign: Vec<&str> = raw_data.split("^").collect();
    let params: Vec<&str> = block_sign[0].split("-").collect();
    Block {
        previous_hash: params[0].to_string(),
        author: PublicKey::from_str(params[1]).unwrap(),
        date: params[2].parse::<i64>().unwrap(),
        height: params[3].parse::<usize>().unwrap(),
        tx: transactions::from_str(params[4].to_string()),
        sign: Signature::from_str(block_sign[1]).unwrap(),
        raw_data: rawdata_copy,
        transaction_data: block_sign[0].to_string(),
    }
}
#[test]
pub fn parse_block() {
    key_agent::init();
    let text="3F6D388DB566932F70F35D15D9FA88822F40075BDAAA370CCB40536D2FC18C3D-026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908-1676449733-1-026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908,1676449733,QURERiBwYXRoL3RvL2ZpbGUgdXNlcjAx@3044022079e97bfe62f56c4d2123c67cff3f3e70e4444491ae4572132f996e8dce912ef9022056cc1a21c6ead5a7b121b67123da4152328ee27914dd4fdf47b8c289d902895b;026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908,1676449733,QURERiBwYXRoL3RvL2ZpbGUgdXNlcjAx@3044022079e97bfe62f56c4d2123c67cff3f3e70e4444491ae4572132f996e8dce912ef9022056cc1a21c6ead5a7b121b67123da4152328ee27914dd4fdf47b8c289d902895b";
    unsafe {
        let signed = sign_util::create_sign(text.to_string(), key_agent::SECRET[0]);
        let blk = from_str(format!("{}^{}", text, signed.to_string()));
        println!("block_check:{}", blk.check());
    }
}
