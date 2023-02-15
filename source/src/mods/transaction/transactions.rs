use std::string;

use super::transaction::{self, Transaction};

pub fn from_str(raw_data: String) -> Vec<Transaction> {
    let params: Vec<&str> = raw_data.split("@").collect();
    let mut tx = Vec::new();
    for i in 0..params.len() {
        tx.push(transaction::from_str(params[i].to_string()));
    }
    tx
}
