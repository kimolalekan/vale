use crate::tx::Transaction;
use blake3::Hasher;

#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub index: u64,
    pub timestamp: i64,
    pub data: String,
    pub prev_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub difficulty: u64,
    pub block_size: u64,
    pub version: u64,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    fn new(index: u64, transactions: Vec<Transaction>, prev_hash: String) -> Self {
        let timestamp = Utc::now();
        let hash = Block::calculate_hash(index, &timestamp, &transactions, &prev_hash);
        Block {
            index,
            timestamp,
            transactions,
            prev_hash,
            hash,
        }
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Hasher::new();

        hasher.update(self.header.prev_hash.as_bytes());
        hasher.update(self.header.timestamp.to_be_bytes());
        hasher.update(self.header.nonce.to_be_bytes());
        hasher.update(self.header.version.to_be_bytes());

        format!("{:x}", hasher.finalize())
    }
}

// fn calculate_merkle_root(transactions: &[Transaction]) -> String {
//     let mut hasher = Sha256::new();

//     for tx in transactions {
//         hasher.update(tx.sender.as_bytes());
//         hasher.update(tx.receiver.as_bytes());
//         hasher.update(tx.amount.to_be_bytes());
//     }

//     format!("{:x}", hasher.finalize())
// }
