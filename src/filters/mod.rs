extern crate bloom;
use self::bloom::BloomFilter;
extern crate num;
use self::num::bigint::BigUint;
use self::num::traits::{Zero, One};

use std::u32;

use types::{Set,Partition,Filter};

#[cfg(test)]
mod test;

pub fn is_subset_sum(set: &[u64], sum: &u64) -> bool {
    if sum == &0u64 {
        return true;
    }
    if set.len() < 1 {
        return false;
    }
    if set.len() == 1 {
        return &set[0] == sum;
    };
    let head = &set[0];
    if head == sum {
        return true;
    }
    let tail = &set[1..];
    let tail_sum: &u64 = &tail.iter().sum();
    if head > sum {
        if tail_sum < sum {
            return false
        }
        return is_subset_sum(tail, sum);
    }
    let remaining_sum: &u64 = &(sum - head);
    if tail_sum == sum || tail_sum == remaining_sum {
        return true;
    };
    if tail_sum < remaining_sum {
        return false;
    };
    is_subset_sum(tail, remaining_sum) || (tail_sum > sum && is_subset_sum(tail, sum))
}

pub struct SubsetSumIterator<'a> {
    set: &'a Set,
    set_size: u64,
    power_set_size: BigUint,
    pattern: BigUint
}

impl<'a> SubsetSumIterator<'a> {
    fn new(set: &'a Set) -> SubsetSumIterator {
        let set_size = set.len();
        SubsetSumIterator {
            set: set,
            set_size: set_size as u64,
            power_set_size: BigUint::one() << set_size,
            pattern: Zero::zero()
        }
    }
}

impl<'a> Iterator for SubsetSumIterator<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if self.pattern >= self.power_set_size {
            return None
        };
        let sum = (0..self.set_size).fold(0, |acc, i| {
            let i: usize = i as usize;
            let mask = BigUint::one() << i;
            if (&self.pattern & mask).is_zero() {
                acc
            } else {
                match self.set.get(i) {
                    Some(value) => acc + value,
                    None => acc
                }
            }
        });
        self.pattern = &self.pattern + BigUint::one();
        Some(sum)
    }
}


pub struct SubsetSumsFilter<'a> {
    set: &'a Set,
    bloom_filter: BloomFilter,
}

impl<'a> SubsetSumsFilter<'a> {
    pub fn new(set: &'a Set) -> SubsetSumsFilter {
        let mut filter = if set.len() as u32 > u32::MAX {
            BloomFilter::with_rate(0.01, u32::MAX)
        } else {
            BloomFilter::with_rate(0.01, set.len() as u32)
        };
        for element in SubsetSumIterator::new(set) {
            filter.insert(&element)
        }
        SubsetSumsFilter {
            set: set,
            bloom_filter: filter,
        }
    }
}

impl<'a> Filter<u64> for SubsetSumsFilter<'a> {

    fn contains(&self, sum: &u64) -> bool {
        match self.bloom_filter.contains(&sum) {
            false => false,
            true => is_subset_sum(self.set.as_slice(), sum)
        }
    }
}

pub struct PartitionsSubsetSumsFilter<'a> {
    partitions: &'a Vec<Partition>,
    bloom_filter: BloomFilter,
}

impl<'a> PartitionsSubsetSumsFilter<'a> {
    pub fn new(partitions: &'a Vec<Partition>) -> PartitionsSubsetSumsFilter {
        let coins = match partitions.first() {
            Some(partition) => partition.iter().flat_map(|set| set.iter() ).count() as u32,
            None => 0
        };
        let mut filter = BloomFilter::with_rate(0.01, coins / 2);
        for partition in partitions {
            for set in partition {
                filter.insert(&set.iter().sum::<u64>().clone());
            };
        };
        PartitionsSubsetSumsFilter {
            partitions: partitions,
            bloom_filter: filter,
        }
    }
}

impl<'a> Filter<u64> for PartitionsSubsetSumsFilter<'a> {

    fn contains(&self, sum: &u64) -> bool {
        if !self.bloom_filter.contains(&sum) {
            return false
        }
        for partition in self.partitions {
            for set in partition {
                if sum == &set.iter().sum::<u64>() {
                    return true;
                };
            };
        };
        return false
    }
}
