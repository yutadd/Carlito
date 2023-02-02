use secp256k1::{Secp256k1, Message,SecretKey};
use std::str::FromStr;
use rand::rngs::OsRng;

/*  
    implements later from https://github.com/rust-bitcoin/rust-secp256k1/blob/master/examples/sign_verify_recovery.rs
    and https://docs.rs/secp256k1/latest/secp256k1/ 
*/
//create file or read file
pub fn init(){
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    println!("{:?}",secret_key.display_secret());
    println!("{:?}",public_key.serialize());
}
fn read_key_from_file()->Vec<SecretKey>{
    let key = SecretKey::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
    Vec::new()
}
fn append_key_to_file(){

}