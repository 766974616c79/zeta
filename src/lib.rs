// TODO:
// - Better perf
// - More complex example
// - No unwrap

use ahash::{AHashMap, AHashSet};
use murmur3::murmur3_x64_128_of_slice;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::hash_map::Entry,
    fmt::Debug,
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    ops::{BitAnd, BitOrAssign, Shl, Shr},
};
use uuid::Uuid;

// Thanks:
// - https://www.geeksforgeeks.org/python/bloom-filters-introduction-and-python-implementation/
// - https://stackoverflow.com/questions/11954086/which-hash-functions-to-use-in-a-bloom-filter

pub const BLOOM_SIZE: u128 = 128_966;
const BLOOM_HASHES: u128 = 3;
const BLOCK_SIZE: usize = 8_192;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub struct Block {
    uuid: Uuid,
    values: Vec<String>,
    indexes: AHashMap<String, AHashSet<usize>>,
    bloom: [u128; usize::div_ceil(BLOOM_SIZE as usize, 128)],
}

impl Block {
    fn load(&mut self) -> Result<(), io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .open(format!("blocks/{}.block", self.uuid))?;

        let mut buffer = lz4_flex::frame::FrameDecoder::new(file);
        let mut values_len_buffer = [0; 8];
        buffer.read_exact(&mut values_len_buffer)?;

        for _ in 0..usize::from_le_bytes(values_len_buffer) {
            let mut value_len_buffer = [0; 8];
            buffer.read_exact(&mut value_len_buffer)?;

            let value_len = usize::from_le_bytes(value_len_buffer);
            let mut value_buffer = vec![0; value_len];
            buffer.read_exact(&mut value_buffer)?;

            let value = String::from_utf8(value_buffer).unwrap();
            self.values.push(value);
        }

        Ok(())
    }

    pub fn insert(&mut self, value: String) {
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

    fn load_indexes(&mut self) {
        let file = OpenOptions::new()
            .read(true)
            .open(format!("blocks/{}.index", self.uuid))
            .unwrap();

        let mut buffer = lz4_flex::frame::FrameDecoder::new(file);
        let mut len_buffer = [0; 8];
        buffer.read_exact(&mut len_buffer).unwrap();

        let len = usize::from_le_bytes(len_buffer);
        for _ in 0..len {
            let mut word_len_buffer = [0; 8];
            buffer.read_exact(&mut word_len_buffer).unwrap();

            let word_len = usize::from_le_bytes(word_len_buffer);
            let mut word_buffer = vec![0; word_len];
            buffer.read_exact(&mut word_buffer).unwrap();

            let word = String::from_utf8(word_buffer).unwrap();
            let mut indexes_len_buffer = [0; 8];
            buffer.read_exact(&mut indexes_len_buffer).unwrap();

            let indexes_len = usize::from_le_bytes(indexes_len_buffer);
            for _ in 0..indexes_len {
                let mut index_buffer = [0; 8];
                buffer.read_exact(&mut index_buffer).unwrap();

                let index = usize::from_le_bytes(index_buffer);
                self.index_insert(word.clone(), index);
            }
        }
    }

    pub fn save_indexes(&self) {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(format!("blocks/{}.index", self.uuid))
            .unwrap();

        let mut buffer = lz4_flex::frame::FrameEncoder::new(file);
        buffer.write_all(&self.indexes.len().to_le_bytes()).unwrap();

        self.indexes.iter().for_each(|(word, indexes)| {
            buffer.write_all(&word.len().to_le_bytes()).unwrap();
            buffer.write_all(word.as_bytes()).unwrap();
            buffer.write_all(&indexes.len().to_le_bytes()).unwrap();

            indexes.iter().for_each(|index| {
                buffer.write_all(&index.to_le_bytes()).unwrap();
            });
        });
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
                values.insert(value);
            }
            Entry::Vacant(entry) => {
                let mut values: AHashSet<usize> = Default::default();
                values.insert(value);

                entry.insert(values);
            }
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
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

pub struct Database {
    blocks: Vec<Block>,
}

impl Default for Database {
    fn default() -> Self {
        _ = fs::create_dir_all("blocks");

        Self {
            blocks: Default::default(),
        }
    }
}

impl Database {
    pub fn insert(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn get(&mut self, value: &str) -> Vec<&String> {
        let normalized_value: String = value
            .chars()
            .filter(|c| !c.is_ascii_punctuation())
            .collect();

        let words: Vec<&str> = normalized_value.split_ascii_whitespace().collect();
        self.blocks
            .iter_mut()
            .rev()
            .filter(|block| words.iter().all(|word| block.bloom_contains(word)))
            .map(|block| {
                block.load_indexes();
                block.load().unwrap();
                block
            })
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

    fn load_bloom(&mut self) -> Result<(), io::Error> {
        let file = OpenOptions::new().read(true).open("bloom.zeta")?;
        let mut buffer = lz4_flex::frame::FrameDecoder::new(file);
        let mut blocks_len_buffer = [0; 8];
        buffer.read_exact(&mut blocks_len_buffer)?;

        for _ in 0..usize::from_le_bytes(blocks_len_buffer) {
            let mut uuid_buffer = [0; 16];
            buffer.read_exact(&mut uuid_buffer)?;

            let uuid = Uuid::from_bytes(uuid_buffer);
            let mut bloom = [0; usize::div_ceil(BLOOM_SIZE as usize, 128)];
            for i in 0..usize::div_ceil(BLOOM_SIZE as usize, 128) {
                let mut bitset_buffer = [0; 16];
                buffer.read_exact(&mut bitset_buffer)?;

                bloom[i] = u128::from_le_bytes(bitset_buffer);
            }

            let block = Block {
                uuid,
                values: Vec::new(),
                indexes: Default::default(),
                bloom,
            };

            self.blocks.push(block);
        }

        Ok(())
    }

    pub fn load(&mut self) -> Result<(), io::Error> {
        self.load_bloom()
    }

    fn save_blocks(&self) {
        self.blocks.par_iter().for_each(|block| {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(format!("blocks/{}.block", block.uuid))
                .unwrap();

            let mut buffer = lz4_flex::frame::FrameEncoder::new(file);
            buffer.write_all(&block.values.len().to_le_bytes()).unwrap();

            block
                .values
                .iter()
                .try_for_each(|value| {
                    buffer.write_all(&value.len().to_le_bytes())?;
                    buffer.write_all(value.as_bytes())
                })
                .unwrap();
        })
    }

    fn save_indexes(&self) {
        self.blocks.par_iter().for_each(|block| {
            block.save_indexes();
        });
    }

    fn save_bloom(&self) -> Result<(), io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open("bloom.zeta")?;

        let mut buffer = lz4_flex::frame::FrameEncoder::new(file);
        buffer.write_all(&self.blocks.len().to_le_bytes())?;

        self.blocks.iter().try_for_each(|block| {
            buffer.write_all(block.uuid.as_bytes())?;

            block
                .bloom
                .iter()
                .try_for_each(|bitset| buffer.write_all(&bitset.to_le_bytes()))
        })
    }

    pub fn save(&self) -> Result<(), io::Error> {
        rayon::scope(|s| {
            s.spawn(|_| self.save_blocks());
            s.spawn(|_| self.save_indexes());
            s.spawn(|_| self.save_bloom().unwrap());
        });

        Ok(())
    }
}
