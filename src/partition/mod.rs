use types::{Set,Partition,Filter};

#[cfg(test)]
mod test;

enum IterResult<T> {
    End,
    Skip,
    Element(T)
}

pub struct SumFilteredPartitionIterator<'a> {
    set: Set,
    filter: &'a Filter<u64>,
    tuple_iterator: TupleIterator,
    left_set: Option<Set>,
    left_set_sum: u64,
    right_partitions_iterator: Option<Box<SumFilteredPartitionIterator<'a>>>
}

impl<'a> SumFilteredPartitionIterator<'a> {
    pub fn new(set: Set, filter: &'a Filter<u64>) -> SumFilteredPartitionIterator {
        let mut tuple_iterator = TupleIterator::new(set.clone());
        match tuple_iterator.next() {
            None => {
                SumFilteredPartitionIterator {
                    set: set.clone(),
                    filter: filter,
                    tuple_iterator: tuple_iterator,
                    left_set_sum: set.iter().sum(),
                    left_set: Some(set),
                    right_partitions_iterator: None
                }
            },
            Some((left, right)) => {
                SumFilteredPartitionIterator {
                    set: set,
                    filter: filter.clone(),
                    tuple_iterator: tuple_iterator,
                    left_set_sum: left.iter().sum(),
                    left_set: Some(left),
                    right_partitions_iterator: Some(Box::new(SumFilteredPartitionIterator::new(right, filter)))
                }
            }
        }
    }

    fn next(&mut self) -> IterResult<Partition> {
        let left_set = match self.left_set.clone() {
            Some(set) => set,
            None => return IterResult::End
        };
        let sum_contained = {
            self.filter.contains(&self.left_set_sum)
        };
        if ! sum_contained {
            match self.tuple_iterator.next() {
                None => {
                    self.left_set = None;
                    self.right_partitions_iterator = None;
                    let set_sum = self.set.iter().sum();
                    let sum_contained = {
                        self.filter.contains(&set_sum)
                    };
                    return if sum_contained {
                        IterResult::Element(vec![self.set.clone()])
                    } else {
                        IterResult::End
                    }
                },
                Some((left, right)) => {
                    self.left_set = Some(left.clone());
                    self.left_set_sum = left.iter().sum();
                    self.right_partitions_iterator = Some(Box::new(SumFilteredPartitionIterator::new(right.clone(), self.filter.clone())));
                    return IterResult::Skip
                }
            }
        }
        let next = match self.right_partitions_iterator {
            Some(ref mut iter) => iter.next(),
            None => None
        };
        match next {
            Some(mut partition) => {
                partition.push(left_set);
                IterResult::Element(partition)
            },
            None => {
                match self.tuple_iterator.next() {
                    None => {
                        self.left_set = None;
                        self.right_partitions_iterator = None;
                        IterResult::Element(vec![self.set.clone()])
                    },
                    Some((left, right)) => {
                        self.left_set = Some(left.clone());
                        self.left_set_sum = left.iter().sum();
                        self.right_partitions_iterator = Some(Box::new(SumFilteredPartitionIterator::new(right.clone(), self.filter.clone())));
                        IterResult::Skip
                    }
                }
            }
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

pub struct TupleIterator {
    first: u64,
    set: Set,
    current_pattern: u64,
    max_pattern: u64
}

impl TupleIterator {
    fn new(set: Set) -> TupleIterator {
        let first = match set.get(0) {
            Some(v) => v.to_owned(),
            None => 0 as u64
        };
        let max_pattern = match set.len() {
            0 => 0,
            1 => 0,
            v => 2u64.pow(v as u32 - 1) - 1
        };
        TupleIterator {
            first: first,
            set: set,
            current_pattern: 1,
            max_pattern: max_pattern
        }
    }
}

impl Iterator for TupleIterator {
    type Item = (Set, Set);

    fn next(&mut self) -> Option<(Set, Set)> {
        if self.current_pattern > self.max_pattern {
            return None
        };
        let mut left_set = vec![self.first];
        let mut right_set = vec![];

        for (index, element) in self.set.iter().enumerate().skip(1) {
            match (self.current_pattern >> (index - 1)) & 1 == 1 {
                false => left_set.push(*element),
                true => right_set.push(*element)
            }
        }
        self.current_pattern += 1;
        return Some((left_set, right_set));
    }
}
