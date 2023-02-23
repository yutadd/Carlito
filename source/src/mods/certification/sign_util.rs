use crate::mods::console::output::{eprintln, println};
use once_cell::sync::Lazy;
use secp256k1::hashes::sha256;
use secp256k1::All;
use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1, SecretKey};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{prelude::*, BufReader};
use std::str::FromStr;
pub static mut TRUSTED_KEY: Lazy<HashMap<isize, String>> = Lazy::new(|| HashMap::new());
pub static SECP: Lazy<Secp256k1<All>> = Lazy::new(|| Secp256k1::new());
pub fn init() {
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open("Config/trusted_hosts.txt")
        .unwrap();
    let reader = BufReader::new(file);
    let mut index = 0;
    unsafe {
        for line in reader.lines() {
            let line = line.unwrap();
            if line.trim().len() > 0 {
                TRUSTED_KEY.insert(index, line);
                index += 1;
            }
        }
    }
    assert!(index > 0);
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
        eprintln(format!("[sign_util]公開鍵のparseに失敗しました"));
        false
    }
}
pub fn is_host_trusted(key: String) -> bool {
    let mut exists: bool = false;
    let mut vector_str = "".to_string();
    unsafe {
        println(format!("[sign_util]trusted_hosts:{}", TRUSTED_KEY.len()));
        for i in 0..TRUSTED_KEY.len() {
            vector_str = format!(
                "{}{}{}",
                vector_str,
                TRUSTED_KEY.get(&(i as isize)).unwrap().as_str(),
                "\n"
            );
            if TRUSTED_KEY.get(&(i as isize)).unwrap().eq(&key) {
                exists = true;
                break;
            }
        }
    }
    println(format!(
        "[sign_util]key:{} was {} on [{}]",
        key, exists, vector_str
    ));
    exists
}
#[test]
fn sign_util_init() {
    init();
}

#[test]
fn sign_util_verify() {
    use crate::mods::certification::key_agent;
    use crate::mods::certification::key_agent::SECRET;
    key_agent::init();
    let sign = create_sign("HelloWorld".to_string(), *SECRET.get().unwrap());
    println(format!("[sign_util]show sign:{}", sign));
    println(format!(
        "[sign_util]verify sign:{}",
        verify_sign(
            "HelloWorld".to_string(),
            sign.to_string(),
            SECRET.get().unwrap().public_key(&SECP)
        )
    ));
    println(format!(
        "[sign_util]verify wrong message:{}",
        verify_sign(
            "HelloWorld01".to_string(),
            sign.to_string(),
            SECRET.get().unwrap().public_key(&SECP)
        )
    ));
    println(format!("[sign_util]end_test_signeture"));
}
