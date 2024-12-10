use crate::contract_type::ContractType;
use crate::store::Storage;
use crate::util::config;
use crate::vault::Keypair;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Contract {
    id: String,
    hash: String,
    contract_type: ContractType,
    address: String,
    owner: String,
    fee: f64,
    limit: u64,
    timestamp: u64,
}

impl Contract {
    fn ledger() -> Storage {
        Storage::init().unwrap()
    }

    pub fn new() -> Result<Contract, String> {
        let keypair = KeyPair.generate();
    }

    pub fn get_contract(private_key: String) -> Result<Contract, String> {}

    pub fn get_contracts(private_key: String) -> Result<Contract, String> {}
}
