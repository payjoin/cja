extern crate serde_json;

extern crate coinjoin_analyzer;
use coinjoin_analyzer::{Partition, Run};

use std::io;
use std::io::Read;

fn main() {
    let mut input = String::new();
    match io::stdin().read_to_string(&mut input) {
        Ok(_) => (),
        Err(error) => return print!("Error while reading file: {}", error),
    };
    let result: Vec<Run> = serde_json::from_str(input.as_str()).expect("Invalid json in input");
    print!("num_transactions\tnum_inputs_per_transaction");
    print!("\tduration_ms\tnum_outputs\tnon_derived_mappings");
    print!(
        "\tinput_output_zeros\tinput_output_ones\tinput_output_average_other\tinput_output_average"
    );
    print!("\tinput_input_zeros\tinput_input_ones\tinput_input_average_other\tinput_input_average");
    print!("\toutput_output_zeros\toutput_output_ones\toutput_output_average_other\toutput_output_average");
    println!("");
    for run in result {
        let non_derived_partitions = filter_derived_partitions(&run.partition_tuples);
        print!(
            "{}\t{}\t{}\t{}\t{}",
            run.num_transactions,
            run.num_inputs_per_transaction,
            (run.duration_secs * 1_000) as f64 + run.duration_nano as f64 / 1_000_000f64,
            run.out_coins.len(),
            non_derived_partitions.len()
        );
        {
            let (zeros, ones, average_other, average) = aggregated_in_out_probability(
                &run.in_coins,
                &run.out_coins,
                &non_derived_partitions,
            );
            print!(
                "\t{}\t{}\t{:.3}\t{:.3}",
                zeros, ones, average_other, average
            );
        };
        {
            let (zeros, ones, average_other, average) =
                aggregated_in_in_probability(&run.in_coins, &non_derived_partitions);
            print!(
                "\t{}\t{}\t{:.3}\t{:.3}",
                zeros, ones, average_other, average
            );
        };
        {
            let (zeros, ones, average_other, average) =
                aggregated_out_out_probability(&run.out_coins, &non_derived_partitions);
            print!(
                "\t{}\t{}\t{:.3}\t{:.3}",
                zeros, ones, average_other, average
            );
        };
        println!("")
    }
}

fn average(v: &Vec<f64>) -> f64 {
    match v.len() {
        0 => 0f64,
        _ => v.iter().sum::<f64>() / v.len() as f64,
    }
}

fn filter_derived_partitions(
    partitions: &Vec<(Partition, Partition)>,
) -> Vec<(Partition, Partition)> {
    let max_index = partitions
        .iter()
        .map(|&(ref in_p, _)| in_p.len())
        .max()
        .unwrap();
    let mut sorted_partitions: Vec<Vec<&(Partition, Partition)>> = Vec::with_capacity(max_index);
    for _ in 0..(max_index + 1) {
        sorted_partitions.push(Vec::new());
    }
    for partition in partitions {
        let size = partition.1.len();
        sorted_partitions[size].push(partition);
    }
    let mut non_derived = Vec::new();
    for i in 1..max_index {
        let partitions = &sorted_partitions[i];
        let plus_one_partitions = &sorted_partitions[i + 1];
        'outer: for partition in partitions {
            for plus_one_partition in plus_one_partitions {
                if is_derived(partition, plus_one_partition) {
                    continue 'outer;
                }
            }
            non_derived.push(partition.clone().clone())
        }
    }
    for p in sorted_partitions[max_index].clone() {
        non_derived.push(p.clone());
    }
    non_derived
}

fn is_derived(part: &(Partition, Partition), plus_part: &(Partition, Partition)) -> bool {
    let in_partition = &part.1;
    let mut in_partition_retained: Vec<Vec<u64>> = Vec::new();
    let mut plus_in_partition = plus_part.1.clone();
    assert!(in_partition.len() + 1 == plus_in_partition.len());
    for in_set in in_partition {
        let mtch = {
            plus_in_partition
                .iter()
                .position(|set| in_set.iter().sum::<u64>() == set.iter().sum::<u64>())
        };
        match mtch {
            Some(i) => {
                plus_in_partition.remove(i);
            }
            None => in_partition_retained.push(in_set.clone()),
        }
    }
    if in_partition_retained.len() == 1 && plus_in_partition.len() == 2 {
        return true;
    }
    return false;
}

