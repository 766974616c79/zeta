use murmur3::murmur3_x64_128_of_slice;
use std::{
    fmt::Debug,
    ops::{BitAnd, BitOrAssign, Shl, Shr},
};

// Thanks:
// - https://www.geeksforgeeks.org/python/bloom-filters-introduction-and-python-implementation/
// - https://stackoverflow.com/questions/11954086/which-hash-functions-to-use-in-a-bloom-filter

pub const BLOOM_SIZE: u128 = 128_966;
const BLOOM_HASHES: u128 = 3;
const BLOCK_SIZE: usize = 8_192;

struct Block {
    values: Vec<String>,
    bloom: [u128; usize::div_ceil(BLOOM_SIZE as usize, 128)],
}

impl Block {
    fn insert(&mut self, value: String) {
        let normalized_value: String = value
            .chars()
            .filter(|c| !c.is_ascii_punctuation())
            .collect();

        normalized_value.split_ascii_whitespace().for_each(|word| {
            self.bloom_insert(word);
        });

        self.values.push(value);
    }

    fn bloom_contains(&self, target: &str) -> bool {
        let word_hash = murmur3_x64_128_of_slice(target.as_bytes(), 0);
        let step_hash = word_hash as u64 as u128;
        let mut hash = word_hash.shr(64);
        for _ in 0..BLOOM_HASHES {
            let normalized_hash = (hash % BLOOM_SIZE) as usize;
            let pos = normalized_hash % 128;
            if self.bloom[normalized_hash / 128].shr(pos).bitand(1) == 0 {
                return false;
            }

            hash += step_hash;
        }

        true
    }

    fn bloom_insert(&mut self, target: &str) {
        let word_hash = murmur3_x64_128_of_slice(target.as_bytes(), 0);
        let step_hash = word_hash as u64 as u128;
        let mut hash = word_hash.shr(64);
        for _ in 0..BLOOM_HASHES {
            let normalized_hash = (hash % BLOOM_SIZE) as usize;
            self.bloom[normalized_hash / 128].bitor_assign(1u128.shl(normalized_hash % 128));

            hash += step_hash;
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            values: Vec::with_capacity(BLOCK_SIZE),
            bloom: [0; usize::div_ceil(BLOOM_SIZE as usize, 128)],
        }
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block (len = {})", self.values.len())
    }
}

#[derive(Default)]
pub struct Database {
    blocks: Vec<Block>,
}

impl Database {
    pub fn insert(&mut self, value: String) {
        if let Some(block) = self
            .blocks
            .iter_mut()
            .rev()
            .find(|block| block.values.len() != BLOCK_SIZE)
        {
            block.insert(value);
        } else {
            let mut block = Block::default();
            block.insert(value);

            self.blocks.push(block);
        }
    }

    pub fn get(&self, value: &str) -> Vec<&String> {
        let normalized_value: String = value
            .chars()
            .filter(|c| !c.is_ascii_punctuation())
            .collect();

        let words: Vec<&str> = normalized_value.split_ascii_whitespace().collect();
        self.blocks
            .iter()
            .rev()
            .filter(|block| words.iter().all(|word| block.bloom_contains(word)))
            .flat_map(|block| &block.values)
            .filter(|sentence| words.iter().all(|word| sentence.contains(word)))
            .collect()
    }
}
