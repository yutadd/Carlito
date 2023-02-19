use std::{collections::HashMap, sync::Mutex, thread, time::Duration};

use crate::mods::block::block::{check, BLOCKCHAIN};
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
use once_cell::sync::Lazy;
use secp256k1::hashes::sha256;
use secp256k1::Message;
pub static mut TRANSACTION_POOL: Lazy<Vec<JsonValue>> = Lazy::new(|| Vec::new());
pub static mut PREVIOUS_GENERATOR: isize = -1; //ブロック読み込みや受け取り時に更新するべし
pub fn block_generate() {
    unsafe {
        loop {
            if connection::is_all_connected() || BLOCKCHAIN.len() > 0 {
                //ブロックの読み込みと準備が完了しているか
                if !PREVIOUS_GENERATOR.eq(&-1) {
                    let mut next_index = -1;

                    //信用リスト内部における次の生成者の添字を算出する
                    if PREVIOUS_GENERATOR < (TRUSTED_KEY.len() - 1).try_into().unwrap() {
                        next_index = PREVIOUS_GENERATOR + 1;
                    } else {
                        next_index = 0;
                    }

                    //算出された次の生成車は自分か
                    if TRUSTED_KEY
                        .get(&next_index)
                        .unwrap()
                        .eq(&key_agent::SECRET[0].public_key(&SECP).to_string())
                    {
                        //一つ前のブロックが生成された時間から+10000ミリ秒以上過ぎているか
                        if NaiveDateTime::from_timestamp_millis(
                            (BLOCKCHAIN[BLOCKCHAIN.len() - 1]["date"].as_isize().unwrap() + 10000)
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
                            //ブロック生成
                            let mut block = object![
                                previous_hash:Message::from_hashed_data::<sha256::Hash>(BLOCKCHAIN[BLOCKCHAIN.len()-1].dump().as_bytes()).to_string(),
                                author:SECRET[0].public_key(&SECP).to_string(),
                                date:Utc::now().timestamp_millis(),
                                height:BLOCKCHAIN[BLOCKCHAIN.len()-1]["height"].as_usize().unwrap()+1,
                                transactions:array![],
                            ];
                            block
                                .insert("sign", create_sign(block.dump(), SECRET[0]).to_string())
                                .unwrap();
                            assert!(check(
                                block.clone(),
                                Message::from_hashed_data::<sha256::Hash>(
                                    BLOCKCHAIN[BLOCKCHAIN.len() - 1].dump().as_bytes(),
                                )
                                .to_string(),
                            ));
                            println("[blockchain_manager]block generated successfully");
                            BLOCKCHAIN.push(block.clone());
                            for c in CONNECTION_LIST.iter() {
                                c.write(format!(
                                    "{{\"type\":\"block\",\"args\":{{\"block\":{}}}}}\r\n",
                                    block.dump()
                                ));
                            }
                        }
                    }
                    thread::sleep(Duration::from_secs(4));
                } else {
                    println(format!("[blockchain_manager]waiting for load local block"));
                    let mut latest_nodes = 0;
                    let mut trusted_nodes = 0;
                    for c in CONNECTION_LIST.iter() {
                        if c.is_latest {
                            latest_nodes += 1;
                        } else {
                            c.write("{\"type\":\"get_latest\"}\r\n".to_string());
                        }
                        if c.is_trusted {
                            trusted_nodes += 1;
                        }
                    }
                    if latest_nodes == trusted_nodes {
                        let author = BLOCKCHAIN[BLOCKCHAIN.len() - 1]["author"].to_string();
                        for i in 0..TRUSTED_KEY.len() {
                            if TRUSTED_KEY.get(&(i as isize)).unwrap().eq(&author) {
                                PREVIOUS_GENERATOR = isize::try_from(i).unwrap();
                            }
                        }
                    } else {
                        println(format!(
                            "[blockchain_manager]there is not latest node:{}/{}",
                            latest_nodes, trusted_nodes
                        ));
                    }
                    thread::sleep(Duration::from_secs(4));
                }
            } else {
                println(format!(
                    "[blockchain_manager]waiting connection and prepare blockchain"
                ));
                thread::sleep(Duration::from_secs(4));
            }
        }
    }
}
