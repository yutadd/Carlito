use super::super::config::config;
use super::connection;
use crate::mods::console::output::{eprintln, println, wprintln};
use once_cell::sync::Lazy;
use std::net::Ipv4Addr;
use std::net::TcpStream;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use trust_dns_client::client::{Client, SyncClient};
use trust_dns_client::rr::{DNSClass, Name, RData, RecordType};
use trust_dns_client::udp::UdpClientConnection;

pub static CLIENT: Lazy<SyncClient<UdpClientConnection>> = Lazy::new(|| {
    config::init();
    SyncClient::new(
        UdpClientConnection::new(
            format!(
                "{}:{}",
                config::YAML.get().unwrap()["network"]["dns"]["ip"]
                    .as_str()
                    .unwrap(),
                config::YAML.get().unwrap()["network"]["dns"]["port"]
                    .as_i64()
                    .unwrap()
            )
            .parse()
            .unwrap(),
        )
        .unwrap(),
    )
});

fn get_addr(name: String) -> Vec<Ipv4Addr> {
    println(format!("[dns_seed]request resolv: {}", name));
    let name = Name::from_str(name.as_str()).unwrap();
    loop {
        match CLIENT.query(&name, DNSClass::IN, RecordType::A) {
            Ok(o) => {
                let mut v = Vec::new();
                for answor in o.answers() {
                    if let Some(RData::A(addr)) = answor.data() {
                        v.push(*addr);
                        println(format!("[dns_seed]fetched seeds addr: {}", *addr));
                    }
                }
                return v;
            }
            Err(e) => {
                eprintln(format!("[dns_seed]error on quering DNS:{}", e.kind()));
                return Vec::new();
            }
        };
    }
}
pub fn init() {
    let is_docker;
    is_docker = config::YAML.get().unwrap()["docker"]["is-docker"]
        .as_bool()
        .unwrap();
    if is_docker {
        let hosts = Vec::from(["node01", "node02", "node03"]);
        let own_name = config::YAML.get().unwrap()["docker"]["own-name"]
            .as_str()
            .unwrap();
        thread::sleep(Duration::from_secs((own_name.as_bytes()[5] - 48).into()));
        for addr in hosts {
            if addr.to_string().eq(own_name) {
                continue;
            }
            // TODO: 起動する順番によってはまだ接続できない場合があるから、そのための例外処理を行う
            let connection: TcpStream;
            match TcpStream::connect(format!("{}:{}", addr.to_string(), 7777)) {
                Ok(stream) => {
                    connection = stream;
                }
                Err(error) => {
                    wprintln(format!("[dns_seed]未接続:{}", error.kind()));
                    continue;
                }
            }
            connection::init(Arc::new(connection));
        }
        //↑dockerが同時に立ち上がり、listeningしていないときに接続を試みることを防ぐため、名前に合わせて数秒待つ
    } else {
        let addrs = get_addr(
            config::YAML.get().unwrap()["network"]["domain"]
                .as_str()
                .unwrap()
                .to_string(),
        );

        for addr in addrs {
            if addr
                .to_string()
                .eq(config::YAML.get().unwrap()["network"]["own-ip"]
                    .as_str()
                    .unwrap())
            {
                continue;
            }
            let connection: TcpStream;
            match TcpStream::connect(format!("{}:{}", addr.to_string(), 7777)) {
                Ok(stream) => {
                    connection = stream;
                }
                Err(error) => {
                    wprintln(format!("[dns_seed]未接続:{}", error.kind()));
                    continue;
                }
            }
            connection::init(Arc::new(connection));
        }
    }

    //TODO: 接続施行およびuser::USERSへの登録を行う。
}
#[test]
fn dns_seed_fetch() {
    let addrs = get_addr("amazon.com".to_string());
    println(format!("[dns_seed]sum:{}", addrs.len()));
}