#[test]
fn test_is_derived() {
    assert!(is_derived(
        &(
            vec![vec![1, 2, 3], vec![3, 4]],
            vec![vec![1, 2, 3], vec![3, 4]]
        ),
        &(
            vec![vec![1, 2], vec![3], vec![3, 4]],
            vec![vec![1, 2], vec![3], vec![3, 4]]
        )
    ));
    assert!(is_derived(
        &(
            vec![vec![2, 2, 3], vec![3, 4]],
            vec![vec![2, 2, 3], vec![3, 4]]
        ),
        &(
            vec![vec![3, 2], vec![2], vec![3, 4]],
            vec![vec![3, 2], vec![2], vec![3, 4]]
        )
    ))
}

fn aggregate_probabilities(probabilities: &Vec<f64>) -> (f64, f64, f64, f64) {
    let zeros = probabilities.iter().filter(|&&p| p == 0f64).count() as f64;
    let ones = probabilities.iter().filter(|&&p| p == 1f64).count() as f64;
    let other: Vec<f64> = probabilities
        .iter()
        .filter(|&&p| p > 0f64 && p < 1f64)
        .map(|&e| e)
        .collect();
    let average_other = average(&other);
    let average = average(probabilities);
    (zeros, ones, average_other, average)
}

fn in_out_probability(
    in_coin: &u64,
    out_coin: &u64,
    partition_tuples: &Vec<(Partition, Partition)>,
) -> f64 {
    partition_tuples
        .iter()
        .filter(|&&(ref in_partition, ref out_partition)| {
            let in_set = match in_partition
                .iter()
                .find(|set| set.iter().find(|&coin| coin == in_coin).is_some())
            {
                Some(set) => set,
                None => panic!("Did not find in coin in partition"),
            };
            let out_set = match out_partition
                .iter()
                .find(|set| set.iter().find(|&coin| coin == out_coin).is_some())
            {
                Some(set) => set,
                None => panic!("Did not find out coin in partition"),
            };
            in_set.iter().sum::<u64>() == out_set.iter().sum::<u64>()
        })
        .count() as f64
        / partition_tuples.len() as f64
}

fn aggregated_in_out_probability(
    in_coins: &Vec<u64>,
    out_coins: &Vec<u64>,
    partition_tuples: &Vec<(Partition, Partition)>,
) -> (f64, f64, f64, f64) {
    let probabilities: Vec<f64> = in_coins
        .iter()
        .flat_map(|in_coin| {
            out_coins
                .iter()
                .map(move |out_coin| in_out_probability(in_coin, out_coin, partition_tuples))
        })
        .collect();
    aggregate_probabilities(&probabilities)
}

fn in_in_probability(
    first_in_coin: &u64,
    second_in_coin: &u64,
    partition_tuples: &Vec<(Partition, Partition)>,
) -> f64 {
    partition_tuples
        .iter()
        .filter(|&&(ref in_partition, _)| {
            in_partition
                .iter()
                .find(|set| {
                    set.iter().find(|&coin| coin == first_in_coin).is_some()
                        && set.iter().find(|&coin| coin == second_in_coin).is_some()
                })
                .is_some()
        })
        .count() as f64
        / partition_tuples.len() as f64
}

fn aggregated_in_in_probability(
    in_coins: &Vec<u64>,
    partition_tuples: &Vec<(Partition, Partition)>,
) -> (f64, f64, f64, f64) {
    let probabilities: Vec<f64> = in_coins
        .iter()
        .enumerate()
        .flat_map(|(i, first_in_coin)| {
            in_coins.iter().skip(i + 1).map(move |second_in_coin| {
                in_in_probability(first_in_coin, second_in_coin, &partition_tuples)
            })
        })
        .collect();
    aggregate_probabilities(&probabilities)
}

fn out_out_probability(
    first_out_coin: &u64,
    second_out_coin: &u64,
    partition_tuples: &Vec<(Partition, Partition)>,
) -> f64 {
    partition_tuples
        .iter()
        .filter(|&&(_, ref out_partition)| {
            out_partition
                .iter()
                .find(|set| {
                    set.iter().find(|&coin| coin == first_out_coin).is_some()
                        && set.iter().find(|&coin| coin == second_out_coin).is_some()
                })
                .is_some()
        })
        .count() as f64
        / partition_tuples.len() as f64
}

fn aggregated_out_out_probability(
    out_coins: &Vec<u64>,
    partition_tuples: &Vec<(Partition, Partition)>,
) -> (f64, f64, f64, f64) {
    let probabilities: Vec<f64> = out_coins
        .iter()
        .enumerate()
        .flat_map(|(i, first_out_coin)| {
            out_coins.iter().skip(i + 1).map(move |second_out_coin| {
                out_out_probability(first_out_coin, second_out_coin, &partition_tuples)
            })
        })
        .collect();
    aggregate_probabilities(&probabilities)
}
