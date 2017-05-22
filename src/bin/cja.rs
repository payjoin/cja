extern crate rand;
use std::fs::File;
use std::io::{Write};
use std::time::Instant;
use std::process::exit;

extern crate serde;
use serde::Deserialize;
extern crate serde_json;
extern crate rmp_serde;
use rmp_serde::Deserializer;

extern crate rayon;
use rayon::prelude::*;

#[macro_use(value_t)]
extern crate clap;
use clap::{Arg,ArgMatches,App,SubCommand};

extern crate coinjoin_analyzer;
use coinjoin_analyzer::{Partition,Distribution,SubsetSumsFilter,PartitionsSubsetSumsFilter,SumFilteredPartitionIterator,Run};

fn main() {
    let matches= get_app().get_matches();
    match matches.subcommand() {
        ("auto", Some(options))    => auto(options),
        ("analyze", Some(options)) => analyze(options),
        _                          => { let _ = get_app().print_help(); }
    }
}

fn analyze(options: &ArgMatches) {
    let inputs: Vec<u64> = value_t!(options.value_of("inputs"), String)
        .unwrap_or_else(|e| e.exit())
        .split(",")
        .map(|i| i.parse::<u64>().unwrap_or_else(|e|{ println!("Invalid input value {}: {}", i, e); exit(1) }) )
        .collect();
    let outputs: Vec<u64> = value_t!(options.value_of("outputs"), String)
        .unwrap_or_else(|e| e.exit())
        .split(",")
        .map(|o| o.parse::<u64>().unwrap_or_else(|e|{ println!("Invalid output value {}: {}", o, e); exit(1) }) )
        .collect();
    let in_partitions: Vec<Partition> = {
        SumFilteredPartitionIterator::new(inputs.clone(), &SubsetSumsFilter::new(&outputs)).collect()
    };
    let out_partitions: Vec<Partition> = {
        SumFilteredPartitionIterator::new(outputs.clone(), &PartitionsSubsetSumsFilter::new(&in_partitions)).collect()
    };
    let mut partition_tuples: Vec<(Partition, Partition)> = Vec::new();
    for in_partition in in_partitions {
        for out_partition in out_partitions.clone() {
            if partitions_match(&in_partition, &out_partition) {
                partition_tuples.push((in_partition.clone(), out_partition.clone()));
            }
        }
    }
    for &(ref input_sets, ref output_sets) in partition_tuples.iter() {
        println!("Input sets: {:?} Output sets: {:?}", input_sets, output_sets);
    }
}

fn auto(options: &ArgMatches) {
    let parallelism = value_t!(options.value_of("parallelism"), usize)
        .unwrap_or_else(|e| e.exit());
    let _ = rayon::initialize(rayon::Configuration::new().set_num_threads(parallelism));
    let distribution_file_name = match options.value_of("distribution") {
        Some(string) => string,
        None => return print!("No distribution file given!")
    };
    let distribution = match read_distribution(&distribution_file_name) {
        Ok(dist) => dist,
        Err(err) => return print!("Error while reading distribution: {}\n", err)
    };
    let transactions = value_t!(options.value_of("transactions"), u64)
        .unwrap_or_else(|e| e.exit());
    let transaction_size = value_t!(options.value_of("size"), u64)
        .unwrap_or_else(|e| e.exit());
    let runs = value_t!(options.value_of("runs"), usize)
        .unwrap_or_else(|e| e.exit());
    let shuffled = value_t!(options.value_of("shuffled"), String)
        .unwrap_or_else(|e| e.exit());
    if shuffled != "none" && shuffled != "input" && shuffled != "output" && shuffled != "distributed" {
        return print!("Passed invalid value for shuffled parameter")
    }
    let result_file_name = match options.value_of("output") {
        Some(string) => string.to_string(),
        None => format!("result-{}-t-{}-s-{}-r-{}.json", shuffled, transactions, transaction_size, runs)
    };

    let mut result: Vec<Run> = Vec::new();
    (0 .. runs)
        .into_par_iter()
        .weight_max()
        .map(|_| run(&distribution, transactions, transaction_size, &shuffled) )
        .collect_into(&mut result);

    let mut file = File::create(result_file_name).unwrap();
    let json_string = serde_json::to_string(&result).unwrap();
    let _ = file.write(json_string.as_bytes());
}

