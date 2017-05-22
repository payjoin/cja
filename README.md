# CoinJoin analyzer
This repository contains several CoinJoin related tools.
* `build_distribution` for building coin size distributions from the blockchain
* `cja` for generating and analyzing CoinJoin transactions
* `calculate_probabilities` for post processing the result of `cja`

These are highly specific tools for my needs but maybe they are of help to someone else.
The Rust library also contains a parser for bitcoind `blk*.dat` files, which might be useful.

# Installation
Clone the repository.
I used Git LFS to store a coin size distribution file that is quite large. 
You might therefore need LFS support to checkout the repository.
Then run `cargo build --release` to build the tools.
I used Rust version 1.17.

# Usage
First, if you can't use the coin size distribution file of this repository, you have to build it yourself.
Simply run `build_distribution /dir/to/blockchain/blk*.dat`.

Then you can use `cja` to generate and analyze CoinJoin transactions.
Run `cja auto -t 4 -s 3 -r 10` to generate 10 CoinJoin transactions with 4 sub-transactions each where each sub-transaction has 3 inputs and 2 outputs.
Use the `-S` flag to select one of our output shuffeling algorithms. `cja help auto` will show all flags and their possible values.
By default, `cja` will write the result to a file called `result-{shuffeling-algo}-t-{transactions}-s-{size}-r-{runs}.json`.
The output will contain the original sub-transactions, the resulting CoinJoin transaction and all mappings that where found.

A result file can be further processed with `calculate_probabilities < result-*.json > result-*.tsv`.
It will calculate the average input-output, input-input, and output-output probabilities, using only none derived mappings.
What this exactly means is explained in our paper that will be published later ...

# License

See the License.txt file for license rights and limitations (MIT).
