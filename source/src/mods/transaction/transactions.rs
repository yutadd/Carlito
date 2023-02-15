use std::string;

use super::transaction::{self, Transaction};

pub fn from_str(raw_data: String) -> Vec<Transaction> {
    let params: Vec<&str> = raw_data.split(";").collect();
    let mut tx = Vec::new();
    for i in 0..params.len() {
        tx.push(transaction::from_str(params[i].to_string()));
    }
    tx
}
#[test]
pub fn parse_transactions() {
    let raw_data="026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908,1676449733,QURERiBwYXRoL3RvL2ZpbGUgdXNlcjAx@3044022079e97bfe62f56c4d2123c67cff3f3e70e4444491ae4572132f996e8dce912ef9022056cc1a21c6ead5a7b121b67123da4152328ee27914dd4fdf47b8c289d902895b;026992eaf45a8a7b3e37ca6d586a3110d2af2c39c5547852d1028bd1144480b908,1676449733,QURERiBwYXRoL3RvL2ZpbGUgdXNlcjAx@3044022079e97bfe62f56c4d2123c67cff3f3e70e4444491ae4572132f996e8dce912ef9022056cc1a21c6ead5a7b121b67123da4152328ee27914dd4fdf47b8c289d902895b";
    let tx = from_str(raw_data.to_string());
    for t in tx.iter() {
        println!("check-transaction:{}", t.check());
    }
}