fn get_app<'a>() -> App<'a, 'a> {
    App::new("cja")
        .author("Felix Konstantin Maurer <maufl@maufl.de>")
        .about("This program generates and analyses CoinJoin transactions.")
        .version("v0.1")
        .subcommand(SubCommand::with_name("auto")
                    .about("generate and analyze CoinJoin transactions for various parameters")
                    .arg(Arg::with_name("transactions")
                         .short("t")
                         .default_value("4")
                         .takes_value(true))
                    .arg(Arg::with_name("size")
                         .short("s")
                         .default_value("3")
                         .takes_value(true))
                    .arg(Arg::with_name("shuffled")
                         .short("S")
                         .default_value("none")
                         .takes_value(true)
                         .possible_values(&["none", "output", "input", "distributed"]))
                    .arg(Arg::with_name("runs")
                         .short("r")
                         .default_value("5")
                         .takes_value(true))
                    .arg(Arg::with_name("parallelism")
                         .short("p")
                         .default_value("5")
                         .takes_value(true))
                    .arg(Arg::with_name("distribution")
                         .short("d")
                         .default_value("distribution.bin")
                         .takes_value(true))
                    .arg(Arg::with_name("output")
                         .short("o")
                         .takes_value(true))
        )
        .subcommand(SubCommand::with_name("analyze")
                    .about("analyze single CoinJoin transaction for given inputs and outputs ")
                    .arg(Arg::with_name("inputs")
                         .short("i")
                         .takes_value(true))
                    .arg(Arg::with_name("outputs")
                         .short("o")
                         .takes_value(true))
        )
}

fn run(distribution: &Distribution, num_transactions: u64, transaction_size: u64, shuffled: & String) -> Run {
    let (transactions, in_coins, out_coins) = match shuffled.as_ref() {
        "output" => distribution.random_coinjoin_transaction_shuffled(num_transactions, transaction_size),
        "input" => distribution.random_coinjoin_transaction_input_shuffled(num_transactions, transaction_size),
        "distributed" => distribution.random_coinjoin_transaction_distributed_shuffled(num_transactions, transaction_size),
        "none" => distribution.random_coinjoin_transaction(num_transactions, transaction_size),
        _ => panic!("Invalid value for shuffled options")
    };

    let now = Instant::now();
    let in_partitions: Vec<Partition> = {
        SumFilteredPartitionIterator::new(in_coins.clone(), &SubsetSumsFilter::new(&out_coins)).collect()
    };
    let out_partitions: Vec<Partition> = {
        SumFilteredPartitionIterator::new(out_coins.clone(), &PartitionsSubsetSumsFilter::new(&in_partitions)).collect()
    };
    let mut partition_tuples: Vec<(Partition, Partition)> = Vec::new();
    for in_partition in in_partitions {
        for out_partition in out_partitions.clone() {
            if partitions_match(&in_partition, &out_partition) {
                partition_tuples.push((in_partition.clone(), out_partition.clone()));
            }
        }
    }
    let duration = now.elapsed();
    Run {
        num_transactions: num_transactions,
        num_inputs_per_transaction: transaction_size,
        original_transactions: transactions,
        in_coins: in_coins,
        out_coins: out_coins,
        partition_tuples: partition_tuples,
        duration_secs: duration.as_secs(),
        duration_nano: duration.subsec_nanos()
    }
}

fn read_distribution(file_name: &str) -> Result<Distribution, String> {
    let file = match File::open(file_name) {
        Ok(file) => file,
        Err(err) => return Err(format!("Error while opening file: {}", err))
    };
    match Deserialize::deserialize(&mut Deserializer::new(file)) {
        Ok(dist) => Ok(dist),
        Err(e) => Err(format!("Could not parse distribution: {}", e))
    }
}

fn partitions_match(a: & Partition, b: & Partition) -> bool {
    'outer: for set_a in a {
        for set_b in b {
            if set_a.iter().sum::<u64>() == set_b.iter().sum::<u64>() {
                continue 'outer
            }
        }
        return false
    }
    true
}

