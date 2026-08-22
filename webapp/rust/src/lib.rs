use crate::{
    environment::{
        AddResult::{self, NotPossible}, PreSorter, SortStrategy,
    }, excel::readXLSSeqs,
};
use serde::Serialize;
use std::io;
use wasm_bindgen::prelude::*;

use js_sys::Uint32Array;
use rand::Rng;

pub mod environment;
mod excel;

// import Javascript's alert method to Rust
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn main() {
    println!("Hello, world!");

    // random sequence

    let mut env: environment::PreSorter =
        PreSorter::new(4, 30, 25, 20, environment::SortStrategy::FirstFitStrategy);

    // let seqs = vec![random_input(10_000)];
    let seqs = readXLSSeqs(
        "/home/robin/code/roularta/palletizer/FW_ Prompt + bestanden/Bundelprofiel TV Film AI.xlsx"
            .to_string(),
    )
    .unwrap();

    let mut steps = 0;

    for seq in seqs {
        for item in seq {
            println!("add {}", item);

            // let result = env.add(item);

            println!("{}", env.stringstate());

            // if let AddResult::NotPossible(_) = result {
            //     println!("STOP after {} steps", steps);

            //     // require enter to continue
            //     let mut buffer = String::new();
            //     let stdin = io::stdin(); // We get `Stdin` here.
            //     stdin.read_line(&mut buffer);

            //     env.reset();
            //     steps = 0;

            //     // break;
            // }
            steps += 1;
        }
    }
}

// fn random_input(n: u32) -> Vec<u32> {
//     let mut result: Vec<u32> = Vec::new();
//     for _ in 1..n {
//         let item = random_integer::random_u32(1, 19);
//         result.push(item);
//     }
//     return result;
// }
//

#[derive(Serialize)]
struct SeqResult {
    pub outputs: Vec<Box<[u32]>>,
    pub steps: u64,
}

pub fn runseq_rs(seq: Vec<u32>, nbuffers: u32,
        maxheight: u32,
        targetheight: u32,
        minheight: u32,
        strategy: SortStrategy,) -> Vec<Box<[u32]>> {
    let mut env: environment::PreSorter =
        PreSorter::new(4, maxheight, targetheight, minheight, strategy,);

    let mut steps = 0;
    let mut outputs: Vec<Box<[u32]>> = Vec::new();

    // let seq = vec![10, 12, 14, 18, 25, 20];

    for item in seq {
        println!("add {}", item);

        if let Some(result) = env.add_wasm2(item){
            outputs.push(result);
            steps += 1;
        }


        println!("{}", env.stringstate());
    }

    loop{
        let out = env.empty_buffers_step();
        if out.iter().sum::<u32>() <= 0{
            break;
        }else{
            outputs.push(out);
        }
    }

    
    outputs.extend(env.empty_buffers());

    println!("done");

    return outputs;
}

#[wasm_bindgen]
pub fn runseq(seq: Vec<u32>, nbuffers: u32,
        maxheight: u32,
        targetheight: u32,
        minheight: u32,
        strategy: SortStrategy,) -> JsValue {
    
    let steps: u64 = 0;
    
    return serde_wasm_bindgen::to_value(&SeqResult{
        outputs: runseq_rs(seq, nbuffers, maxheight, targetheight, minheight, strategy),
        steps
    })
    .unwrap();
}
