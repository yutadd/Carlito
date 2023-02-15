use mods::certification::key_agent;
use mods::certification::sign_util;
use mods::network::connection_listener;
use mods::network::dns_seed;
use std::io::stdin;
use std::thread;
mod mods;
/**

ラウンドロビンでは、過去のブロックを改ざんするには、全員の秘密鍵を入手しないとブロックを繋げられない。
TODO: 目的の定義
TODO: 目的に沿ったトランザクションの仕組み策定
TODO: ジェネシスブロックの表現
TODO: ラウンドロビンブロック生成の実装
TODO: ノード間TLS通信
TODO: プロキシへの対応
*/
fn main() {
    println!("Initializing...");
    key_agent::init();
    sign_util::init();
    thread::spawn(|| {
        connection_listener::run();
        println!("thread-Inited");
    });
    dns_seed::init();
    println!("Inited");
    loop {
        let line = &mut String::new();
        let size = stdin().read_line(line).unwrap();
        if size > 0 {
            println!("your input:{}", line);
        }
    }
}
