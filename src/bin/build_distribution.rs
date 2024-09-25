extern crate coinjoin_analyzer;
use coinjoin_analyzer::{BlockFileIterator, Distribution};
extern crate rmp_serde;
extern crate serde;
use serde::Serialize;

use rmp_serde::Serializer;
use std::env::args;
use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let max_coin_value = 100_000_000_000;
    let bucket_size = 100;
    let mut buckets: std::collections::BTreeMap<u64, f64> = std::collections::BTreeMap::new();

    println!("Parsing blocks");
    let num_files = (args().len() - 1) as f64;
    let mut current_file = 1f64;
    for file in args().skip(1) {
        let iter = match BlockFileIterator::open(file) {
            Ok(i) => i,
            Err(_) => panic!("Could not read file"),
        };
        print!("{:.0}% ", current_file / num_files * 100f64);
        for block in iter {
            print!(".");
            let _ = std::io::stdout().flush();
            for transaction in block.transactions.iter() {
                for output in transaction.outputs.iter() {
                    if output.value > max_coin_value {
                        continue;
                    }
                    let bucket = output.value as u64 / bucket_size;
                    if let std::collections::btree_map::Entry::Vacant(e) = buckets.entry(bucket) {
                        e.insert(1f64);
                    } else {
                        let v = buckets
                            .get_mut(&bucket)
                            .expect("Unable to get key which buckets contain");
                        *v += 1f64;
                    }
                }
            }
        }
        println!();
        current_file += 1f64;
    }

    println!("Cumulating buckets");
    let mut previous = 0f64;
    for value in buckets.values_mut() {
        *value += previous;
        previous = *value;
    }
    println!("Normalizing buckets");
    for value in buckets.values_mut() {
        *value /= previous;
    }
    println!("Writing result");
    let dist = Distribution::new(
        buckets
            .iter()
            .map(|(key, value)| (*key * bucket_size, *value))
            .collect(),
    );
    save_to_rmp::<Distribution>(Path::new("distribution.bin"), &dist)
}

/// Save Serializable data to a file as RustMessagePack format.
/// Note if the file already exist. **The existing file is deleted.**
/*
pub async fn save_to<T>(file : &Path, data : &T) -> Result<(), Box<dyn Error>>
    where T : Serialize{
    save_to_rmp(file,data)
}
*/

fn save_to_rmp<T>(file: &Path, data: &T) -> Result<(), Box<dyn Error>>
where
    T: Serialize,
{
    if file.exists() {
        fs::remove_file(file)?;
    }
    let file_handler = OpenOptions::new().write(true).create_new(true).open(file)?;

    let mut buf = Vec::new();
    data.serialize(&mut Serializer::new(&mut buf))?;
    let mut buf_writer = BufWriter::new(&file_handler);
    buf_writer.write_all(buf.as_slice())?;
    Ok(())
}
