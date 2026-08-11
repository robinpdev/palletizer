use std::{collections::HashSet, ops::Add};

use serde::{Deserialize, Serialize};
use wasm_bindgen::{convert::VectorIntoWasmAbi, prelude::*};

#[derive(Clone)]
#[wasm_bindgen]
pub struct PreSorterConfig {
    targetheight: u32,
    minheight: u32,
    maxheight: u32,
}

#[wasm_bindgen]
pub struct PreSorter {
    buffers: Box<[Vec<u32>]>,    //size of stacks in buffer positions
    currentOutput: Vec<u32>, // current size of next output stack
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
                    let maxbuffer = buffers.iter().enumerate().max_by(|a, b| a.1.cmp(b.1));
                    if let Some(maxbuf) = maxbuffer {
                        if *maxbuf.1 > item {
                            actions.push(SortAction::Pop(maxbuf.0));
                            actions.push(SortAction::AddTo(maxbuf.0));
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

#[derive(Serialize, Deserialize)]
pub enum AddResult {
    NotPossible(u32),
    NoOutput(u32),
    Output(Box<[u32]>),
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
        let buffers: Box<[Vec<u32>]> = (0..nbuffers).map(|_| Vec::new()).collect();

        let sorter: PreSorter = PreSorter {
            buffers,
            strategy: strategy,
            currentOutput: Vec::new(),
            config: PreSorterConfig {
                targetheight,
                minheight,
                maxheight,
            },
        };

        return sorter;
    }

    pub fn getBufferSizes(& self) -> Box<[u32]>{
        let sizes : Vec<u32> = self.buffers.iter().map(|b| b.iter().sum()).collect();
        return sizes.into_boxed_slice();
    }

    fn getBufferSize(&self, ind: usize) -> u32{
        return self.buffers[ind].iter().sum();
    }

    #[wasm_bindgen]
    pub fn add_wasm(&mut self, item: u32) -> Result<JsValue, JsValue> {
        let result = self.add(item);
        return Ok(serde_wasm_bindgen::to_value(&result)?);
    }

    pub fn add_wasm2(&mut self, item: u32) -> Option<Box<[u32]>>{
        let result = self.add(item);
        match result {
            AddResult::Output(o) => {
                return Some(o);
            }
            _ => None
        }
    }

    fn push_buffer(&mut self, ind: usize){
        self.currentOutput.extend(self.buffers[ind].iter());
        self.buffers[ind].clear();
    }

    fn add(&mut self, item: u32) -> AddResult {
        let actions = self.strategy.add(item, &self.getBufferSizes(), &self);
        let mut result: AddResult = AddResult::NoOutput(0);

        let mut bufsizes = self.getBufferSizes();

        for action in actions {
            print!("{} - {:?}: {}", item, action, self.currentOutput.iter().sum::<u32>());
            match action {
                SortAction::AddTo(bufind) => {
                    assert!(bufsizes[bufind] == 0);

                    bufsizes[bufind] = item;
                    self.buffers[bufind].push(item);
                }
                SortAction::Pop(bufind) => {
                    assert!(self.currentOutput.iter().sum::<u32>() + bufsizes[bufind] <= self.config.maxheight);

                    self.push_buffer(bufind);
                    bufsizes[bufind] = 0;
                }
                SortAction::Pass => {
                    assert!(self.currentOutput.iter().sum::<u32>() + item <= self.config.maxheight);
                    self.currentOutput.push(item);
                }
                SortAction::Output => {
                    // assert!(self.currentOutput >= self.config.minheight);
                    result = AddResult::Output(self.currentOutput.clone().into_boxed_slice());
                    self.currentOutput.clear();
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
    pub fn get_buffers(&self) -> Result<JsValue, JsValue> {
        let r : Vec<Box<[u32]>> = self.buffers.iter().map(|b| b.clone().into_boxed_slice()).collect();
        return Ok(serde_wasm_bindgen::to_value(&r)?);
    }

    #[wasm_bindgen]
    pub fn stringstate(&self) -> String {
        let mut out: String = String::new();
        for buffer in &self.buffers {
            out += "[";
            for _ in (0..buffer.iter().sum()) {
                out += "■";
            }
            for _ in (0..(self.config.maxheight - buffer.iter().sum::<u32>())) {
                out += " ";
            }
            out += "] ";
            out += &(self.config.maxheight - buffer.iter().sum::<u32>() as u32).to_string();
            out += "\n";
        }

        return out;
    }

    pub fn reset(&mut self) {
        for i in 0..self.buffers.len() {
            self.buffers[i].clear();
        }
        self.currentOutput.clear();
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
