use std::{sync::Mutex, thread, time::Duration};

use crate::mods::block::block::{self, check, BLOCKCHAIN};
use crate::mods::certification::key_agent::SECRET;
use crate::mods::certification::sign_util::{create_sign, SECP, TRUSTED_KEY};
use crate::mods::console::output::println;
use crate::mods::{certification::key_agent, network::connection};
use chrono::{NaiveDateTime, Utc};
use json::{array, object};
use once_cell::sync::Lazy;
use secp256k1::hashes::sha256;
use secp256k1::Message;
#[derive(Clone)]
struct BlockChainStats {
    pub previous_generator: isize,
    //pub transaction_pool: Vec<JsonValue>,
}
static STATS: Lazy<Mutex<BlockChainStats>> = Lazy::new(|| {
    Mutex::new(BlockChainStats {
        previous_generator: -1,
        //transaction_pool: Vec::new(),
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
            let next_index;
            //信用リスト内部における次の生成者の添字を算出する
            if get_previous_generator() < ((TRUSTED_KEY.len() - 1) as isize) {
                next_index = get_previous_generator() + 1;
            } else {
                next_index = 0;
            }
            for c in connection::STATS
                .write()
                .unwrap()
                .connection_list
                .iter_mut()
            {
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
            if TRUSTED_KEY.get(&next_index).unwrap().eq(&key_agent::SECRET
                .get()
                .unwrap()
                .public_key(&SECP)
                .to_string())
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
                        author:SECRET.get().unwrap().public_key(&SECP).to_string(),
                        date:Utc::now().timestamp_millis(),
                        height:height+1,
                        transactions:array![],
                    ];
                    block
                        .insert(
                            "sign",
                            create_sign(block.dump(), *SECRET.get().unwrap()).to_string(),
                        )
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
                    let mut _next_index = -1;
                    if get_previous_generator() < ((TRUSTED_KEY.len() - 1) as isize) {
                        _next_index = get_previous_generator() + 1;
                    } else {
                        _next_index = 0;
                    }
                    println(format!(
                        "[blockchain_manager]next generator:{}",
                        _next_index
                    ));
                    for c in connection::STATS
                        .write()
                        .unwrap()
                        .connection_list
                        .iter_mut()
                    {
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
                let mut _next_index = -1;
                if get_previous_generator() < ((TRUSTED_KEY.len() - 1) as isize) {
                    _next_index = get_previous_generator() + 1;
                } else {
                    _next_index = 0;
                }
                println(format!(
                    "[blockchain_manager]next generator:{}",
                    _next_index
                ));
            }
            thread::sleep(Duration::from_secs(4));
        }
    }
}
