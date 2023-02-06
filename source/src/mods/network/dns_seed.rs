use once_cell::sync::Lazy;
use std::net::Ipv4Addr;
use trust_dns_client::client::{Client,SyncClient};
use trust_dns_client::udp::UdpClientConnection;
use std::str::FromStr;
use trust_dns_client::op::DnsResponse;
use trust_dns_client::rr::{DNSClass,RData, Name, RecordType};
pub static CLIENT:Lazy<SyncClient<UdpClientConnection>>=Lazy::new(||SyncClient::new(UdpClientConnection::new("8.8.8.8:53".parse().unwrap()).unwrap()));
fn get_addr(name:String)->Vec<Ipv4Addr>{
        let name = Name::from_str(name.as_str()).unwrap();
        let response: DnsResponse = CLIENT.query(&name, DNSClass::IN, RecordType::A).unwrap();
        let mut v=Vec::new();
        for answor in  response.answers(){
            if let Some(RData::A(addr)) =answor.data() {
                v.push(*addr);
                println!("fetched seeds addr:\n{}",*addr);
            }
            
        }
        v
}
pub fn init(){
    let addrs=get_addr("seed.yutadd.com".to_string());
    //TODO: 接続施行およびuser::USERSへの登録を行う。
}
#[test]
fn dns_seed_fetch(){
    let addrs=get_addr("yahoo.co.jp".to_string());
    println!("sum:{}",addrs.len());
}