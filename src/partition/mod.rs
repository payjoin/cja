use types::{Filter, Partition, Set};

#[cfg(test)]
mod test;

enum IterResult<T> {
    End,
    Skip,
    Element(T),
}

/// Given a set and a filter, enumerate all partitions of the set whose parts
/// match any value in the filter.
pub struct SumFilteredPartitionIterator<'a> {
    set: Set,
    filter: &'a dyn Filter<u64>,
    tuple_iterator: TupleIterator,
    left_set: Option<Set>,
    left_set_sum: u64,
    right_partitions_iterator: Option<Box<SumFilteredPartitionIterator<'a>>>,
}

impl<'a> SumFilteredPartitionIterator<'a> {
    pub fn new(set: Set, filter: &'a dyn Filter<u64>) -> SumFilteredPartitionIterator {
        let mut tuple_iterator = TupleIterator::new(set.clone());
        match tuple_iterator.next() {
            // This case is triggered when `set` is a singleton
            None => SumFilteredPartitionIterator {
                set: set.clone(),
                filter,
                tuple_iterator,
                left_set_sum: set.iter().sum(),
                left_set: Some(set),
                right_partitions_iterator: None,
            },
            // Otherwise, non-trivial partitions of the left set exist, iterate
            // over the cartesian product of the powerset of the left set and a
            // recursion into the sum filtered partitioned iterator of the right
            // set.
            Some((left, right)) => SumFilteredPartitionIterator {
                set,
                filter,
                tuple_iterator,
                left_set_sum: left.iter().sum(),
                left_set: Some(left),
                right_partitions_iterator: Some(Box::new(SumFilteredPartitionIterator::new(
                    right, filter,
                ))),
            },
        }
    }

    /// Complexity is exponential, a bit hard to quantify exactly how much
    /// especially in the average case as opposed to worst case.
    /// This is due to potentially high complexity of filter queries, higher than
    /// expected false positive rate in underlying bloom filters, repeated calls
    /// to filter.contains() on true positives.
    /// Theoretically $O(2^n)$.
    fn next(&mut self) -> IterResult<Partition> {
        let left_set = match self.left_set.clone() {
            Some(set) => set,
            None => return IterResult::End,
        };

        // Depending on the complexity of Filter::contains(), which in some
        // cases is significant, this will multiply it by the number of
        // partitions of the right set since the filter will be re-evaluated
        // wrt the left set repeatedly even as it remains fixed.
        let sum_contained = { self.filter.contains(&self.left_set_sum) };
        if !sum_contained {
            // If the left set doesn't match the fitler, a different
            // subset/complement split is needed.
            match self.tuple_iterator.next() {
                // if the underlying TupleIterator is depleted, the only
                // possibility left is that the entire set matches the filter.
                None => {
                    self.left_set = None;
                    self.right_partitions_iterator = None;
                    let set_sum = self.set.iter().sum();
                    let sum_contained = { self.filter.contains(&set_sum) };
                    return if sum_contained {
                        IterResult::Element(vec![self.set.clone()])
                    } else {
                        IterResult::End
                    };
                }
                // Otherwise, the next subset from the TupleIterator becomes the
                // new left_set, and its complement is recursively iterated
                // through a nested SumFilteredPartitionIterator.
                //
                // This only sets up the next call to `next`, IterResult::Skip
                // is returned unconditionally.
                Some((left, right)) => {
                    self.left_set = Some(left.clone());
                    self.left_set_sum = left.iter().sum();
                    self.right_partitions_iterator = Some(Box::new(
                        SumFilteredPartitionIterator::new(right.clone(), self.filter),
                    ));
                    return IterResult::Skip;
                }
            }
        }

        // Since all sub-cases of the previous conditional return
        // unconditionally, the left set matches the filter.
        let next = match self.right_partitions_iterator {
            Some(ref mut iter) => iter.next(),
            None => None,
        };
        match next {
            // Since the nested partition iterator shares the same filter as
            // this one, all parts of the partition of the complement (right)
            // set match, as does the subset (left set). Extend the partition
            // with the left set as a part, and return the resulting partition
            // of the initial set.
            Some(mut partition) => {
                partition.push(left_set);
                IterResult::Element(partition)
            }
            // Otherwise, advance the tuple iterator to obtain the next left set.
            None => match self.tuple_iterator.next() {
                // If it is depleted, the trivial partition is returned.
                None => {
                    self.left_set = None;
                    self.right_partitions_iterator = None;
                    // FIXME if self.set.sum() is not contained in the filter, it
                    // will still be returned. due to the way sumset filters are
                    // constructed, this is not an issue since if the left set
                    // is in the filter, its complement and their union will
                    // also be in the filter.
                    IterResult::Element(vec![self.set.clone()])
                }
                // If another subset/complement pair is available, create a new
                // nested SumFilteredPartitionIterator for the complement set to be
                Some((left, right)) => {
                    self.left_set = Some(left.clone());
                    self.left_set_sum = left.iter().sum();
                    self.right_partitions_iterator = Some(Box::new(
                        SumFilteredPartitionIterator::new(right.clone(), self.filter),
                    ));
                    IterResult::Skip
                }
            },
        }
    }
}

impl<'a> Iterator for SumFilteredPartitionIterator<'a> {
    type Item = Partition;

    fn next(&mut self) -> Option<Partition> {
        loop {
            match self.next() {
                IterResult::Element(p) => return Some(p),
                IterResult::End => return None,
                IterResult::Skip => {}
            }
        }
    }
}

/// Enumerates all 2-partitions (all pairs of a non-empty proper subset and its
/// complement, distinct up to equality of unordered pairs) of a Set.
///
/// The maximum size of the set is technically 64, practically limited by
/// running time which is $O(2^n)$.
pub struct TupleIterator {
    first: u64,
    set: Set,
    current_pattern: u64,
    max_pattern: u64,
}

impl TupleIterator {
    fn new(set: Set) -> TupleIterator {
        // The powerset is indexed by a u64 used as a bit vector, so sizes
        // larger than 64 are not supported.
        assert!(set.len() <= 64);

        let first = match set.first() {
            Some(v) => v.to_owned(),
            None => 0_u64,
        };
        let max_pattern = match set.len() {
            0 => 0,
            1 => 0,
            v => 2u64.pow(v as u32 - 1) - 1,
        };
        TupleIterator {
            first,
            set,
            current_pattern: 1,
            max_pattern,
        }
    }
}

impl Iterator for TupleIterator {
    type Item = (Set, Set);

    fn next(&mut self) -> Option<(Set, Set)> {
        if self.current_pattern > self.max_pattern {
            return None;
        };
        let mut left_set = vec![self.first];
        let mut right_set = vec![];

        for (index, element) in self.set.iter().enumerate().skip(1) {
            match (self.current_pattern >> (index - 1)) & 1 == 1 {
                false => left_set.push(*element),
                true => right_set.push(*element),
            }
        }
        self.current_pattern += 1;
        Some((left_set, right_set))
    }
}
