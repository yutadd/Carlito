use std::str::FromStr;

use chrono::{DateTime, Utc};
use secp256k1::{ecdsa::Signature, PublicKey};

use crate::mods::{
    certification::sign_util,
    transaction::{transaction::Transaction, transactions},
};

pub struct Block {
    pub author: PublicKey,
    pub date: DateTime<Utc>,
    pub tx: Vec<Transaction>,
    pub sign: Signature,
    pub raw_data: String,
}
impl Block {
    pub fn check(&self) -> bool {
        if sign_util::verify_sign(self.raw_data.clone(), self.sign.to_string(), self.author) {
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
    let params: Vec<&str> = raw_data.split(",").collect();
    Block {
        author: PublicKey::from_str(params[0]).unwrap(),
        date: DateTime::from_str(params[1]).unwrap(),
        tx: transactions::from_str(params[2].to_string()),
        sign: Signature::from_str(params[3]).unwrap(),
        raw_data: raw_data,
    }
}
