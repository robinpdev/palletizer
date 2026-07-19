use crate::{
    environment::{
        AddResult::{self, NotPossible},
        PreSorter,
    },
    excel::readXLSSeqs,
};
use std::io;
use wasm_bindgen::prelude::*;

use rand::Rng;

mod environment;
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

            let result = env.add(item);

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

#[wasm_bindgen]
pub fn runseq(seq: Vec<u32>) -> String {
    let mut env: environment::PreSorter =
        PreSorter::new(4, 30, 25, 20, environment::SortStrategy::FirstFitStrategy);

    let mut steps = 0;

    // let seq = vec![10, 12, 14, 18, 25, 20];

    for item in seq {
        println!("add {}", item);

        let result = env.add(item);

        println!("{}", env.stringstate());

        // if let NotPossible(_) = result {
        //     println!("STOP after {} steps", steps);

        //     // // require enter to continue
        //     // let mut buffer = String::new();
        //     // let stdin = io::stdin(); // We get `Stdin` here.
        //     // stdin.read_line(&mut buffer);

        //     env.reset();
        //     steps = 0;

        //     break;
        // }
        steps += 1;
    }

    return env.stringstate();
}
