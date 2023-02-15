use std::str::FromStr;

use base64::Engine;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, Utc};
use secp256k1::{ecdsa::Signature, PublicKey};

use crate::mods::certification::sign_util;
pub struct Transaction {
    pub author: PublicKey,
    pub date: NaiveDateTime,
    pub text_b64: String,
    pub sign: Signature,
    pub raw_data: String,
}
impl Transaction {
    pub fn check(&self) -> bool {
        sign_util::verify_sign(self.raw_data.clone(), self.sign.to_string(), self.author)
    }
}
/**
 * 階層構造を扱わないのでraw_textにはcsvを用いる。
*/
pub fn from_str(raw_data: String) -> Transaction {
    let params: Vec<&str> = raw_data.split(",").collect();
    Transaction {
        author: PublicKey::from_str(params[0]).unwrap(),
        date: NaiveDateTime::from_str(params[1]).unwrap(),
        text_b64: params[2].to_string(),
        sign: Signature::from_str(params[3]).unwrap(),
        raw_data,
    }
}

#[test]
fn create_transaction() {
    let raw_data = "f0cd5f5b47d983c4c5c173444e577bcffda3884f6f53b03cf5f97b5ed25d692f,".to_string();
}
#[test]
fn show_date() {
    let timestamp = Local::now().timestamp_millis();
    println!("{}", timestamp);
    println!(
        "{}",
        NaiveDateTime::from_timestamp_millis(timestamp)
            .unwrap()
            .timestamp_millis()
    )
}
