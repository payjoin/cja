extern crate bloom;
use self::bloom::BloomFilter;
extern crate num;
use self::num::bigint::BigUint;
use self::num::traits::{One, Zero};

use std::u32;

use types::{Filter, Partition, Set};

#[cfg(test)]
mod test;

/// Solve decision version of subset sum by brute force, returning true if `sum`
/// can be exactly expressed by summing a subset of `set`. Complexity is
/// $O(2^n)$ in the size of the set.
pub fn is_subset_sum(set: &[u64], sum: &u64) -> bool {
    if sum == &0u64 {
        return true;
    }
    if set.is_empty() {
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
            return false;
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

/// Enumerates the sumset of a given set.
pub struct SubsetSumIterator<'a> {
    set: &'a Set,
    set_size: u64,
    power_set_size: BigUint,
    pattern: BigUint,
}

impl<'a> SubsetSumIterator<'a> {
    fn new(set: &'a Set) -> SubsetSumIterator {
        let set_size = set.len();
        SubsetSumIterator {
            set,
            set_size: set_size as u64,
            power_set_size: BigUint::one() << set_size,
            pattern: Zero::zero(),
        }
    }
}

impl<'a> Iterator for SubsetSumIterator<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if self.pattern >= self.power_set_size {
            return None;
        };
        let sum = (0..self.set_size).fold(0, |acc, i| {
            let i: usize = i as usize;
            let mask = BigUint::one() << i;
            if (&self.pattern & mask).is_zero() {
                acc
            } else {
                match self.set.get(i) {
                    Some(value) => acc + value,
                    None => acc,
                }
            }
        });
        self.pattern = &self.pattern + BigUint::one();
        Some(sum)
    }
}

/// Match sums in the sumset of a given set.
pub struct SubsetSumsFilter<'a> {
    set: &'a Set,
    bloom_filter: BloomFilter,
}

impl<'a> SubsetSumsFilter<'a> {
    /// Initialize the filter. $O(2^n)$ complexity, since the full sumset is
    /// computed in order to construct a bloom filter.
    pub fn new(set: &'a Set) -> SubsetSumsFilter {
        let mut filter = if set.len() as u32 > u32::MAX {
            BloomFilter::with_rate(0.01, u32::MAX)
        } else {
            // note that the false positive rate may be significantly higher,
            // since for small values of set.len() the sumset may be
            // exponentially larger in some cases
            BloomFilter::with_rate(0.01, set.len() as u32)
        };
        for element in SubsetSumIterator::new(set) {
            filter.insert(&element)
        }
        SubsetSumsFilter {
            set,
            bloom_filter: filter,
        }
    }
}

impl<'a> Filter<u64> for SubsetSumsFilter<'a> {
    /// $O(2^n)$ complexity due to re-evaluation of the sumset by call to
    /// `is_subset_sum`, except when the bloom filter excludes a query ($O(1)$).
    fn contains(&self, sum: &u64) -> bool {
        match self.bloom_filter.contains(&sum) {
            false => false,
            true => is_subset_sum(self.set.as_slice(), sum),
        }
    }
}

/// Match sums in the parts of a given set of partitions of a set.
pub struct PartitionsSubsetSumsFilter<'a> {
    partitions: &'a Vec<Partition>,
    bloom_filter: BloomFilter,
}

impl<'a> PartitionsSubsetSumsFilter<'a> {
    pub fn new(partitions: &'a Vec<Partition>) -> PartitionsSubsetSumsFilter {
        let coins = match partitions.first() {
            Some(partition) => partition.iter().flat_map(|set| set.iter()).count() as u32,
            None => 0,
        };
        // here too the false positive rate is potentially higher, as coins is
        // the size of the underlying set, whereas the insertions are elements
        // of the sumset.
        let mut filter = BloomFilter::with_rate(0.01, coins / 2);
        for partition in partitions {
            for set in partition {
                filter.insert(&set.iter().sum::<u64>().clone());
            }
        }
        PartitionsSubsetSumsFilter {
            partitions,
            bloom_filter: filter,
        }
    }
}

impl<'a> Filter<u64> for PartitionsSubsetSumsFilter<'a> {
    /// Complexity is $O(nm)$ where $m$ is the size of the given set of
    /// partitions, and $n$ is the size of the underlying set.
    fn contains(&self, sum: &u64) -> bool {
        if !self.bloom_filter.contains(&sum) {
            return false;
        }
        for partition in self.partitions {
            for set in partition {
                if sum == &set.iter().sum::<u64>() {
                    return true;
                };
            }
        }
        false
    }
}
