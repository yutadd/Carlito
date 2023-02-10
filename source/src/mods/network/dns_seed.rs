#![feature(catch_expr)]
use super::super::config_wrapper::config;
use super::connection;
use super::connection::UNTRUSTED_USERS;
use once_cell::sync::Lazy;
use std::net::Ipv4Addr;
use std::net::TcpStream;
use std::str::FromStr;
use std::sync::Arc;
use trust_dns_client::client::{Client, SyncClient};
use trust_dns_client::op::DnsResponse;
use trust_dns_client::rr::{DNSClass, Name, RData, RecordType};
use trust_dns_client::udp::UdpClientConnection;

pub static CLIENT: Lazy<SyncClient<UdpClientConnection>> = Lazy::new(|| unsafe {
    SyncClient::new(
        UdpClientConnection::new(
            format!(
                "{}:{}",
                config::YAML["network"]["dns"]["ip"].as_str().unwrap(),
                config::YAML["network"]["dns"]["port"].as_i64().unwrap()
            )
            .parse()
            .unwrap(),
        )
        .unwrap(),
    )
});

fn get_addr(name: String) -> Vec<Ipv4Addr> {
    println!("request resolv: {}", name);
    let name = Name::from_str(name.as_str()).unwrap();
    let response: DnsResponse = CLIENT.query(&name, DNSClass::IN, RecordType::A).unwrap();
    let mut v = Vec::new();
    for answor in response.answers() {
        if let Some(RData::A(addr)) = answor.data() {
            v.push(*addr);
            println!("fetched seeds addr:\n{}", *addr);
        }
    }
    v
}
pub fn init() {
    let is_docker;
    unsafe {
        is_docker = config::YAML["docker"]["is-docker"].as_bool().unwrap();
    }
    if is_docker {
        let hosts = Vec::from(["node01", "node02", "node03"]);
        for addr in hosts {
            unsafe {
                if addr
                    .to_string()
                    .eq(config::YAML["docker"]["own-name"].as_str().unwrap())
                {
                    continue;
                }
            }
            // TODO: 起動する順番によってはまだ接続できない場合があるから、そのための例外処理を行う
            let connection: TcpStream;
            match TcpStream::connect(format!("{}:{}", addr.to_string(), 7777)) {
                Ok(stream) => {
                    connection = stream;
                }
                Err(error) => {
                    println!("未接続:{}", error.kind());
                    continue;
                }
            }

            let _user = connection::init(Arc::new(connection), false);
            _user.read_thread();
            unsafe {
                UNTRUSTED_USERS.push(_user);
            }
        }
    } else {
        let mut addrs = Vec::new();
        unsafe {
            addrs = get_addr(
                config::YAML["network"]["domain"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            );
        }
        for addr in addrs {
            unsafe {
                if addr
                    .to_string()
                    .eq(config::YAML["network"]["own-ip"].as_str().unwrap())
                {
                    continue;
                }
            }
            let connection: TcpStream;
            match TcpStream::connect(format!("{}:{}", addr.to_string(), 7777)) {
                Ok(stream) => {
                    connection = stream;
                }
                Err(error) => {
                    println!("未接続:{}", error.kind());
                    continue;
                }
            }

            let _user = connection::init(Arc::new(connection), false);
            _user.read_thread();
            unsafe {
                UNTRUSTED_USERS.push(_user);
            }
        }
    }

    //TODO: 接続施行およびuser::USERSへの登録を行う。
}
#[test]
fn dns_seed_fetch() {
    let addrs = get_addr("yahoo.co.jp".to_string());
    println!("sum:{}", addrs.len());
}
