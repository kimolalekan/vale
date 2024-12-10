use crate::account::Account;
use crate::store::{Storage, StorageKind};
use crate::tx::TransactionStatus;
use crate::util::config;
use crate::vault::Crypto;
use blake3::Hasher;
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EncryptData {
    Plain(String),
    Vector(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionData {
    sender: String,
    receiver: String,
    amount: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionPrimitive {
    Plain(TransactionData),
    Encrypt(EncryptData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlainTransaction {
    id: String,
    sender: String,
    receiver: String,
    amount: f64,
    fee: f64,
    size: f64,
    timestamp: u64,
    narration: String,
    status: String,
    tx_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncryptedTransaction {
    id: String,
    sender_data: TransactionPrimitive,
    receiver_data: TransactionPrimitive,
    fee: f64,
    size: f64,
    timestamp: u64,
    narration: String,
    status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Transaction {
    Plain(PlainTransaction),
    Encrypted(EncryptedTransaction),
}

impl Transaction {
    fn ledger() -> Storage {
        Storage::init().unwrap()
    }

    pub fn init(sender: String, receiver: String, amount: f64, narration: String) -> Self {
        let mut hasher = Hasher::new();
        let timestamp = Utc::now().timestamp() as u64;
        hasher.update(&timestamp.to_be_bytes());
        let id = hasher.finalize().to_hex().to_string();
        let id: String = hex::encode(id);

        let size = Self::calculate_size_in_byte(
            &id,
            &sender,
            &receiver,
            amount,
            narration.clone(),
            timestamp,
        );
        let fee = Self::calculate_dynamic_fee(size);

        let status = TransactionStatus::as_str(&TransactionStatus::Pending);
        let status = status.to_string();

        let transaction = PlainTransaction {
            id,
            sender,
            receiver,
            amount,
            fee,
            size,
            timestamp,
            narration,
            status,
            tx_key: None,
        };

        Transaction::Plain(transaction)
    }

    fn calculate_size_in_byte(
        id: &str,
        sender: &str,
        receiver: &str,
        amount: f64,
        narration: String,
        timestamp: u64,
    ) -> f64 {
        let status = TransactionStatus::as_str(&TransactionStatus::Pending);
        let status = status.to_string();

        let temp_transaction = PlainTransaction {
            id: id.to_string(),
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            amount,
            fee: 0.0,
            size: 0.0,
            narration,
            timestamp,
            status,
            tx_key: None,
        };

        let size_bytes = mem::size_of_val(&temp_transaction);
        let size_bytes: u64 = size_bytes as u64;
        let byte = size_bytes * amount as u64;
        byte as f64
    }

    fn calculate_dynamic_fee(size: f64) -> f64 {
        let base_fee_per_byte = config::BASE_FEE_PER_BYTE as f64;
        let max_supply = config::MAX_SUPPLY as f64;
        let network_congestion_factor = Self::get_network_congestion_factor();
        let fee = size * base_fee_per_byte * network_congestion_factor / max_supply;
        fee
    }

    fn get_network_congestion_factor() -> f64 {
        let recent_tx_count = Self::get_recent_transaction_count();

        let low_congestion = 500;
        let moderate_congestion = 1000;
        let high_congestion = 2000;

        let congestion_factor = if recent_tx_count <= low_congestion {
            config::LOW_CONGESTION
        } else if recent_tx_count <= moderate_congestion {
            config::MODERATE_CONGESTION
        } else if recent_tx_count <= high_congestion {
            config::HIGH_CONGESTION
        } else {
            config::NORMAL_CONGESTION
        };

        congestion_factor
    }

    fn get_recent_transaction_count() -> u64 {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..10000) // Simulated transaction count (0 to 10000)
    }

    pub fn process_transaction(data: PlainTransaction) -> Result<Transaction, String> {
        let mut _data = data.to_owned();

        let store = Self::ledger();
        let key = bincode::serialize(&data.id).map_err(|e| e.to_string())?;
        let tx_data = TransactionData {
            sender: data.sender,
            amount: data.amount,
            receiver: data.receiver,
        };

        let _tx_data = tx_data.clone();

        let receiver_key = Account::get_account_index(tx_data.sender)?;
        let receiver_data = bincode::serialize(&_tx_data).map_err(|e| e.to_string())?;
        let receiver_data = Crypto::encrypt(receiver_data, Some(receiver_key))
            .map_err(|e| format!("Encryption failed: {}", e))?;
        let receiver_data: TransactionPrimitive =
            TransactionPrimitive::Encrypt(EncryptData::Vector(receiver_data.data));

        let sender_data = bincode::serialize(&_tx_data).map_err(|e| e.to_string())?;
        let sender_data =
            Crypto::encrypt(sender_data, None).map_err(|e| format!("Encryption failed: {}", e))?;
        let tx_key = sender_data.key;
        let sender_data: TransactionPrimitive =
            TransactionPrimitive::Encrypt(EncryptData::Vector(sender_data.data));

        let tx_serialize = EncryptedTransaction {
            id: data.id,
            sender_data,
            receiver_data,
            fee: data.fee,
            size: data.size,
            timestamp: data.timestamp,
            narration: data.narration,
            status: data.status,
        };

        let value = bincode::serialize(&tx_serialize).map_err(|e| e.to_string())?;
        let cf = StorageKind::Transaction.name();
        store.put(cf, &key, &value, true)?;

        _data.tx_key = tx_key;
        let tx = Transaction::Plain(_data);
        Ok(tx)
    }

    pub fn get_transaction(tx_id: String, tx_key: Option<String>) -> Result<Transaction, String> {
        let store = Self::ledger();

        let key = bincode::serialize(&tx_id).map_err(|e| e.to_string())?;
        let cf = StorageKind::Transaction.name();
        let value = store.get(cf, &key)?;

        let encrypted_tx: EncryptedTransaction =
            bincode::deserialize(&value).map_err(|e| e.to_string())?;

        let sender_data = match encrypted_tx.sender_data {
            TransactionPrimitive::Encrypt(EncryptData::Vector(ref encrypted_sender)) => {
                let decrypted_sender = Crypto::decrypt(
                    encrypted_sender.clone(),
                    tx_key.as_deref().unwrap_or_default(),
                )
                .map_err(|e| format!("Sender decryption failed: {}", e))?;
                bincode::deserialize::<TransactionData>(&decrypted_sender.data)
                    .map_err(|e| e.to_string())?
            }
            _ => return Err("Invalid sender data format".to_string()),
        };

        let receiver_data = match encrypted_tx.receiver_data {
            TransactionPrimitive::Encrypt(EncryptData::Vector(ref encrypted_receiver)) => {
                let decrypted_receiver = Crypto::decrypt(
                    encrypted_receiver.clone(),
                    tx_key.as_deref().unwrap_or_default(),
                )
                .map_err(|e| format!("Receiver decryption failed: {}", e))?;
                bincode::deserialize::<TransactionData>(&decrypted_receiver.data)
                    .map_err(|e| e.to_string())?
            }
            _ => return Err("Invalid receiver data format".to_string()),
        };

        let tx = Transaction::Encrypted(EncryptedTransaction {
            id: encrypted_tx.id,
            sender_data: TransactionPrimitive::Plain(sender_data),
            receiver_data: TransactionPrimitive::Plain(receiver_data),
            fee: encrypted_tx.fee,
            size: encrypted_tx.size,
            timestamp: encrypted_tx.timestamp,
            narration: encrypted_tx.narration,
            status: encrypted_tx.status,
        });

        Ok(tx)
    }

    pub fn get_transaction_details(
        tx_id: String,
        tx_key: Option<String>,
    ) -> Result<Transaction, String> {
        let store = Self::ledger();

        let key = bincode::serialize(&tx_id).map_err(|e| e.to_string())?;
        let cf = StorageKind::Transaction.name();
        let value = store.get(cf, &key)?;

        let encrypted_tx: EncryptedTransaction =
            bincode::deserialize(&value).map_err(|e| e.to_string())?;

        let sender_data = match encrypted_tx.sender_data {
            TransactionPrimitive::Encrypt(EncryptData::Vector(ref encrypted_sender)) => {
                let decrypted_sender = Crypto::decrypt(
                    encrypted_sender.clone(),
                    tx_key.as_deref().unwrap_or_default(),
                )
                .map_err(|e| format!("Sender decryption failed: {}", e))?;
                bincode::deserialize::<TransactionData>(&decrypted_sender.data)
                    .map_err(|e| e.to_string())?
            }
            _ => return Err("Invalid sender data format".to_string()),
        };

        let receiver_data = match encrypted_tx.receiver_data {
            TransactionPrimitive::Encrypt(EncryptData::Vector(ref encrypted_receiver)) => {
                let decrypted_receiver = Crypto::decrypt(
                    encrypted_receiver.clone(),
                    tx_key.as_deref().unwrap_or_default(),
                )
                .map_err(|e| format!("Receiver decryption failed: {}", e))?;
                bincode::deserialize::<TransactionData>(&decrypted_receiver.data)
                    .map_err(|e| e.to_string())?
            }
            _ => return Err("Invalid receiver data format".to_string()),
        };

        let tx = Transaction::Encrypted(EncryptedTransaction {
            id: encrypted_tx.id,
            sender_data: TransactionPrimitive::Plain(sender_data),
            receiver_data: TransactionPrimitive::Plain(receiver_data),
            fee: encrypted_tx.fee,
            size: encrypted_tx.size,
            timestamp: encrypted_tx.timestamp,
            narration: encrypted_tx.narration,
            status: encrypted_tx.status,
        });

        Ok(tx)
    }
}

// fn main() {
//     let amount = 1.0;
//     let transaction = Transaction::new(
//         String::from("4uejsjs"),
//         String::from("047917ebb077d0fbd0e48c068052da547e43d346e86ca5fba35bf3a58b0adbe1"),
//         amount,
//     );

//     println!("Transaction: {:?}", transaction);
// }
