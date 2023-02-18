use crate::mods::console::output::{eprintln, println};
use mods::block::block;
use mods::block::block::read_block_from_local;
use mods::certification::key_agent;
use mods::certification::sign_util;
use mods::network::connection_listener;
use mods::network::dns_seed;
use std::io::stdin;
use std::thread;

use crate::mods::PoA::blockchain_manager;
mod mods;
/**

ラウンドロビンでは、過去のブロックを改ざんするには、全員の秘密鍵を入手しないとブロックを繋げられない。<br />
ネイティブ通貨の概念が存在しないため、ローカルブロックの読み込みは順番とハッシュの繋がりのチェックだけ行う。<br />
TODO: ローテーションの実装方法。
TODO: ブロック生成方法の策定<br />
TODO: ファイルでの表現の作成<br />
TODO: ブロックに関するファイル操作の実装<br />
TODO: ブロックのシェア<br />
TODO: ブロック＆トランザクションのモジュールディレクトリの整形<br />
TODO: ノード間TLS通信<br />
TODO: プロキシへの対応<br />
*/
fn main() {
    println(format!("[main]Initializing..."));
    key_agent::init();
    sign_util::init();
    thread::spawn(|| {
        connection_listener::run();
        println(format!("[main]thread-Inited"));
    });
    dns_seed::init();
    thread::spawn(|| block::read_block_from_local());
    thread::spawn(|| blockchain_manager::block_generate());
    println(format!("[main]Inited"));
    loop {
        let line = &mut String::new();
        let size = stdin().read_line(line).unwrap();
        if size > 0 {
            println(format!("[main]your input:{}", line));
        }
    }
}
