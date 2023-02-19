use std::str::FromStr;

use crate::mods::certification::{
    key_agent,
    sign_util::{self, create_sign, SECP},
};
use crate::mods::console::output::{eprintln, println};
use base64::Engine;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, Utc};
use json::{object, JsonValue};
use secp256k1::{ecdsa::Signature, PublicKey, SecretKey};

pub fn check(transaction: JsonValue) -> bool {
    let transaction_without_sign = object![
        author:transaction["author"].to_string(),
        date:transaction["date"].to_string().parse::<usize>().unwrap(),
        text_b64:transaction["text_b64"].to_string(),
    ];
    sign_util::verify_sign(
        transaction_without_sign.dump(),
        transaction["sign"].to_string(),
        PublicKey::from_str(transaction_without_sign["author"].as_str().unwrap()).unwrap(),
    )
}

/**
 * 階層構造を扱わないのでraw_textにはcsvを用いる。
*/

#[test]
pub fn create_transaction() {
    let mut example_transaction = object![
        author:"02affab182d89e0ae1aa3e30e974b1ca55452f12f8e21d6e0125c47e689c614630".to_string(),
        date:1676449733,
        text_b64:"QURERiBwYXRoL3RvL2ZpbGUgdXNlcjAx".to_string(),
    ];
    let dumped_json = example_transaction.dump();
    println(format!("[transaction]dumped_transaction:{}", dumped_json));
    example_transaction
        .insert(
            "sign",
            create_sign(
                dumped_json,
                SecretKey::from_str(
                    "c2b56c7e50a19fbdd8fe5546fb21d2d7cb60c5fe95cd719bc64ba1fbf0bec955",
                )
                .unwrap(),
            )
            .to_string(),
        )
        .unwrap();
    println(format!(
        "[transaction]created_transaction_full:{}",
        example_transaction.dump()
    ));
    println(format!(
        "[transaction]check created transaction:{}",
        check(example_transaction)
    ));
}
