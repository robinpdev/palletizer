use crate::{
    environment::{
        AddResult::{self, NotPossible},
        PreSorter, simulate_random,
    },
    excel::readXLSSeqs,
};
use std::io;
use wasm_bindgen::prelude::*;

mod environment;
mod excel;

pub fn main() {
    // simulate_random(1_000_000);
    simulate_excel();
}

pub fn simulate_excel() {
    let mut env: environment::PreSorter =
        PreSorter::new(4, 30, 25, 20, environment::SortStrategy::FirstFitStrategy);

    // let seqs = vec![random_input(10_000)];
    let seqs = readXLSSeqs(
        "/home/robin/code/roularta/palletizer/FW_ Prompt + bestanden/Bundelprofiel TV Film AI.xlsx"
            .to_string(),
    )
    .unwrap();

    let mut steps: u64 = 0;
    let mut fails: u64 = 0;

    for seq in seqs {
        println!("{:?}", seq);
        for item in seq {
            let result = env.add(item);
            steps += 1;
            match result {
                AddResult::NotPossible(_) => {
                    fails += 1;
                    env.reset();
                }
                _ => {}
            }
        }
        env.reset();
    }

    println!(
        "{} fails over {} steps = 1 fail every {} steps",
        fails,
        steps,
        1.0 / (fails as f64 / steps as f64)
    );
}
// pub fn main() {
//     println!("Hello, world!");

//     // random sequence

//     let mut env: environment::PreSorter =
//         PreSorter::new(4, 30, 25, 20, environment::SortStrategy::FirstFitStrategy);

//     // let seqs = vec![random_input(10_000)];
//     let seqs = [[1, 2, 8, 5, 22, 14, 21, 12]];

//     let mut steps = 0;

//     for seq in seqs {
//         for item in seq {
//             println!("add {}", item);

//             let result = env.add(item);

//             println!("{}", env.stringstate());

//             // if let AddResult::NotPossible(_) = result {
//             //     println!("STOP after {} steps", steps);

//             //     // require enter to continue
//             //     let mut buffer = String::new();
//             //     let stdin = io::stdin(); // We get `Stdin` here.
//             //     stdin.read_line(&mut buffer);

//             //     env.reset();
//             //     steps = 0;

//             //     // break;
//             // }
//             steps += 1;
//         }
//     }
// }
