use serde_json;

use super::*;
use filters::{PartitionsSubsetSumsFilter, SubsetSumsFilter};
use types::{Partition, Run, Set};

#[test]
fn test_sum_filtered_partition_iterator() {
    let set = vec![3, 4, 19];
    let subsetsum = &SubsetSumsFilter::new(&set);
    let iter = SumFilteredPartitionIterator::new(vec![1, 3, 18], subsetsum);
    assert_eq!(
        iter.collect::<Vec<Partition>>(),
        vec![vec![vec![3], vec![1, 18]], vec![vec![1, 3, 18]]]
    );
}

#[test]
fn regression_test_sum_filtered_partition_iterator() {
    let none_shuffled = include_str!("result-none-t-3-s-2-r-1.json");
    let output_shuffled = include_str!("result-output-t-3-s-2-r-1.json");
    let input_shuffled = include_str!("result-input-t-3-s-2-r-1.json");
    let test_files = vec![none_shuffled, output_shuffled, input_shuffled];
    let mut counter: u64 = 0;
    for file in test_files {
        let run: Run = serde_json::from_str(file).expect("Invalid json in input");
        let in_coins = run.in_coins;
        let out_coins = run.out_coins;
        let in_partitions: Vec<Partition> = {
            SumFilteredPartitionIterator::new(in_coins.clone(), &SubsetSumsFilter::new(&out_coins))
                .collect()
        };
        let out_partitions: Vec<Partition> = {
            SumFilteredPartitionIterator::new(
                out_coins.clone(),
                &PartitionsSubsetSumsFilter::new(&in_partitions),
            )
            .collect()
        };
        let mut partition_tuples: Vec<(Partition, Partition)> = Vec::new();
        for in_partition in in_partitions {
            for out_partition in out_partitions.clone() {
                if partitions_match(&in_partition, &out_partition) {
                    partition_tuples.push((in_partition.clone(), out_partition.clone()));
                }
            }
        }
        for (in_partition, out_partition) in run.partition_tuples {
            counter += 1;
            assert!(
                partition_tuples
                    .iter()
                    .any(|(in_p, out_p)| partition_eq(&in_partition, in_p)
                        && partition_eq(&out_partition, out_p)),
                "{}", "For expected mapping {:?} {:?} no mapping was generated."
            )
        }
    }
    assert_eq!(counter, 30);
}

fn partitions_match(a: &Partition, b: &Partition) -> bool {
    'outer: for set_a in a {
        for set_b in b {
            if set_a.iter().sum::<u64>() == set_b.iter().sum::<u64>() {
                continue 'outer;
            }
        }
        return false;
    }
    true
}

fn partition_eq(part_a: &Partition, part_b: &Partition) -> bool {
    part_a
        .iter()
        .all(|set_a| part_b.iter().any(|set_b| set_eq(set_a, set_b)))
}

fn set_eq(set_a: &Set, set_b: &Set) -> bool {
    set_a
        .iter()
        .all(|element_a| set_b.iter().any(|element_b| element_a == element_b))
}

#[test]
fn test_tuple_iterator() {
    assert_eq!(
        TupleIterator::new(vec![1]).collect::<Vec<(Set, Set)>>(),
        vec![]
    );
    assert_eq!(
        TupleIterator::new(vec![1, 2]).collect::<Vec<(Set, Set)>>(),
        vec![(vec![1], vec![2])]
    );
    assert_eq!(
        TupleIterator::new(vec![1, 2, 3]).collect::<Vec<(Set, Set)>>(),
        vec![
            (vec![1, 3], vec![2]),
            (vec![1, 2], vec![3]),
            (vec![1], vec![2, 3])
        ]
    )
}
