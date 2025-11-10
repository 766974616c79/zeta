use ahash::AHashMap;
use lz4_flex::compress_prepend_size;
use murmur3::murmur3_x64_128_of_slice;
use std::{
    collections::hash_map::Entry,
    fmt::Debug,
    fs::OpenOptions,
    io::{BufReader, BufWriter, Read, Write},
    ops::{BitAnd, BitOrAssign, Shl, Shr},
};

// Thanks:
// - https://www.geeksforgeeks.org/python/bloom-filters-introduction-and-python-implementation/
// - https://stackoverflow.com/questions/11954086/which-hash-functions-to-use-in-a-bloom-filter

pub const BLOOM_SIZE: u128 = 128_966;
const BLOOM_HASHES: u128 = 3;
const BLOCK_SIZE: usize = 8_192;

#[derive(Debug)]
pub struct Block {
    // todo: remove pub
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

/*impl Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block (len = {})", self.values.len())
    }
}*/

#[derive(Default)]
pub struct Database {
    pub blocks: Vec<Block>, // todo: remove pub
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

    pub fn load_indexes(&mut self) {
        let file = OpenOptions::new()
            .read(true)
            .open(format!("indexes.zeta"))
            .unwrap(); // TODO: Remove unwrap

        let mut buffer = BufReader::new(file);
        let mut blocks_len_buffer = [0; 8];
        buffer.read_exact(&mut blocks_len_buffer).unwrap();

        for i in 0..usize::from_le_bytes(blocks_len_buffer) {
            if let Some(block) = self.blocks.get_mut(i) {
                let mut indexes_len_buffer = [0; 8];
                buffer.read_exact(&mut indexes_len_buffer).unwrap();

                for _ in 0..usize::from_le_bytes(indexes_len_buffer) {
                    let mut word_len_buffer = [0; 8];
                    buffer.read_exact(&mut word_len_buffer).unwrap();

                    let word_len = usize::from_le_bytes(word_len_buffer);
                    let mut word_buffer: Vec<u8> = Vec::new();
                    word_buffer.resize_with(word_len, Default::default); // todo: review

                    buffer.read_exact(&mut word_buffer).unwrap();

                    let word = String::from_utf8(word_buffer).unwrap();
                    let mut indexes_len_buffer = [0; 8];
                    buffer.read_exact(&mut indexes_len_buffer).unwrap();

                    for _ in 0..usize::from_le_bytes(indexes_len_buffer) {
                        let mut index_buffer = [0; 8];
                        buffer.read_exact(&mut index_buffer).unwrap();

                        block.index_insert(word.clone(), usize::from_le_bytes(index_buffer));
                    }
                }

                println!("{:?}", block);
            }
        }
    }

    pub fn save_indexes(&self) {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("indexes.zeta"))
            .unwrap(); // TODO: Remove unwrap

        let mut buffer = BufWriter::new(file);
        buffer.write_all(&self.blocks.len().to_le_bytes()).unwrap(); // TODO: Remove unwrap

        self.blocks.iter().for_each(|block| {
            buffer
                .write_all(&block.indexes.len().to_le_bytes())
                .unwrap(); // TODO: 2**64 is big & Remove unwrap

            block.indexes.iter().for_each(|(word, indexes)| {
                buffer.write_all(&word.len().to_le_bytes()).unwrap(); // TODO: 2**64 is big & Remove unwrap
                buffer.write_all(word.as_bytes()).unwrap(); // TODO: Remove unwrap
                buffer.write_all(&indexes.len().to_le_bytes()).unwrap(); // TODO: Remove unwrap

                indexes.iter().for_each(|index| {
                    buffer.write_all(&index.to_le_bytes()).unwrap(); // TODO: Remove unwrap
                });
            });
        });

        buffer.flush().unwrap(); // TODO: Remove unwrap
    }

    /*pub fn save(&self) {
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
    }*/
}
