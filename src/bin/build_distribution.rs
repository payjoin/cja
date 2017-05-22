extern crate coinjoin_analyzer;
use coinjoin_analyzer::{Distribution,BlockFileIterator};
extern crate rmp_serde;
extern crate serde;
use serde::{Serialize};

use std::env::args;
use std::fs::File;
use std::io::Write;


fn main() {
    let max_coin_value = 100_000_000_000;
    let bucket_size = 100;
    let mut buckets: std::collections::BTreeMap<u64, f64> = std::collections::BTreeMap::new();
    let mut out_file = File::create("distribution.bin").expect("Unable to open output file");

    println!("Parsing blocks");
    let num_files = (args().len() - 1) as f64;
    let mut current_file = 1f64;
    for file in args().skip(1) {
        let iter = match BlockFileIterator::open(file) {
            Ok(i) => i,
            Err(_) => panic!("Could not read file")
        };
        print!("{:.0}% ", current_file / num_files * 100f64);
        for block in iter {
            print!(".");
            let _ = std::io::stdout().flush();
            for transaction in block.transactions.iter() {
                for output in transaction.outputs.iter() {
                    if output.value > max_coin_value {
                        continue
                    }
                    let bucket = output.value as u64 / bucket_size;
                    if buckets.contains_key(&bucket) {
                        let mut v = buckets.get_mut(&bucket).expect("Unable to get key which buckets contain");
                         *v = *v + 1f64;
                    } else {
                        buckets.insert(bucket, 1f64);
                    }
                }
            }
        }
        println!("");
        current_file += 1f64;
    }

    println!("Cumulating buckets");
    let mut previous = 0f64;
    for value in buckets.values_mut() {
        *value = *value + previous;
        previous = *value;
    }
    println!("Normalizing buckets");
    for value in buckets.values_mut() {
        *value = *value / previous;
    }
    println!("Writing result");
    let dist = Distribution::new(buckets.iter().map(|(key, value)| (*key * bucket_size, *value)).collect());
    dist.serialize(&mut rmp_serde::Serializer::new(&mut out_file)).unwrap();
}
