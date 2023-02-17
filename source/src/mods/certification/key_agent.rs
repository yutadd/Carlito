use crate::mods::util::system;
use rand::rngs::OsRng;
use secp256k1::{Secp256k1, SecretKey};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, BufReader, Write};
use std::{path::Path, str::FromStr};
pub static mut SECRET: Vec<SecretKey> = Vec::new();

/*
    implements later from https://github.com/rust-bitcoin/rust-secp256k1/blob/master/examples/sign_verify_recovery.rs
    and https://docs.rs/secp256k1/latest/secp256k1/
*/
//create file or read file
pub fn init() {
    let mut is_exst = true;
    let f: File;
    if !Path::new("secret/secret.txt").exists() {
        println!("[key_agent]secret isn't exists.");
        fs::create_dir_all("secret/").unwrap();
        is_exst = false;
    }
    println!("[key_agent]secret is exists");
    f = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open("secret/secret.txt")
        .unwrap();
    if !is_exst {
        println!("[key_agent]creating secret key.");
        create_new_key();
    }
    unsafe {
        read_key_from_file(f);
        assert!(SECRET.len() > 0);
    }
}
fn create_new_key() {
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    append_key_to_file(secret_key);
}
unsafe fn read_key_from_file(file: File) {
    let secp = Secp256k1::new();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        let key: SecretKey = SecretKey::from_str(&line).unwrap();
        SECRET.push(key);
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
pub fn get_key(index: usize) -> Option<&'static SecretKey> {
    unsafe { SECRET.get(index) }
}
pub fn get_key_length() -> usize {
    unsafe { SECRET.len() }
}
#[test]
fn key_agent_init() {
    init();
    for i in 0..(get_key_length()) {
        println!("[key_agent]init:{}", get_key(i).unwrap().display_secret());
    }
}
#[test]
fn make_key() {
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    println!("[key_agent]MAKE_KEY_SECRET:{}", secret_key.display_secret());
    println!(
        "[key_agent]MAKE_KEY_SECRET:{}",
        secret_key.public_key(&secp).to_string()
    )
}
#[test]
fn key_from_str() {
    let secp = Secp256k1::new();
    let sk =
        SecretKey::from_str("c2b56c7e50a19fbdd8fe5546fb21d2d7cb60c5fe95cd719bc64ba1fbf0bec955")
            .unwrap();
    println!(
        "[key_agent]key_from_str:{}",
        sk.public_key(&secp).to_string()
    );
}
