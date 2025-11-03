use murmur3::murmur3_x64_128_of_slice;

// Thanks: https://www.geeksforgeeks.org/python/bloom-filters-introduction-and-python-implementation/ && https://github.com/Claudenw/BloomFilters/wiki/Bloom-Filters----An-overview

const BLOOM_SIZE: u128 = 128_966;
const BLOOM_HASHES: u128 = 3;

pub struct Data {
    pub values: Vec<String>,
    pub bloom: [u8; BLOOM_SIZE as usize],
}

impl Data {
    pub fn contains(&self, target: &str) -> bool {
        self.values.iter().any(|value| value.contains(target))
    }

    pub fn bloom_contains(&self, target: &str) -> bool {
        let word_hash = murmur3_x64_128_of_slice(target.as_bytes(), 0);
        let step_hash = word_hash as u64 as u128;
        let mut hash = word_hash >> 64;
        for _ in 0..BLOOM_HASHES {
            hash += step_hash * BLOOM_HASHES;

            if self.bloom[(hash % BLOOM_SIZE) as usize] == 0 {
                return false;
            }
        }

        true
    }

    pub fn compute(&mut self) {
        self.values.iter().for_each(|value| {
            value.split_whitespace().for_each(|word| {
                let word_hash = murmur3_x64_128_of_slice(word.as_bytes(), 0);
                let step_hash = word_hash as u64 as u128;
                let mut hash = word_hash >> 64;
                for _ in 0..BLOOM_HASHES {
                    hash += step_hash * BLOOM_HASHES;

                    self.bloom[(hash % BLOOM_SIZE) as usize] = 1;
                }
            });
        });
    }
}
