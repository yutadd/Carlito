use std::str::FromStr;

use crate::mods::certification::{
    key_agent,
    sign_util::{self, create_sign, SECP},
};
use crate::mods::console::output::{eprintln, println};
use base64::Engine;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, Utc};
use json::{object, JsonValue};
use secp256k1::{ecdsa::Signature, PublicKey};

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
pub fn parsing_json() {
    key_agent::init();
    sign_util::init();
    let example_transaction = object![
        author:"026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908".to_string(),
        date:1676449733,
        text_b64:"QURERiBwYXRoL3RvL2ZpbGUgdXNlcjAx".to_string(),
    ];
    let dumped_json = example_transaction.dump();
    println(format!("[transaction]dumped_transaction:{}", dumped_json));
    unsafe {
        println(format!(
            "[transaction]created_transaction_sign:{}",
            create_sign(dumped_json, key_agent::SECRET[0])
        ))
    }
    let check_result=check(json::parse("{
    \"author\":\"026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908\",
    \"date\":1676449733,
    \"text_b64\":\"QURERiBwYXRoL3RvL2ZpbGUgdXNlcjAx\",
    \"sign\":\"3045022100c4d6d23647dcbdbd1bf9f7abdbd2c427e6d0b732db4633f9fa6ceecdaa5f317b022013c8aba9606e48a5be1eebad06475fb5baeb1e92cd4059c10ee6507c9d38587a\"
}").unwrap());
    println(format!(
        "[transaction]check_example_transaction:{}",
        check_result
    ));
    assert!(check_result);
}
