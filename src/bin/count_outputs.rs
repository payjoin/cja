extern crate coinjoin_analyzer;
use coinjoin_analyzer::{BlockFileIterator};

use std::env::args;


fn main() {
    let mut num_outputs = 0u64;
    let num_files = (args().len() - 1) as f64;
    let mut current_file = 1f64;
    for file in args().skip(1) {
        let iter = match BlockFileIterator::open(file) {
            Ok(i) => i,
            Err(_) => panic!("Could not read file")
        };
        println!("{:.0}%", current_file / num_files * 100f64);
        for block in iter {
            for transaction in block.transactions.iter() {
                for output in transaction.outputs.iter() {
                    if output.pk_script.len() > 0 {
                        num_outputs += 1
                    }
                }
            }
        }
        current_file += 1f64;
    }

    println!("There are {} outputs currently in the blockchain", num_outputs)
}
