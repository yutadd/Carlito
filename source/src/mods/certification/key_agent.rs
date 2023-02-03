use secp256k1::{Secp256k1, Message,SecretKey, PublicKey};
use std::{str::FromStr, ptr::null_mut, fmt::format, path::Path};
use rand::rngs::OsRng;
use once_cell::sync::Lazy;
use std::env;
use std::fs::File;
use crate::mods::util::system;
use crate::mods::file_system::file_controll::{self, get_file};
use std::io::{prelude::*, ErrorKind};
use std::io::BufReader;
pub static SECRET:Vec<Lazy<SecretKey>>=Vec::new();

/*  
    implements later from https://github.com/rust-bitcoin/rust-secp256k1/blob/master/examples/sign_verify_recovery.rs
    and https://docs.rs/secp256k1/latest/secp256k1/ 
*/
//create file or read file
pub fn init(){
    let f;
    f=get_file("../secret/secret.txt".to_string());
    read_key_from_file(f);
}
fn create_new_key(){
    let f=get_file("../secret/secret.txt".to_string());
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    println!("{:?}",secret_key.display_secret());
    println!("{:?}",public_key.serialize());
    append_key_to_file(secret_key,f);
}
fn read_key_from_file(file:File)->Vec<&'static SecretKey>{
    let mut svec:Vec<&'static SecretKey>=  Vec::new();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        let key:&'static SecretKey;
        match SecretKey::from_str(&line){
            Ok(sec)=>key=&sec,
            Err(E)=>system::exit_with_error(E.to_string())
        };
        svec.push(&key);
    }
    svec
    
}
fn append_key_to_file(key:SecretKey,file:File){
    let secret_str=format!("{}\n",key.display_secret());
    
}
