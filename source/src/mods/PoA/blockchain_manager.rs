use std::{collections::HashMap, sync::Mutex, thread, time::Duration};

use crate::mods::block::block::BLOCKCHAIN;
use crate::mods::certification::sign_util::{SECP, TRUSTED_KEY};
use crate::mods::config::config::YAML;
use crate::mods::console::output::{eprintln, println};
use crate::mods::network::connection::CONNECTION_LIST;
use crate::mods::{
    certification::{key_agent, sign_util},
    network::connection,
};
use once_cell::sync::Lazy;
pub static mut PREVIOUS_GENERATOR: isize = -1; //ブロック読み込みや受け取り時に更新するべし
pub fn block_generate() {
    loop {
        if connection::is_all_connected() {
            unsafe {
                if !PREVIOUS_GENERATOR.eq(&-1) {
                    let mut next_index = -1;
                    if PREVIOUS_GENERATOR < (TRUSTED_KEY.len() - 1).try_into().unwrap() {
                        next_index = PREVIOUS_GENERATOR + 1;
                    } else {
                        next_index = 0;
                    }
                    if TRUSTED_KEY
                        .get(&next_index)
                        .unwrap()
                        .eq(&key_agent::SECRET[0].public_key(&SECP).to_string())
                    {
                        println(format!("[blockchain_manager]GENERATING BLOCK!"));
                    }
                    thread::sleep(Duration::from_secs(1));
                } else {
                    println(format!("[blockchain_manager]preloaded chain is not ready"));
                    thread::sleep(Duration::from_secs(8));
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
                        println("[blockchain_manager]all node latest");
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
                }
            }
        } else {
            println(format!(
                "[blockchain_manager]waiting connection for start generate block."
            ));
            thread::sleep(Duration::from_secs(8));
        }
    }
}
