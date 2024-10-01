use rand::{random, thread_rng, Open01, Rng};
use types::{Set, Transaction};

#[derive(Serialize, Deserialize)]
pub struct Distribution {
    pub cumulative_normalized: Vec<(u64, f64)>,
}

fn realize_subsum(v: &Vec<u64>, sum: u64) -> Vec<u64> {
    let mut d = sum;
    v.iter()
        .flat_map(|&o| {
            if d == 0 {
                vec![o]
            } else if o <= d {
                d -= o;
                vec![o]
            } else if o > d {
                let r = vec![o - d, d];
                d = 0;
                r
            } else {
                panic!("Universe is collapsing")
            }
        })
        .collect()
}

impl Distribution {
    pub fn new(cumulative_normalized: Vec<(u64, f64)>) -> Distribution {
        Distribution {
            cumulative_normalized,
        }
    }

    pub fn random_coinjoin_transaction(
        &self,
        num_transactions: u64,
        transaction_size: u64,
    ) -> (Vec<Transaction>, Set, Set) {
        let mut transactions: Vec<Transaction> = Vec::new();
        let mut in_coins: Set = self.random_set(transaction_size);
        let mut out_coins: Set = self.output_pair(&in_coins);
        transactions.push(Transaction::new(in_coins.clone(), out_coins.clone()));
        for _ in 1..num_transactions {
            let mut new_in = self.random_set(transaction_size);
            let mut new_out = self.output_pair(&new_in);
            transactions.push(Transaction::new(new_in.clone(), new_out.clone()));
            in_coins.append(&mut new_in);
            out_coins.append(&mut new_out);
        }
        (transactions, in_coins, out_coins)
    }

    pub fn random_coinjoin_transaction_shuffled(
        &self,
        num_transactions: u64,
        transaction_size: u64,
    ) -> (Vec<Transaction>, Set, Set) {
        let mut transactions: Vec<Transaction> = Vec::new();
        let mut in_coins: Set = self.random_set(transaction_size);
        let mut out_coins: Set = self.output_pair(&in_coins);
        transactions.push(Transaction::new(in_coins.clone(), out_coins.clone()));
        for _ in 1..num_transactions {
            let mut new_in = self.random_set(transaction_size);
            let mut new_out = self.output_pair(&new_in);
            transactions.push(Transaction::new(new_in.clone(), new_out.clone()));
            let diff: i64 =
                new_out.iter().sum::<u64>() as i64 - out_coins.iter().sum::<u64>() as i64;
            if diff > 0 {
                new_out = realize_subsum(&new_out, diff as u64)
            } else if diff < 0 {
                out_coins = realize_subsum(&out_coins, -diff as u64)
            };
            in_coins.append(&mut new_in);
            out_coins.append(&mut new_out);
        }
        (transactions, in_coins, out_coins)
    }

    pub fn random_coinjoin_transaction_input_shuffled(
        &self,
        num_transactions: u64,
        transaction_size: u64,
    ) -> (Vec<Transaction>, Set, Set) {
        let mut transactions: Vec<Transaction> = Vec::new();
        let mut in_coins: Set = self.random_set(transaction_size);
        let mut out_coins: Set = self.output_pair(&in_coins);
        transactions.push(Transaction::new(in_coins.clone(), out_coins.clone()));
        for _ in 1..num_transactions {
            let mut new_in = self.random_set(transaction_size);
            let mut new_out = self.output_pair(&new_in);
            transactions.push(Transaction::new(new_in.clone(), new_out.clone()));
            in_coins.append(&mut new_in);
            thread_rng().shuffle(&mut in_coins);
            let mut random_in_sum: u64 = in_coins.iter().take(transaction_size as usize).sum();
            while random_in_sum >= new_out.iter().sum() && random_in_sum >= out_coins.iter().sum() {
                thread_rng().shuffle(&mut in_coins);
                random_in_sum = in_coins.iter().take(transaction_size as usize).sum();
            }
            if random_in_sum < new_out.iter().sum() {
                new_out = realize_subsum(&new_out, random_in_sum)
            } else if random_in_sum < out_coins.iter().sum() {
                out_coins = realize_subsum(&out_coins, random_in_sum)
            };
            out_coins.append(&mut new_out);
        }
        (transactions, in_coins, out_coins)
    }

    pub fn random_coinjoin_transaction_distributed_shuffled(
        &self,
        num_transactions: u64,
        transaction_size: u64,
    ) -> (Vec<Transaction>, Set, Set) {
        let mut transactions: Vec<Transaction> = Vec::new();
        let mut in_coins: Set = Vec::new();
        let mut out_sets: Vec<Set> = Vec::new();
        for _ in 0..num_transactions {
            let mut new_in = self.random_set(transaction_size);
            let new_out = self.output_pair(&new_in);
            transactions.push(Transaction::new(new_in.clone(), new_out.clone()));
            out_sets.push(new_out);
            in_coins.append(&mut new_in);
        }
        let out_coins: Set = out_sets
            .iter()
            .flat_map(|out_set| {
                thread_rng().shuffle(&mut in_coins);
                let mut random_in_sum: u64 = 0;
                let out_sum: u64 = out_set.iter().sum();
                for &coin in in_coins.iter() {
                    if random_in_sum + coin <= out_sum {
                        random_in_sum += coin
                    }
                }
                realize_subsum(out_set, random_in_sum)
            })
            .collect();
        (transactions, in_coins, out_coins)
    }

    fn random_set(&self, n: u64) -> Set {
        (0..n).map(|_| self.random_coin()).collect()
    }

    fn random_coin(&self) -> u64 {
        loop {
            let Open01(rand) = random::<Open01<f64>>();
            let coin = match self
                .cumulative_normalized
                .binary_search_by(|(_, probability)| {
                    probability
                        .partial_cmp(&rand)
                        .expect("Impossible situation")
                }) {
                Ok(i) => {
                    let &(coin, _) = self.cumulative_normalized.get(i).unwrap();
                    coin
                }
                Err(i) => {
                    let &(upper_coin, _) = self.cumulative_normalized.get(i).unwrap();
                    let lower_coin = match i {
                        0 => 0u64,
                        _ => {
                            let &(lower_coin, _) = self.cumulative_normalized.get(i - 1).unwrap();
                            lower_coin
                        }
                    };
                    let diff = upper_coin - lower_coin;
                    lower_coin + (diff as f64 * rand) as u64
                }
            };
            if coin > 0 {
                return coin;
            }
        }
    }

    fn output_pair(&self, s: &Vec<u64>) -> Vec<u64> {
        let sum: u64 = s.iter().sum();
        loop {
            let random_output = self.random_coin();
            if random_output < sum {
                return vec![random_output, sum - random_output];
            }
        }
    }
}
