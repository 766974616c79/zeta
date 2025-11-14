use ahash::AHashMap;
use murmur3::murmur3_x64_128_of_slice;
use std::{
    collections::hash_map::Entry,
    fmt::Debug,
    fs::{self, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Write},
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
    loaded: bool,
}

impl Block {
    fn load(&mut self, index: usize) -> Result<(), io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .open(format!("blocks/{index}.zeta"))?;

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
            loaded: false,
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
            .enumerate()
            .map(|(index, block)| {
                if !block.loaded {
                    block.loaded = true;
                    block.load(index).unwrap(); // TODO: remove unwrap
                }

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
        let mut buffer = BufReader::new(file);
        let mut blocks_len_buffer = [0; 8];
        buffer.read_exact(&mut blocks_len_buffer)?;

        for _ in 0..usize::from_le_bytes(blocks_len_buffer) {
            let mut bloom = [0; usize::div_ceil(BLOOM_SIZE as usize, 128)];
            for i in 0..usize::div_ceil(BLOOM_SIZE as usize, 128) {
                let mut bitset_buffer = [0; 16];
                buffer.read_exact(&mut bitset_buffer)?;

                bloom[i] = u128::from_le_bytes(bitset_buffer);
            }

            let block = Block {
                values: Vec::new(),
                indexes: Default::default(),
                bloom,
                loaded: false,
            };

            self.blocks.push(block);
        }

        Ok(())
    }

    fn load_indexes(&mut self) -> Result<(), io::Error> {
        let file = OpenOptions::new().read(true).open("indexes.zeta")?;
        let mut buffer = lz4_flex::frame::FrameDecoder::new(file);
        let mut blocks_len_buffer = [0; 8];
        buffer.read_exact(&mut blocks_len_buffer)?;

        for i in 0..usize::from_le_bytes(blocks_len_buffer) {
            if let Some(block) = self.blocks.get_mut(i) {
                let mut indexes_len_buffer = [0; 8];
                buffer.read_exact(&mut indexes_len_buffer)?;

                for _ in 0..usize::from_le_bytes(indexes_len_buffer) {
                    let mut word_len_buffer = [0; 8];
                    buffer.read_exact(&mut word_len_buffer)?;

                    let word_len = usize::from_le_bytes(word_len_buffer);
                    let mut word_buffer = vec![0; word_len];
                    buffer.read_exact(&mut word_buffer)?;

                    let word = String::from_utf8(word_buffer).unwrap();
                    let mut indexes_len_buffer = [0; 8];
                    buffer.read_exact(&mut indexes_len_buffer)?;

                    let mut indexes = Vec::new();
                    for _ in 0..usize::from_le_bytes(indexes_len_buffer) {
                        let mut index_buffer = [0; 8];
                        buffer.read_exact(&mut index_buffer)?;

                        let index = usize::from_le_bytes(index_buffer);
                        indexes.push(index);
                    }

                    block.indexes.insert(word, indexes);
                }
            }
        }

        Ok(())
    }

    pub fn load(&mut self) -> Result<(), io::Error> {
        self.load_bloom()?;
        self.load_indexes()
    }

    fn save_bloom(&self) -> Result<(), io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open("bloom.zeta")?;

        let mut buffer = BufWriter::new(file);
        buffer.write_all(&self.blocks.len().to_le_bytes())?;

        self.blocks.iter().try_for_each(|block| {
            block
                .bloom
                .iter()
                .try_for_each(|bitset| buffer.write_all(&bitset.to_le_bytes()))
        })?;

        buffer.flush()
    }

    fn save_indexes(&self) -> Result<(), io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open("indexes.zeta")?;

        let mut buffer = lz4_flex::frame::FrameEncoder::new(file);
        buffer.write_all(&self.blocks.len().to_le_bytes())?;

        self.blocks.iter().try_for_each(|block| {
            buffer.write_all(&block.indexes.len().to_le_bytes())?; // TODO: 2**64 is big

            block.indexes.iter().try_for_each(|(word, indexes)| {
                buffer.write_all(&word.len().to_le_bytes())?; // TODO: 2**64 is big
                buffer.write_all(word.as_bytes())?;
                buffer.write_all(&indexes.len().to_le_bytes())?;

                indexes
                    .iter()
                    .try_for_each(|index| buffer.write_all(&index.to_le_bytes()))
            })
        })?;

        buffer.flush()
    }

    pub fn save(&self) -> Result<(), io::Error> {
        fs::create_dir_all("blocks")?;

        self.save_bloom()?;
        self.save_indexes()?;

        self.blocks
            .iter()
            .enumerate()
            .try_for_each(|(index, block)| {
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(format!("blocks/{index}.zeta"))?;

                let mut buffer = lz4_flex::frame::FrameEncoder::new(file);
                buffer.write_all(&block.values.len().to_le_bytes())?; // TODO: 2**64 is big

                block.values.iter().try_for_each(|value| {
                    buffer.write_all(&value.len().to_le_bytes())?; // TODO: 2**64 is big
                    buffer.write_all(value.as_bytes())
                })?;

                buffer.flush()
            })
    }
}
