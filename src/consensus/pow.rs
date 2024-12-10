use crate::blockchain::block::Block;
use crate::blockchain::chain::Blockchain;
use crate::store;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_SUPPLY: u64 = 1_000_000_000; // New max supply for 1 billion tokens
const BLOCK_TIME_SECONDS: u64 = 120; // 2 minutes block time
const DIFFICULTY_WINDOW: usize = 720; // Window size for dynamic difficulty adjustment
const MIN_DIFFICULTY: usize = 1_000; // Minimum difficulty for low network hashrate

/// Struct representing Proof of Work
pub struct ProofOfWork {
    difficulty: usize,
}

impl ProofOfWork {
    pub fn new(initial_difficulty: usize) -> Self {
        ProofOfWork {
            difficulty: initial_difficulty,
        }
    }

    /// Dynamically adjust the difficulty based on the block times in the recent history
    pub fn adjust_difficulty(&mut self, blockchain: &Blockchain) {
        let last_blocks = blockchain.get_last_blocks(DIFFICULTY_WINDOW);

        if last_blocks.len() < DIFFICULTY_WINDOW {
            self.difficulty = MIN_DIFFICULTY;
            return;
        }

        let total_time: u64 = last_blocks
            .windows(2)
            .map(|window| {
                let block_time_diff = window[1].timestamp() - window[0].timestamp();
                block_time_diff
            })
            .sum();

        let avg_block_time = total_time / (last_blocks.len() as u64 - 1);
        let target_time = BLOCK_TIME_SECONDS;

        // Adjust difficulty based on block time:
        if avg_block_time > target_time {
            self.difficulty -=
                (self.difficulty as u64 * (avg_block_time - target_time) / target_time) as usize;
        } else {
            self.difficulty +=
                (self.difficulty as u64 * (target_time - avg_block_time) / target_time) as usize;
        }

        // Ensure difficulty doesn't drop below a minimum level
        if self.difficulty < MIN_DIFFICULTY {
            self.difficulty = MIN_DIFFICULTY;
        }
    }

    /// Validates a block by checking its hash with the required difficulty
    pub fn validate(
        &self,
        block: &Block,
        blockchain: &Blockchain,
        db: &Database,
    ) -> Result<bool, String> {
        let hash = block.hash();
        let target = self.target_difficulty();

        if &hash[..self.difficulty] == target {
            Ok(true)
        } else {
            Err("Invalid proof of work".to_string())
        }
    }

    /// Mines a new block
    pub fn mine_new_block(&self, blockchain: &Blockchain, db: &Database) -> Result<Block, String> {
        if blockchain.total_supply() >= MAX_SUPPLY {
            return Err("Maximum supply reached".to_string());
        }

        let mut block = Block::new();
        let target = self.target_difficulty();
        let mut nonce = 0;

        while !block.hash().starts_with(target) {
            nonce += 1;
            block.set_nonce(nonce);
        }

        block.set_timestamp(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());

        blockchain.add_block(block.clone(), db)?;
        self.adjust_difficulty(blockchain); // Adjust difficulty after the block is mined

        Ok(block)
    }

    fn target_difficulty(&self) -> String {
        "0".repeat(self.difficulty)
    }
}
