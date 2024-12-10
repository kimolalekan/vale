use crate::chain::{Block, BlockHeader};
use crate::tx::Transaction;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        let now = Utc::now().timestamp();

        let genesis_block = Block {
            header: BlockHeader {
                index: 0,
                timestamp: now as i64,
                data: "Genesis Block".to_string(),
                prev_hash: "0".repeat(64),
                hash: String::new(),
                nonce: 0,
                difficulty: 4,
                block_size: 0,
                version: 1,
            },
            transactions: vec![],
        };

        let mut blockchain = Blockchain {
            blocks: vec![genesis_block],
        };

        blockchain.blocks[0].header.hash = blockchain.blocks[0].calculate_hash();
        blockchain
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) {
        let now = Utc::now().timestamp();

        let prev_block = self.blocks.last().unwrap();
        let new_block = Block {
            header: BlockHeader {
                index: prev_block.header.index + 1,
                timestamp: now as i64,
                data: "New Block".to_string(),
                prev_hash: prev_block.header.hash.clone(),
                hash: String::new(),
                nonce: 0,
                difficulty: 4,
                block_size: 0,
                version: 1,
            },
            transactions,
        };

        new_block.header.hash = new_block.calculate_hash();
        self.blocks.push(new_block);
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.blocks.len() {
            let current_block = &self.blocks[i];
            let prev_block = &self.blocks[i - 1];

            if current_block.header.hash != current_block.calculate_hash() {
                return false;
            }

            if current_block.header.prev_hash != prev_block.header.hash {
                return false;
            }
        }
        true
    }
}
