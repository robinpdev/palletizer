use std::{collections::HashSet, ops::Add};

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone)]
#[wasm_bindgen]
pub struct PreSorterConfig {
    targetheight: u32,
    minheight: u32,
    maxheight: u32,
}

#[wasm_bindgen]
pub struct PreSorter {
    buffers: Box<[u32]>,    //size of stacks in buffer positions
    pub currentOutput: u32, // current size of next output stack
    strategy: SortStrategy,
    config: PreSorterConfig,
}

#[wasm_bindgen]
pub enum SortStrategy {
    FirstFitStrategy,
}

#[derive(Debug)]
enum SortAction {
    AddTo(usize),
    Pop(usize),
    Pass,
    Output,
    NotPossible,
}

impl SortStrategy {
    fn add(&self, item: u32, buffers: &Box<[u32]>, presorter: &PreSorter) -> Box<[SortAction]> {
        match self {
            SortStrategy::FirstFitStrategy => {
                // pass if item within spec
                if item >= presorter.config.minheight && item <= presorter.config.maxheight {
                    return [SortAction::Pass, SortAction::Output].into();
                }

                let mut mybuffers = buffers.clone();
                let mut currentout = 0;
                let mut actions: Vec<SortAction> = Vec::new();

                // look for combinations of buffers + item that are within spec
                let mut bag: Vec<u32> = Vec::new();
                bag.push(item);
                bag.extend_from_slice(buffers);
                let mut thesum: HashSet<usize> = HashSet::new();
                let sum = find_sum(
                    &bag.into_boxed_slice(),
                    presorter.config.minheight,
                    presorter.config.maxheight,
                    &mut thesum,
                    0,
                );

                if let Some(_) = sum {
                    for ind in thesum {
                        if ind == 0 {
                            actions.push(SortAction::Pass);
                        } else {
                            actions.push(SortAction::Pop(ind - 1));
                        }
                    }
                    actions.push(SortAction::Output);
                    return actions.into_boxed_slice();
                } else {
                    // no sum found, try to add to buffer
                    for (i, buffer) in buffers.iter().enumerate() {
                        if *buffer == 0 {
                            actions.push(SortAction::AddTo(i));
                            return actions.into_boxed_slice();
                        }
                    }

                    // try to switch with other larger buffer
                    for (i, buffer) in buffers.iter().enumerate() {
                        if *buffer > item {
                            actions.push(SortAction::Pop(i));
                            actions.push(SortAction::AddTo(i));
                            actions.push(SortAction::Output);
                            return actions.into_boxed_slice();
                        }
                    }

                    return [SortAction::Pass, SortAction::Output].into();
                }
            }
        }
    }
}

fn find_sum(
    bag: &Box<[u32]>,
    min: u32,
    max: u32,
    res: &mut HashSet<usize>,
    ind: usize,
) -> Option<bool> {
    if ind >= bag.len() {
        return None;
    }
    let item = bag[ind];
    if item >= min && item <= max {
        res.insert(ind);
        return Some(true);
    }
    if (min as i32 - item as i32) < 0 as i32 {
        return None;
    }
    //only if sorted
    // if item >= max || item <= min {
    //     return None;
    // }
    if let Some(with) = find_sum(bag, min - item, max - item, res, ind + 1) {
        res.insert(ind);
        return Some(true);
    } else if let Some(without) = find_sum(bag, min, max, res, ind + 1) {
        return Some(true);
    } else {
        return None;
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct ClearAddToResult {
    index: usize,
    height: u32,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub enum AddResult {
    NotPossible(u32),
    NoOutput(u32),
    Output(u32),
}

#[wasm_bindgen]
pub struct Buffer {
    capacity: u32,
    height: u32,
}

#[wasm_bindgen]
impl PreSorter {
    #[wasm_bindgen]
    pub fn new(
        nbuffers: u32,
        maxheight: u32,
        targetheight: u32,
        minheight: u32,
        strategy: SortStrategy,
    ) -> PreSorter {
        let buffers: Box<[u32]> = (0..nbuffers).map(|_| 0).collect();

        let sorter: PreSorter = PreSorter {
            buffers,
            strategy: strategy,
            currentOutput: 0,
            config: PreSorterConfig {
                targetheight,
                minheight,
                maxheight,
            },
        };

        return sorter;
    }

    #[wasm_bindgen]
    pub fn add_wasm(&mut self, item: u32) -> Result<JsValue, JsValue> {
        let result = self.add(item);
        return Ok(serde_wasm_bindgen::to_value(&result)?);
    }
    pub fn add(&mut self, item: u32) -> AddResult {
        let actions = self.strategy.add(item, &self.buffers, &self);
        let mut result: AddResult = AddResult::NoOutput(0);

        for action in actions {
            print!("{} - {:?}: {}", item, action, self.currentOutput);
            match action {
                SortAction::AddTo(bufind) => {
                    assert!(self.buffers[bufind] == 0);

                    self.buffers[bufind] = item;
                }
                SortAction::Pop(bufind) => {
                    assert!(self.currentOutput + self.buffers[bufind] <= self.config.maxheight);

                    self.currentOutput += self.buffers[bufind];
                    self.buffers[bufind] = 0;
                }
                SortAction::Pass => {
                    assert!(self.currentOutput + item <= self.config.maxheight);
                    self.currentOutput += item;
                }
                SortAction::Output => {
                    // assert!(self.currentOutput >= self.config.minheight);
                    result = AddResult::Output(self.currentOutput);
                    self.currentOutput = 0;
                }
                SortAction::NotPossible => {
                    println!("bro wat");
                    result = AddResult::NotPossible(0);
                    break;
                }
            }
            println!(" -> {:?}", self.currentOutput);
        }

        return result;
    }

    #[wasm_bindgen]
    pub fn stringstate(&self) -> String {
        let mut out: String = String::new();
        for buffer in &self.buffers {
            out += "[";
            for _ in (0..*buffer) {
                out += "■";
            }
            for _ in (0..(self.config.maxheight - *buffer)) {
                out += " ";
            }
            out += "] ";
            out += &(self.config.maxheight - *buffer).to_string();
            out += "\n";
        }

        return out;
    }

    pub fn reset(&mut self) {
        for i in 0..self.buffers.len() {
            self.buffers[i] = 0;
        }
        self.currentOutput = 0;
    }
}

pub fn simulate_random(steps: u64) {
    let mut env: PreSorter = PreSorter::new(4, 30, 25, 20, SortStrategy::FirstFitStrategy);

    env.reset();
    let mut fails: u64 = 0;
    for i in 0..steps {
        let item = rand::random_range(1..30);
        let result = env.add(item);
        match result {
            AddResult::NotPossible(_) => {
                fails += 1;
                env.reset();
            }
            _ => {}
        }
    }

    println!(
        "{} fails over {} steps = 1 fail every {} steps",
        fails,
        steps,
        1.0 / (fails as f64 / steps as f64)
    );
}
