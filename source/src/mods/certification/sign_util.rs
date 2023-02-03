use secp256k1::{Secp256k1, Message, PublicKey};
pub static mut TRUSTED_HOST:Vec<PublicKey>=Vec::new();