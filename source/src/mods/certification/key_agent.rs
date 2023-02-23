use crate::mods::console::output::println;
use once_cell::sync::OnceCell;
use rand::rngs::OsRng;
use secp256k1::SecretKey;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, BufReader, Write};
use std::{path::Path, str::FromStr};

use super::sign_util::SECP;
pub static SECRET: OnceCell<SecretKey> = OnceCell::new();

/*
    implements later from https://github.com/rust-bitcoin/rust-secp256k1/blob/master/examples/sign_verify_recovery.rs
    and https://docs.rs/secp256k1/latest/secp256k1/
*/
//create file or read file
pub fn init() {
    let mut is_exst = true;
    let f: File;
    if !Path::new("secret/secret.txt").exists() {
        println(format!("[key_agent]secret isn't exists."));
        fs::create_dir_all("secret/").unwrap();
        is_exst = false;
    }
    println(format!("[key_agent]secret is exists"));
    f = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open("secret/secret.txt")
        .unwrap();
    if !is_exst {
        println(format!("[key_agent]creating secret key."));
        create_new_key();
    }
    read_key_from_file(f);
}
fn create_new_key() {
    let (secret_key, _public_key) = SECP.generate_keypair(&mut OsRng);
    append_key_to_file(secret_key);
}
fn read_key_from_file(file: File) {
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        let key: SecretKey = SecretKey::from_str(&line).unwrap();
        SECRET.set(key).unwrap();
    }
}
fn append_key_to_file(key: SecretKey) {
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(Path::new("secret/secret.txt"))
        .unwrap();
    let secret_str = format!("{}\n", key.display_secret());
    f.write(secret_str.as_bytes()).unwrap();
    f.flush().unwrap();
}

#[test]

fn key_agent_init() {
    use secp256k1::Secp256k1;
    let secp = Secp256k1::new();
    init();

    println(format!(
        "[key_agent]MAKE_KEY_SECRET:{}",
        SECRET.get().unwrap().display_secret()
    ));

    println(format!(
        "[key_agent]MAKE_KEY_PUBLIC:{}",
        SECRET.get().unwrap().public_key(&secp).to_string()
    ))
}
#[test]
fn key_from_str() {
    use secp256k1::Secp256k1;
    let secp = Secp256k1::new();
    let sk =
        SecretKey::from_str("c2b56c7e50a19fbdd8fe5546fb21d2d7cb60c5fe95cd719bc64ba1fbf0bec955")
            .unwrap();
    println(format!(
        "[key_agent]key_from_str:{}",
        sk.public_key(&secp).to_string()
    ));
}
