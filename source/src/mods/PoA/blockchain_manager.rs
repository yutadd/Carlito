use std::{collections::HashMap, sync::Mutex, thread, time::Duration};

use once_cell::sync::Lazy;

use crate::mods::{
    certification::{key_agent, sign_util},
    network::connection,
};
pub static mut PREVIOUS_GENERATOR: Mutex<String> = Mutex::new(String::new()); //ブロック読み込みや受け取り時に更新するべし

pub fn block_generate() {
    loop {
        if connection::is_all_connected() {
            unsafe {
                if !PREVIOUS_GENERATOR.lock().unwrap().eq(&String::new()) {
                    println!("[blockchain_manager]GENERATE BLOCK!");
                    thread::sleep(Duration::from_secs(1));
                } else {
                    println!("[blockchain_manager]preloaded chain is not ready");
                    thread::sleep(Duration::from_secs(8));
                }
            }
        } else {
            println!("[blockchain_manager]waiting connection for start generate block.");
            thread::sleep(Duration::from_secs(8));
        }
    }
}
pub fn get_next_generator(
    trusted_list: HashMap<usize, String>,
    previous_generator: String,
) -> String {
    for i in 0..trusted_list.len() {
        if trusted_list.get(&i).unwrap().eq(&*previous_generator) {
            return match i.eq(&(trusted_list.len() - 1)) {
                true => trusted_list.get(&0).unwrap().clone(),
                false => trusted_list.get(&(i + 1)).unwrap().clone(),
            };
        }
    }
    eprintln!("[blockchain_manager]trusted_key has no value");
    return String::new();
}
#[test]
pub fn get_nextgenerator() {
    key_agent::init();
    sign_util::init();
    unsafe {
        get_next_generator(
            sign_util::TRUSTED_KEY.clone(),
            "026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908".to_string(),
        );
    }
}
