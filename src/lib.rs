use ahash::AHashMap;
use bytes::{BufMut, BytesMut};
use lz4_flex::compress_prepend_size;
use murmur3::murmur3_x64_128_of_slice;
use std::{
    collections::hash_map::Entry,
    fmt::Debug,
    fs::{self, OpenOptions},
    io::Write,
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
    indexes: AHashMap<String, Vec<usize>>,
    bloom: [u128; usize::div_ceil(BLOOM_SIZE as usize, 128)],
}

impl Block {
    fn insert(&mut self, value: String) {
        let normalized_value: String = value
            .chars()
            .filter(|c| !c.is_ascii_punctuation())
            .collect();

        let index = self.values.len();
        normalized_value.split_ascii_whitespace().for_each(|word| {
            self.bloom_insert(word);
            self.index_insert(word.to_string(), index);
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

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    fn index_insert(&mut self, target: String, value: usize) {
        match self.indexes.entry(target) {
            Entry::Occupied(mut entry) => {
                let values = entry.get_mut();
                if values.binary_search(&value).is_err() {
                    values.push(value);
                    values.sort_unstable();
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![value]);
            }
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            values: Vec::with_capacity(BLOCK_SIZE),
            indexes: Default::default(),
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
        if let Some(block) = self.blocks.last_mut()
            && block.values.len() != BLOCK_SIZE
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
            .flat_map(|block| {
                words
                    .iter()
                    .flat_map(|word| {
                        if let Some(indexes) = block.indexes.get(*word) {
                            indexes
                                .iter()
                                .map(|index| block.values.get(*index).unwrap())
                                .collect()
                        } else {
                            Vec::new()
                        }
                    })
                    .collect::<Vec<&String>>()
            })
            .collect()
    }

    pub fn save(&self) {
        fs::create_dir_all("blocks").unwrap(); // TODO: Remove unwrap

        self.blocks.iter().enumerate().for_each(|(index, block)| {
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(format!("blocks/{index}.zeta"))
                .unwrap(); // TODO: Remove unwrap

            let mut buffer = BytesMut::new();
            buffer.put_u16(block.values.len() as u16);

            block.values.iter().for_each(|value| {
                buffer.put_u64(value.len() as u64);
                buffer.put(&compress_prepend_size(value.as_bytes())[..]);
            });

            file.write_all(&buffer).unwrap(); // TODO: Remove unwrap
            file.flush().unwrap(); // TODO: Remove unwrap
        });
    }
}
