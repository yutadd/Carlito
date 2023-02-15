use std::str::FromStr;

use base64::Engine;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, Utc};
use secp256k1::{ecdsa::Signature, PublicKey};

use crate::mods::certification::{
    key_agent,
    sign_util::{self, SECP},
};
pub struct Transaction {
    pub author: PublicKey,
    pub date: i64,
    pub text_b64: String,
    pub sign: Signature,
    pub raw_data: String,
    pub transaction_data: String,
}
impl Transaction {
    pub fn check(&self) -> bool {
        println!("{}\n{}\n{}", self.transaction_data, self.sign, self.author);
        sign_util::verify_sign(
            self.transaction_data.clone(),
            self.sign.to_string(),
            self.author,
        )
    }
}
/**
 * 階層構造を扱わないのでraw_textにはcsvを用いる。
*/
pub fn from_str(raw_data: String) -> Transaction {
    let raw_clone = raw_data.clone();
    let sign_params: Vec<&str> = raw_clone.split("@").collect();
    let param: Vec<&str> = sign_params[0].split(",").collect();
    println!("sign:{}", sign_params[1]);
    unsafe {
        println!(
            "{}",
            key_agent::SECRET
                .get(0)
                .unwrap()
                .public_key(&SECP)
                .to_string()
        );
    }
    Transaction {
        author: PublicKey::from_str(param[0]).unwrap(),
        date: param[1].parse::<i64>().unwrap(),
        text_b64: param[2].to_string(),
        sign: Signature::from_str(sign_params[1]).unwrap(),
        raw_data,
        transaction_data: sign_params[0].to_string(), //raw_data_without_sign
    }
}

#[test]
fn parse_transaction() {
    key_agent::init();
    //pubk,timestamp,QURERiBwYXRoL3RvL2ZpbGUgdXNlcjAx(base64 of ADDF path/to/file user01),
    let text="026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908,1676449733,QURERiBwYXRoL3RvL2ZpbGUgdXNlcjAx";
    unsafe {
        let sign = sign_util::create_sign(text.to_string(), *key_agent::SECRET.get(0).unwrap());
        let raw_data = format!("{}@{}", text, sign.to_string());
        println!("raw_data:{}", raw_data);
        let ts = from_str(raw_data);
        println!("transaction_check_sign:{}", ts.check());
        assert!(ts.check());
    }
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
