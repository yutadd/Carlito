use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex, thread, time::Duration};

use crate::mods::block::block::{self, check, BLOCKCHAIN};
use crate::mods::certification::key_agent::SECRET;
use crate::mods::certification::sign_util::{create_sign, SECP, TRUSTED_KEY};
use crate::mods::config::config::YAML;
use crate::mods::console::output::{eprintln, println};
use crate::mods::network::connection::CONNECTION_LIST;
use crate::mods::{
    certification::{key_agent, sign_util},
    network::connection,
};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use json::{array, object, JsonValue};
use once_cell::sync::{Lazy, OnceCell};
use secp256k1::hashes::sha256;
use secp256k1::Message;
#[derive(Clone)]
struct BlockChainStats {
    pub previous_generator: isize,
    pub transaction_pool: Vec<JsonValue>,
}
static STATS: Lazy<Mutex<BlockChainStats>> = Lazy::new(|| {
    Mutex::new(BlockChainStats {
        previous_generator: -1,
        transaction_pool: Vec::new(),
    })
});
pub fn set_previous_generator(index: isize) {
    println(format!(
        "[blockchain_manager]before_change:{}",
        STATS.lock().unwrap().previous_generator
    ));
    STATS.lock().unwrap().previous_generator = index;
    println(format!(
        "[blockchain_manager]after_change:{}",
        STATS.lock().unwrap().previous_generator
    ));
}
pub fn get_previous_generator() -> isize {
    STATS.lock().unwrap().previous_generator
}
pub fn block_generate() {
    unsafe {
        loop {
            let mut next_index = -1;
            //信用リスト内部における次の生成者の添字を算出する
            if get_previous_generator() < ((TRUSTED_KEY.len() - 1) as isize) {
                next_index = get_previous_generator() + 1;
            } else {
                next_index = 0;
            }
            for c in CONNECTION_LIST.iter() {
                if BLOCKCHAIN.read().unwrap().len() > 0 {
                    c.write(format!(
                        "{{\"type\":\"block\",\"args\":{{\"block\":{}}}}}\r\n",
                        BLOCKCHAIN.read().unwrap()[BLOCKCHAIN.read().unwrap().len() - 1].dump()
                    ));
                } else {
                    println("[blockchain_manager]block not generated yet")
                }
            }
            //算出された次の生成車は自分か
            if TRUSTED_KEY
                .get(&next_index)
                .unwrap()
                .eq(&key_agent::SECRET[0].public_key(&SECP).to_string())
            {
                //一つ前のブロックが生成された時間から+10000ミリ秒以上過ぎているかもしくはブロックがまだ生成されていないか
                if BLOCKCHAIN.read().unwrap().len() == 0
                    || NaiveDateTime::from_timestamp_millis(
                        (BLOCKCHAIN.read().unwrap()[BLOCKCHAIN.read().unwrap().len() - 1]["date"]
                            .as_isize()
                            .unwrap()
                            + 10000)
                            .try_into()
                            .unwrap(),
                    )
                    .unwrap()
                    .cmp(
                        &NaiveDateTime::from_timestamp_millis(Utc::now().timestamp_millis())
                            .unwrap(),
                    )
                    .is_lt()
                {
                    let height;
                    if BLOCKCHAIN.read().unwrap().len() > 0 {
                        height = BLOCKCHAIN.read().unwrap()[BLOCKCHAIN.read().unwrap().len() - 1]
                            ["height"]
                            .as_usize()
                            .unwrap();
                    } else {
                        height = 0;
                    }
                    let previous;
                    if BLOCKCHAIN.read().unwrap().len() > 0 {
                        previous = Message::from_hashed_data::<sha256::Hash>(
                            BLOCKCHAIN.read().unwrap()[BLOCKCHAIN.read().unwrap().len() - 1]
                                .dump()
                                .as_bytes(),
                        )
                        .to_string()
                    } else {
                        previous = block::GENESIS_BLOCK_HASH.to_string();
                    }
                    let mut block = object![//ブロック生成
                        previous_hash:previous.clone(),
                        author:SECRET[0].public_key(&SECP).to_string(),
                        date:Utc::now().timestamp_millis(),
                        height:height+1,
                        transactions:array![],
                    ];
                    block
                        .insert("sign", create_sign(block.dump(), SECRET[0]).to_string())
                        .unwrap();
                    assert!(check(block.clone(), previous));
                    println("[blockchain_manager]block generated successfully");
                    BLOCKCHAIN.write().unwrap().push(block.clone());
                    for i in 0..TRUSTED_KEY.len() {
                        if TRUSTED_KEY.get(&(i as isize)).unwrap().eq(&block["author"]) {
                            set_previous_generator(i as isize);
                            break;
                        }
                    }
                    //信用リスト内部における次の生成者の添字を算出する
                    let _next_index = -1;
                    if get_previous_generator() < ((TRUSTED_KEY.len() - 1) as isize) {
                        next_index = get_previous_generator() + 1;
                    } else {
                        next_index = 0;
                    }
                    println(format!("[blockchain_manager]next generator:{}", next_index));
                    for c in CONNECTION_LIST.iter() {
                        c.write(format!(
                            "{{\"type\":\"block\",\"args\":{{\"block\":{}}}}}\r\n",
                            block.dump()
                        ));
                    }
                } else {
                    println("[blockchain_manager]time not elapsed");
                }
            } else {
                println("[blockchain_manager]generator is not me this time");
                let _next_index = -1;
                if get_previous_generator() < ((TRUSTED_KEY.len() - 1) as isize) {
                    next_index = get_previous_generator() + 1;
                } else {
                    next_index = 0;
                }
                println(format!("[blockchain_manager]next generator:{}", next_index));
            }
            thread::sleep(Duration::from_secs(4));
        }
    }
}
