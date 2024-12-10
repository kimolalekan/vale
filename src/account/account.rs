use super::wallet::Wallet;
use crate::{
    store::{Storage, StorageKind},
    vault::Crypto,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BalanceType {
    Binary(Vec<u8>),
    Text(String),
    Decimal(f64),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Balance {
    pub address: String,
    pub balance: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountWithPrivateKey {
    pub address: String,
    pub balance: f64,
    pub public_key: String,
    pub private_key: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Account {
    pub address: String,
    pub balance: BalanceType,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AccountIndex {
    address: String,
    public_key: String,
}

impl BalanceType {
    pub fn to_vec(&self) -> Result<Vec<u8>, String> {
        match self {
            BalanceType::Binary(data) => Ok(data.clone()),
            BalanceType::Text(text) => {
                let bytes = text.as_bytes().to_vec();
                Ok(bytes)
            }
            BalanceType::Decimal(decimal) => {
                let bytes = decimal.to_ne_bytes().to_vec();
                Ok(bytes)
            }
        }
    }
}

impl Account {
    fn ledger() -> Storage {
        Storage::init().unwrap()
    }

    pub fn new() -> Result<AccountWithPrivateKey, String> {
        let wallet = Wallet::new();
        let private_key = wallet.private_key.clone();
        let public_key = wallet.public_key.clone();
        let timestamp = Utc::now().timestamp() as u64;

        let account_with_public_key = AccountWithPrivateKey {
            address: wallet.address.clone(),
            balance: 0.0,
            public_key: public_key.clone(),
            private_key,
            timestamp,
        };

        let balance: f64 = 0.0;
        let balance = balance.to_string();
        let balance_bytes = balance.as_bytes();

        let encrypted_balance = Crypto::encrypt(balance_bytes.to_vec(), Some(public_key.clone()))?;
        let balance_type = BalanceType::Binary(encrypted_balance.data);

        let account = Account {
            address: wallet.address.clone(),
            balance: balance_type,
            timestamp,
        };

        let account_index = AccountIndex {
            address: account.address.clone(),
            public_key: public_key.clone(),
        };

        let store = Self::ledger();
        let key: Vec<u8> = bincode::serialize(&public_key).map_err(|e| e.to_string())?;
        let value: Vec<u8> = bincode::serialize(&account).map_err(|e| e.to_string())?;
        store.put(StorageKind::Account.name(), &key, &value, true)?;

        let address_key: Vec<u8> =
            bincode::serialize(&account_index.address).map_err(|e| e.to_string())?;
        let address_value: Vec<u8> =
            bincode::serialize(&account_index.public_key).map_err(|e| e.to_string())?;
        store.put(
            StorageKind::Index.name(),
            &address_key,
            &address_value,
            true,
        )?;

        Ok(account_with_public_key)
    }

    pub fn get_account_index(address: String) -> Result<String, String> {
        let store = Self::ledger();

        let key: Vec<u8> = bincode::serialize(&address).map_err(|e| e.to_string())?;
        let value = store.get(StorageKind::Index.name(), &key)?;

        let value: String = bincode::deserialize(&value).map_err(|e| e.to_string())?;
        Ok(value)
    }

    pub fn get_account(address: String) -> Result<Account, String> {
        let store = Self::ledger();

        let key: Vec<u8> = bincode::serialize(&address).map_err(|e| e.to_string())?;
        let account_key = store.get(StorageKind::Index.name(), &key)?;
        let account_key: String = bincode::deserialize(&account_key).map_err(|e| e.to_string())?;

        let wallet_address = Wallet::verify_address(&address);
        if wallet_address.is_ok() {
            let store = Self::ledger();
            let key: Vec<u8> = bincode::serialize(&account_key).map_err(|e| e.to_string())?;
            let account = store.get(StorageKind::Account.name(), &key)?;
            let account: Account = bincode::deserialize(&account).map_err(|e| e.to_string())?;

            let account_balance = Account {
                address: account.address,
                balance: BalanceType::Text("Encrypted provide private_key to decrypt".to_string()),
                timestamp: account.timestamp,
            };

            Ok(account_balance)
        } else {
            Err("Not a valid wallet address".to_string())
        }
    }

    pub fn get_account_details(private_key: String) -> Result<Account, String> {
        let public_key = Wallet::verify(&private_key)?;

        let store = Self::ledger();
        let key: Vec<u8> = bincode::serialize(&public_key).map_err(|e| e.to_string())?;
        let account = store.get(StorageKind::Account.name(), &key)?;
        let account: Account = bincode::deserialize(&account).map_err(|e| e.to_string())?;
        let balance_bytes = match account.balance {
            BalanceType::Binary(data) => data,
            _ => return Err("Balance type is not binary".to_string()),
        };

        let _key = public_key.clone();
        let decrypted_data = Crypto::decrypt(balance_bytes, &_key)?;
        let balance = String::from_utf8_lossy(&decrypted_data.data).to_string();
        let balance = balance.parse::<f64>().unwrap();
        let account_details = Account {
            address: account.address,
            balance: BalanceType::Decimal(balance),
            timestamp: account.timestamp,
        };

        Ok(account_details)
    }

    pub fn get_balance(address: String, private_key: String) -> Result<Balance, String> {
        let wallet_address = Wallet::verify_address(&address);
        let public_key = Wallet::verify(&private_key)?;

        if wallet_address.is_ok() {
            let store = Self::ledger();
            let key: Vec<u8> = bincode::serialize(&public_key).map_err(|e| e.to_string())?;
            let account = store.get(StorageKind::Account.name(), &key)?;
            let account: Account = bincode::deserialize(&account).map_err(|e| e.to_string())?;

            let balance_bytes = match account.balance {
                BalanceType::Binary(data) => data,
                _ => return Err("Balance type is not binary".to_string()),
            };

            let _key = public_key.clone();
            let decrypted_data = Crypto::decrypt(balance_bytes, &_key)?;
            let balance = String::from_utf8_lossy(&decrypted_data.data).to_string();
            let balance = balance.parse::<f64>().unwrap();
            let account_balance = Balance {
                address: account.address,
                balance,
            };

            Ok(account_balance)
        } else {
            Err("Not a valid wallet address".to_string())
        }
    }

    pub fn get_accounts(page: usize, limit: usize) -> Result<Vec<Account>, String> {
        let start = if page > 1 { (page - 1) * limit } else { 0 };
        let store = Self::ledger();
        let results = match store.batch_get(StorageKind::Account.name(), start, limit) {
            Ok(results) => results,
            Err(e) => {
                eprintln!("Error performing batch get: {}", e);
                return Err(e);
            }
        };

        let mut results_vec = vec![];
        for (_key, value) in results.iter() {
            let data: Account = bincode::deserialize(value).map_err(|e| e.to_string())?;
            let data: Account = Account {
                address: data.address,
                balance: BalanceType::Text("Encrypted provide private_key to decrypt".to_string()),
                timestamp: data.timestamp,
            };
            results_vec.push(data);
        }

        Ok(results_vec)
    }

    pub fn total_accounts() -> Result<i64, String> {
        let store = Self::ledger();
        let total = store.get(
            StorageKind::Analytics.name(),
            StorageKind::Account.name().as_bytes(),
        )?;
        let accounts: i64 = bincode::deserialize(&total).map_err(|e| e.to_string())?;

        Ok(accounts)
    }
}
