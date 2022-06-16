use std::io::{Read,Error};
use std::path::Path;
use std::fs::File;
use std::fmt;
use std::borrow::BorrowMut;
use nom::{le_u8,le_u16,le_u32,le_u64,le_i64,IResult,Needed};

#[derive(Debug)]
pub struct BlockHeader {
    pub version: u32,
    pub previous_block_header_hash: [u8; 32],
    pub merkle_root_hash: [u8; 32],
    pub time: u32,
    pub n_bits: u32,
    pub nonce: u32
}

impl fmt::Display for BlockHeader {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "BlockHeader{{ version: {}, previous_block_header_hash: ", self.version)?;
        for &byte in self.previous_block_header_hash.as_ref() {
            write!(formatter, "{:x}", byte)?;
        }
        write!(formatter, ", merkle_root_hash: ")?;
        for &byte in self.merkle_root_hash.as_ref() {
            write!(formatter, "{:x}", byte)?;
        }
        write!(formatter, ", time: {}, n_bits: {}. nonce: {} }}", self.time, self.n_bits, self.nonce)
    }
}

fn reverse_hash(hash: &[u8; 32]) -> [u8; 32] {
    let mut result: [u8; 32] = [0; 32];
    for i in 0..15 {
        result[i] = hash[31-i];
        result[31-i] = hash[i];
    };
    result
}

named!(pub parse_block_header<&[u8], BlockHeader>,
       do_parse!(
           version: le_u32 >>
               previous_block_header_hash: count_fixed!(u8, le_u8, 32) >>
               merkle_root_hash: count_fixed!(u8, le_u8, 32) >>
               time: le_u32 >>
               n_bits: le_u32 >>
               nonce: le_u32 >>

               (BlockHeader{
                   version: version,
                   previous_block_header_hash: reverse_hash(&previous_block_header_hash),
                   merkle_root_hash: reverse_hash(&merkle_root_hash),
                   time: time,
                   n_bits: n_bits,
                   nonce: nonce
               })
       )
);

#[derive(Debug)]
pub struct Outpoint {
    pub hash: [u8; 32],
    pub index: u32
}

named!(pub parse_outpoint<&[u8], Outpoint>,
       do_parse!(
           hash: count_fixed!(u8, le_u8, 32) >>
               index: le_u32 >>
               (Outpoint{
                   hash: reverse_hash(&hash),
                   index: index
               })
       )
);

#[derive(Debug)]
pub struct TransactionInput {
    pub previous_output: Outpoint,
    pub sequence: u32,
    pub script: Vec<u8>
}

fn parse_compact(input: &[u8]) -> IResult<&[u8], u64> {
    if input.len() < 1 {
        return IResult::Incomplete(Needed::Size(1))
    }
    let rest = &input[1..];
    match input[0] {
        0xff => le_u64(rest),
        0xfe => le_u32(rest).map(|v| v as u64),
        0xfd => le_u16(rest).map(|v| v as u64),
        n => IResult::Done(rest, n as u64)
    }
}

named!(pub parse_transaction_input<&[u8], TransactionInput>,
       do_parse!(
           previous_output: parse_outpoint >>
               script_size: parse_compact >>
               script: take!(script_size) >>
               sequence: le_u32 >>
               (TransactionInput{
                   previous_output: previous_output,
                   script: script.to_vec(),
                   sequence: sequence
               })
       )
);

#[derive(Debug)]
pub struct TransactionOutput {
    pub value: i64,
    pub pk_script: Vec<u8>
}

named!(pub parse_transaction_output<&[u8], TransactionOutput>,
       do_parse!(
           value: le_i64 >>
               script_size: parse_compact >>
               pk_script: take!(script_size) >>
               (TransactionOutput{
                   value: value,
                   pk_script: pk_script.to_vec()
               })
       )
);

#[derive(Debug)]
pub struct Transaction {
    pub version: u32,
    pub lock_time: u32,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>
}

named!(pub parse_transaction<&[u8], Transaction>,
       do_parse!(
           version: le_u32 >>
               tx_in_count: parse_compact >>
               inputs: count!(parse_transaction_input, tx_in_count as usize) >>
               tx_out_count: parse_compact >>
               outputs: count!(parse_transaction_output, tx_out_count as usize) >>
               lock_time: le_u32 >>
               (Transaction{
                   version: version,
                   lock_time: lock_time,
                   inputs: inputs,
                   outputs: outputs
               })
       )
);

#[derive(Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>
}

named!(pub parse_block<&[u8], Block>,
       do_parse!(
               header: parse_block_header >>
               tx_count: parse_compact >>
               transactions: count!(parse_transaction, tx_count as usize) >>

               (Block{
                   header: header,
                   transactions: transactions
               })
       )
);

pub struct BlockFileIterator {
    file: File,
}

impl BlockFileIterator {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<BlockFileIterator, Error> {
        match File::open(path) {
            Ok(file) => Ok(BlockFileIterator{ file: file }),
            Err(err) => Err(err)
        }
    }
}

impl Iterator for BlockFileIterator {
    type Item = Block;

    fn next(&mut self) -> Option<Block> {
        let mut buff = [0u8; 4];
        let n = self.file.read(buff.borrow_mut()).expect("Unable to read magic bytes");
        if n == 0 {
            return None
        }
        if buff != [0xf9, 0xbe, 0xb4, 0xd9] {
            return None
        }
        let _ = self.file.read(buff.borrow_mut()).expect("Unable to read block size");
        let size = le_u32(buff.as_ref()).to_full_result().expect("Unable to convert block size") as usize;
        let mut serialized_block = vec![0u8; size];
        self.file.read(serialized_block.as_mut_slice()).expect("Unable to read block");
        let block = parse_block(serialized_block.as_slice()).to_full_result().expect("Unabled to parse block");
        Some(block)
    }
}
