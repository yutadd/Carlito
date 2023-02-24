use crate::mods::certification::key_agent::SECRET;
use crate::mods::certification::sign_util::{self};
use crate::mods::certification::sign_util::{create_sign, SECP};
use crate::mods::console::output::{eprintln, println};
use chrono::Utc;
use json::{object, JsonValue};
use secp256k1::PublicKey;
use std::str::FromStr;
pub fn check(transaction: &JsonValue) -> bool {
    let transaction_without_sign = object![
        author:transaction["author"].to_string(),
        date:transaction["date"].to_string().parse::<usize>().unwrap(),
        content_type:transaction["content_type"].to_string(),
        content_b64:transaction["text_b64"].to_string(),
    ];
    sign_util::verify_sign(
        transaction_without_sign.dump(),
        transaction["sign"].to_string(),
        PublicKey::from_str(transaction_without_sign["author"].as_str().unwrap()).unwrap(),
    )
}
pub fn create_transaction(content_type: String, content_b64: String) -> Option<JsonValue> {
    let mut example_transaction = object![
        author:SECRET.get().unwrap().public_key(&SECP).to_string(),
        date:Utc::now().timestamp_millis(),
        content_type:content_type.to_string(),
        content_b64:content_b64.to_string(),
    ];
    let dumped_json = example_transaction.dump();
    println(format!("[transaction]dumped_transaction:{}", dumped_json));
    example_transaction
        .insert(
            "sign",
            create_sign(dumped_json, *SECRET.get().unwrap()).to_string(),
        )
        .unwrap();
    if check(&example_transaction) {
        println("[transaction]created transaction successfully");
        return Option::Some(example_transaction);
    } else {
        eprintln("[transaction]failed to create transaction");
        return Option::None;
    }
}
#[test]
pub fn test_transaction() {
    let mut example_transaction = object![
        author:"02affab182d89e0ae1aa3e30e974b1ca55452f12f8e21d6e0125c47e689c614630".to_string(),
        date:1676449733,
        content_type:"carlito_asm".to_string(),
        content_b64:"QURERiBwYXRoL3RvL2ZpbGUgdXNlcjAx".to_string(),
    ];
    let dumped_json = example_transaction.dump();
    println(format!("[transaction]dumped_transaction:{}", dumped_json));
    example_transaction
        .insert(
            "sign",
            create_sign(dumped_json, *SECRET.get().unwrap()).to_string(),
        )
        .unwrap();
    println(format!(
        "[transaction]created_transaction_full:{}",
        example_transaction.dump()
    ));
    println(format!(
        "[transaction]check created transaction:{}",
        check(&example_transaction)
    ));
}
