use crate::{
    environment::{
        AddResult::{self, NotPossible},
        PreSorter, SortStrategy, simulate_random,
    },
    excel::readXLSSeqs,
};
use std::io;
use wasm_bindgen::prelude::*;

mod environment;
mod excel;
use rust::runseq_rs;

pub fn main() {
    // simulate_random(1_000_000);
    // simulate_json();
    mytest();
}

pub fn simulate_json() {
    let contents = std::fs::read_to_string("test.json").expect("failed to read test.json");
    let json: serde_json::Value =
        serde_json::from_str(&contents).expect("failed to parse test.json");
    let seqs = json
        .get("seqs")
        .and_then(serde_json::Value::as_array)
        .expect("test.json must contain a `seqs` array");

    for seq in seqs {
        let values: Vec<u32> = seq
            .as_array()
            .expect("each sequence must be an array")
            .iter()
            .map(|value| {
                value
                    .as_u64()
                    .and_then(|value| u32::try_from(value).ok())
                    .expect("sequence values must be u32")
            })
            .collect();
        runseq_rs(
            values,
            4,
            30,
            25,
            20,
            rust::environment::SortStrategy::FirstFitStrategy,
        );
    }
}

pub fn mytest() {
    let values = vec![2, 1, 26, 40, 40, 26, 26, 40, 32];

    let mut env = environment::PreSorter::new(4, 44, 40, 36, SortStrategy::FirstFitStrategy);

    for value in values {
        let outs = env.add_wasm2(value);
        if let Some(out) = outs {
            println!("output: {:?}, buffers: {:?}", out, env.getBufferSizes());
        }
    }
}
