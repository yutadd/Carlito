use std::sync::RwLock;
use std::{sync::Mutex, thread, time::Duration};

use crate::mods::block::block::{self, check, BLOCKCHAIN};
use crate::mods::certification::key_agent::SECRET;
use crate::mods::certification::sign_util::{create_sign, SECP, TRUSTED_KEY};
use crate::mods::console::output::println;
use crate::mods::{certification::key_agent, network::connection};
use chrono::{NaiveDateTime, Utc};
use json::{array, object};
use secp256k1::hashes::sha256;
use secp256k1::Message;
#[derive(Clone)]
pub struct BlockChainStats {
    pub previous_generator: isize,
    //pub transaction_pool: Vec<JsonValue>,
}
pub static STATS: RwLock<BlockChainStats> = RwLock::new(BlockChainStats {
    previous_generator: -1,
    //transaction_pool: Vec::new(),
});

pub fn block_generate() {
    loop {
        let next_index;
        let mut stats_copy;
        {
            let _stats = STATS.read().unwrap();
            stats_copy = (*_stats).clone();
        }
        let blockchain_copy;
        {
            let _blockchain = BLOCKCHAIN.read().unwrap();
            blockchain_copy = (*_blockchain).clone();
        }

        let trusted_copy;
        {
            let _trusted = TRUSTED_KEY.read().unwrap();
            trusted_copy = (*_trusted).clone();
        }
        if stats_copy.previous_generator < ((trusted_copy.len() - 1) as isize) {
            next_index = stats_copy.previous_generator + 1;
        } else {
            next_index = 0;
        }
        //信用リスト内部における次の生成者の添字を算出する
        let mut _stats = connection::STATS.write().unwrap();
        for c in _stats.connection_list.iter_mut() {
            if blockchain_copy.len() > 0 {
                /*  c.write(format!(
                    "{{\"type\":\"block\",\"args\":{{\"block\":{}}}}}\r\n",
                    blockchain_copy[blockchain_copy.len() - 1].dump()
                ));*/
            } else {
                println("[blockchain_manager]block not generated yet")
            }
        }
        drop(_stats);
        //算出された次の生成車は自分か
        if trusted_copy.get(&next_index).unwrap().eq(&key_agent::SECRET
            .get()
            .unwrap()
            .public_key(&SECP)
            .to_string())
        {
            //一つ前のブロックが生成された時間から+10000ミリ秒以上過ぎているかもしくはブロックがまだ生成されていないか
            if blockchain_copy.len() == 0
                || NaiveDateTime::from_timestamp_millis(
                    (blockchain_copy[blockchain_copy.len() - 1]["date"]
                        .as_isize()
                        .unwrap()
                        + 10000)
                        .try_into()
                        .unwrap(),
                )
                .unwrap()
                .cmp(&NaiveDateTime::from_timestamp_millis(Utc::now().timestamp_millis()).unwrap())
                .is_lt()
            {
                let height;
                if blockchain_copy.len() > 0 {
                    height = blockchain_copy[blockchain_copy.len() - 1]["height"]
                        .as_usize()
                        .unwrap();
                } else {
                    height = 0;
                }
                let previous;
                if blockchain_copy.len() > 0 {
                    previous = Message::from_hashed_data::<sha256::Hash>(
                        blockchain_copy[blockchain_copy.len() - 1].dump().as_bytes(),
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
                let mut _blockchain = BLOCKCHAIN.write().unwrap();
                _blockchain.push(block.clone());
                drop(_blockchain);
                let mut _stats = STATS.write().unwrap();
                for i in 0..trusted_copy.len() {
                    if trusted_copy
                        .get(&(i as isize))
                        .unwrap()
                        .eq(&block["author"])
                    {
                        _stats.previous_generator = i as isize;
                        break;
                    }
                }
                stats_copy = (*_stats).clone();
                drop(_stats);
                //信用リスト内部における次の生成者の添字を算出する
                let mut _next_index = -1;
                if stats_copy.previous_generator < ((trusted_copy.len() - 1) as isize) {
                    _next_index = stats_copy.previous_generator + 1;
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
            if stats_copy.previous_generator < ((trusted_copy.len() - 1) as isize) {
                _next_index = stats_copy.previous_generator + 1;
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
