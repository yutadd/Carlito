use once_cell::sync::Lazy;
use secp256k1::{Secp256k1, Message, PublicKey, SecretKey, ecdsa::Signature};
use secp256k1::hashes::{sha256};
use std::{str::FromStr, };
use crate::mods::util::system;
use std::fs::{OpenOptions};
use secp256k1::All;
use std::io::{prelude::*,BufReader};
pub static mut TRUSTED_KEY:Vec<String>=Vec::new();
pub static SECP:Lazy<Secp256k1<All>> = Lazy::new(||Secp256k1::new());
pub fn init(){
    let file=OpenOptions::new().create(true).read(true).write(true).open("Config/trusted_hosts.txt").unwrap();
    let mut pvec:Vec<String>=  Vec::new();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        pvec.push(line);
    }
    unsafe{
        for _ in 0..pvec.len(){
            TRUSTED_KEY.push(pvec.pop().unwrap());
        }
    }
    
}
pub fn create_sign(original_message:String,secret_key:SecretKey)->Signature{
    let message = Message::from_hashed_data::<sha256::Hash>(original_message.as_bytes());
    SECP.sign_ecdsa(&message, &secret_key)
}
pub fn verify_sign(original_message:String,sig:String,public_key:PublicKey)->bool{
    let sig=Signature::from_str(&sig);
    if sig.is_ok() {
        let message=Message::from_hashed_data::<sha256::Hash>(original_message.as_bytes());
        SECP.verify_ecdsa(&message, &sig.unwrap(), &public_key).is_ok()
    }else{
        println!("公開鍵のparseに失敗しました");
        false
    }
    
}
pub fn is_host_trusted(x_only_public_key:String)->bool{
    let mut exists:bool=false;
    unsafe{
        for i in 0..TRUSTED_KEY.len(){
            if TRUSTED_KEY.get(i).unwrap().eq_ignore_ascii_case(&x_only_public_key){
                exists=true;
            }
        }
    }
    exists
}