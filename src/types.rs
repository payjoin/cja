pub type Set = Vec<u64>;
pub type Partition = Vec<Set>;
#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub inputs: Set,
    pub outputs: Set,
}

impl Transaction {
    pub fn new(inputs: Set, outputs: Set) -> Transaction {
        Transaction {
            inputs: inputs,
            outputs: outputs,
        }
    }
}

pub trait Filter<T> {
    fn contains(&self, _: &T) -> bool;
}

#[derive(Serialize, Deserialize)]
pub struct Run {
    pub num_transactions: u64,
    pub num_inputs_per_transaction: u64,
    pub original_transactions: Vec<Transaction>,
    pub in_coins: Vec<u64>,
    pub out_coins: Vec<u64>,
    pub partition_tuples: Vec<(Partition, Partition)>,
    pub duration_secs: u64,
    pub duration_nano: u32,
}
