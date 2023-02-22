use crate::mods::console::output::{eprintln, println};
use mods::block::block;
use mods::block::block::read_block_from_local;
use mods::certification::key_agent;
use mods::certification::sign_util;
use mods::network::connection;
use mods::network::connection_listener;
use mods::network::dns_seed;
use once_cell::sync::Lazy;
use std::io::stdin;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use crate::mods::PoA::blockchain_manager;
mod mods;
/**

ラウンドロビンでは、過去のブロックを改ざんするには、全員の秘密鍵を入手しないとブロックを繋げられない。<br />
ネイティブ通貨の概念が存在しないため、ローカルブロックの読み込みは順番とハッシュの繋がりのチェックだけ行う。<br />
TODO: 起動時の最新出ないノードリストをすべて表示するようにする
TODO: ブロックのシェア<br />
TODO: ブロック＆トランザクションのモジュールディレクトリの整形<br />
TODO: ノード間TLS通信<br />
TODO: プロキシへの対応<br />
*/
fn main() {
    println(format!("[main]Initializing..."));
    key_agent::init();
    sign_util::init();
    block::read_block_from_local();
    thread::spawn(|| blockchain_manager::block_generate());
    thread::spawn(|| {
        connection_listener::run();
    });
    println(format!("[main]thread-Inited"));
    dns_seed::init();
    println(format!("[main]Inited"));
    loop {
        let line = &mut String::new();
        let size = stdin().read_line(line).unwrap();
        if size > 0 {
            println(format!("[main]your input:{}", line));
        }
    }
}
struct TEST {
    pub num: isize,
}
impl Clone for TEST {
    fn clone(&self) -> TEST {
        TEST { num: self.num }
    }
}
impl TEST {
    pub fn test_th(&self) {
        println!("num:{}", TESTARRAY.lock().unwrap()[0].num);
        TESTARRAY.lock().unwrap()[0].num = 50;
    }
}
static TESTARRAY: Lazy<Mutex<Vec<TEST>>> = Lazy::new(|| Mutex::new(Vec::new()));
#[test]
pub fn numeric_convert() {
    let i: isize = 10;
    let u: usize = i as usize;
    assert!(u == 10);
    let i = u as isize;
    assert!(i == 10)
}
#[test]
pub fn access_test() {
    let test = TEST { num: -1 };
    TESTARRAY.lock().unwrap().push(test.clone());
    thread::spawn(move || test.test_th());
    thread::sleep(Duration::from_secs(1));
    println!("finish:{}", TESTARRAY.lock().unwrap()[0].num);
    println!("finish:{}", TESTARRAY.lock().unwrap()[0].num);
    println!("leng:{}", TESTARRAY.lock().unwrap().len());
}
