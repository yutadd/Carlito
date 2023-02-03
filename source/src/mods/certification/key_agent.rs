use secp256k1::{Secp256k1, Message,SecretKey, PublicKey};
use std::{str::FromStr, ptr::null_mut};
use rand::rngs::OsRng;
use once_cell::sync::Lazy;
pub static SECRET:Option<Lazy<SecretKey>>=None;
/*  
    implements later from https://github.com/rust-bitcoin/rust-secp256k1/blob/master/examples/sign_verify_recovery.rs
    and https://docs.rs/secp256k1/latest/secp256k1/ 
*/
//create file or read file
pub fn init(){
    add_new_key();
   //ファイルの存在を確認し、file_not_existsかread_key_from_file()か振り分けをする。
}
fn file_not_exists(){
    //ユーザーへ鍵を作成するか確認し、処理を振り分ける。
}
fn add_new_key(){
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    println!("{:?}",secret_key.display_secret());
    println!("{:?}",public_key.serialize());
    append_key_to_file(secret_key)
}
fn read_key_from_file()->Vec<SecretKey>{
    let key = SecretKey::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
    Vec::new()
}
fn append_key_to_file(key:SecretKey){
    println!("let me convert");
    assert_eq!(key,SecretKey::from_slice(&key.secret_bytes()).expect("OMG this is not convertible!"));
    println!("did i done successfully?");
}
