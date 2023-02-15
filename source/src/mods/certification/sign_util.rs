use once_cell::sync::Lazy;
use secp256k1::hashes::sha256;
use secp256k1::All;
use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1, SecretKey};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{prelude::*, BufReader};
use std::str::FromStr;
pub static mut TRUSTED_KEY: Lazy<HashMap<usize, String>> = Lazy::new(|| HashMap::new());
pub static SECP: Lazy<Secp256k1<All>> = Lazy::new(|| Secp256k1::new());
pub fn init() {
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open("Config/trusted_hosts.txt")
        .unwrap();
    let reader = BufReader::new(file);
    let index = 0;
    unsafe {
        for line in reader.lines() {
            let line = line.unwrap();
            TRUSTED_KEY.insert(index, line);
        }
    }
}
pub fn create_sign(original_message: String, secret_key: SecretKey) -> Signature {
    let message = Message::from_hashed_data::<sha256::Hash>(original_message.as_bytes());
    SECP.sign_ecdsa(&message, &secret_key)
}
pub fn verify_sign(original_message: String, sig: String, public_key: PublicKey) -> bool {
    let sig = Signature::from_str(&sig);
    if sig.is_ok() {
        let message = Message::from_hashed_data::<sha256::Hash>(original_message.as_bytes());
        SECP.verify_ecdsa(&message, &sig.unwrap(), &public_key)
            .is_ok()
    } else {
        println!("公開鍵のparseに失敗しました");
        false
    }
}
pub fn is_host_trusted(key: String) -> bool {
    let mut exists: bool = false;
    unsafe {
        for i in 0..TRUSTED_KEY.len() {
            if TRUSTED_KEY.get(&i).unwrap().eq_ignore_ascii_case(&key) {
                exists = true;
            }
        }
    }
    exists
}
#[test]
fn sign_util_init() {
    init();
}
#[test]
fn sign_util_trusted_host() {
    init();
    assert!(is_host_trusted(
        "026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908".to_string()
    ));
}

#[test]
fn sign_util_verify() {
    use crate::mods::certification::key_agent;
    key_agent::init();
    let sign = create_sign("HelloWorld".to_string(), *key_agent::get_key(0).unwrap());
    println!("{}", sign);
    println!(
        "{}",
        verify_sign(
            "HelloWorld".to_string(),
            sign.to_string(),
            key_agent::get_key(0).unwrap().public_key(&SECP)
        )
    );
    println!(
        "{}",
        verify_sign(
            "HelloWorld01".to_string(),
            sign.to_string(),
            key_agent::get_key(0).unwrap().public_key(&SECP)
        )
    );
    println!("end_test_signeture");
}
