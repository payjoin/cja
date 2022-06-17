use super::*;
use types::Filter;

#[test]
fn test_is_subset_sum() {
    let mut set = vec![1, 2, 3, 4, 5, 6, 7];
    for subsetsum in SubsetSumIterator::new(&set) {
        assert!(
            is_subset_sum(&set.as_slice(), &subsetsum),
            format!(
                "{} is a subset sum of {:?} but is_subset_sum returned false",
                subsetsum, set
            )
        )
    }
    set = vec![43, 234, 2, 3453, 32, 23432];
    for subsetsum in SubsetSumIterator::new(&set) {
        assert!(
            is_subset_sum(&set.as_slice(), &subsetsum),
            format!(
                "{} is a subset sum of {:?} but is_subset_sum returned false",
                subsetsum, set
            )
        )
    }
    set = vec![0, 23, 434, 4343, 234];
    for subsetsum in SubsetSumIterator::new(&set) {
        assert!(
            is_subset_sum(&set.as_slice(), &subsetsum),
            format!(
                "{} is a subset sum of {:?} but is_subset_sum returned false",
                subsetsum, set
            )
        )
    }
}

#[test]
fn test_subset_sum_iterator() {
    assert_eq!(
        SubsetSumIterator::new(&vec![1, 2, 3]).fold(0, |acc, _| acc + 1),
        2u32.pow(3)
    );
    assert_eq!(
        SubsetSumIterator::new(&vec![1, 2, 3, 4, 5, 6]).fold(0, |acc, _| acc + 1),
        2u32.pow(6)
    );
    assert_eq!(
        SubsetSumIterator::new(&vec![]).collect::<Vec<u64>>(),
        vec![0]
    );
    assert_eq!(
        SubsetSumIterator::new(&vec![1]).collect::<Vec<u64>>(),
        vec![0, 1]
    );
    assert_eq!(
        SubsetSumIterator::new(&vec![6]).collect::<Vec<u64>>(),
        vec![0, 6]
    );
    assert_eq!(
        SubsetSumIterator::new(&vec![1, 2]).collect::<Vec<u64>>(),
        vec![0, 1, 2, 3]
    );
    assert_eq!(
        SubsetSumIterator::new(&vec![1, 2, 3]).collect::<Vec<u64>>(),
        vec![0, 1, 2, 3, 3, 4, 5, 6]
    );
}

#[test]
fn test_subset_sum_set() {
    let mut set = vec![1, 2, 3];
    {
        let subset_sum_set = SubsetSumsFilter::new(&set);
        assert!(subset_sum_set.contains(&1));
        assert!(subset_sum_set.contains(&2));
        assert!(subset_sum_set.contains(&3));
        assert!(subset_sum_set.contains(&4));
        assert!(subset_sum_set.contains(&5));
        assert!(subset_sum_set.contains(&6));
        assert!(!subset_sum_set.contains(&7));
        assert!(!subset_sum_set.contains(&50));
    }
    set = vec![3, 4, 19];
    let subset_sum_set = SubsetSumsFilter::new(&set);
    assert!(subset_sum_set.contains(&3));
    assert!(subset_sum_set.contains(&4));
    assert!(subset_sum_set.contains(&19));
    assert!(subset_sum_set.contains(&7));
    assert!(subset_sum_set.contains(&22));
    assert!(subset_sum_set.contains(&23));
    assert!(subset_sum_set.contains(&26));
    assert!(!subset_sum_set.contains(&1));
    assert!(!subset_sum_set.contains(&21));
}
