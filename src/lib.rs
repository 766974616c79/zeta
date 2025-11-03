use murmur3::murmur3_x64_128_of_slice;
use std::ops::{BitAnd, BitOrAssign, Shl, Shr};

// Thanks: https://www.geeksforgeeks.org/python/bloom-filters-introduction-and-python-implementation/ && https://github.com/Claudenw/BloomFilters/wiki/Bloom-Filters----An-overview

pub const BLOOM_SIZE: u128 = 128_966;
const BLOOM_HASHES: u128 = 3;

pub struct Data {
    pub values: Vec<String>,
    pub bloom: [u128; usize::div_ceil(BLOOM_SIZE as usize, 128)],
}

impl Data {
    pub fn new(values: Vec<String>) -> Self {
        Self {
            values,
            bloom: [0; usize::div_ceil(BLOOM_SIZE as usize, 128)],
        }
    }

    pub fn contains(&self, target: &str) -> bool {
        self.values.iter().any(|value| value.contains(target))
    }

    pub fn bloom_contains(&self, target: &str) -> bool {
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

    pub fn compute(&mut self) {
        self.values.iter().for_each(|value| {
            value.split_whitespace().for_each(|word| {
                let word_hash = murmur3_x64_128_of_slice(word.as_bytes(), 0);
                let step_hash = word_hash as u64 as u128;
                let mut hash = word_hash.shr(64);
                for _ in 0..BLOOM_HASHES {
                    let normalized_hash = (hash % BLOOM_SIZE) as usize;
                    self.bloom[normalized_hash / 128]
                        .bitor_assign(1u128.shl(normalized_hash % 128));

                    hash += step_hash;
                }
            });
        });
    }
}
