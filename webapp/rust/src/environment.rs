use crate::environment::SortAction::{AddTo, ClearAndAddTo, NotPossible};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone)]
#[wasm_bindgen]
pub struct PreSorterConfig {
    targetheight: u32,
    minheight: u32,
}

#[wasm_bindgen]
pub struct PreSorter {
    buffers: Box<[Buffer]>,
    strategy: SortStrategy,
    config: PreSorterConfig,
}

#[wasm_bindgen]
pub enum SortStrategy {
    FirstFitStrategy,
}

impl SortStrategy {
    fn add(&self, item: u32, buffers: &Box<[Buffer]>, config: &PreSorterConfig) -> SortAction {
        match self {
            SortStrategy::FirstFitStrategy => {
                for (i, buffer) in buffers.iter().enumerate() {
                    if buffer.height + item <= buffer.capacity {
                        return SortAction::AddTo(i);
                    }
                }
                for (i, buffer) in buffers.iter().enumerate() {
                    if buffer.height >= config.minheight && item <= buffer.capacity {
                        return SortAction::ClearAndAddTo(i);
                    }
                }
                return NotPossible;
            }
        }
    }
}

enum SortAction {
    AddTo(usize),
    ClearAndAddTo(usize),
    NotPossible,
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
    NotPossible(usize),
    AddedTo(usize),
    ClearandAddTo(ClearAddToResult),
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
        capacities: u32,
        targetheight: u32,
        minheight: u32,
        strategy: SortStrategy,
    ) -> PreSorter {
        let buffers: Box<[Buffer]> = (0..nbuffers)
            .map(|_| Buffer {
                capacity: capacities,
                height: 0,
            })
            .collect();

        let sorter: PreSorter = PreSorter {
            buffers,
            strategy: strategy,
            config: PreSorterConfig {
                targetheight,
                minheight,
            },
        };

        return sorter;
    }

    #[wasm_bindgen]
    pub fn add(&mut self, item: u32) -> Result<JsValue, JsValue> {
        let action = self.strategy.add(item, &self.buffers, &self.config);
        let mut result: AddResult;

        let addbuffer: &mut Buffer;
        let addbufind: usize;
        match action {
            AddTo(bufind) => {
                addbufind = bufind;
                println!("adding to {}", bufind + 1);
                addbuffer = &mut self.buffers[bufind];
                assert!(addbuffer.height + item <= addbuffer.capacity);

                addbuffer.height += item;
                result = AddResult::AddedTo(bufind);
            }
            ClearAndAddTo(bufind) => {
                addbufind = bufind;
                addbuffer = &mut self.buffers[bufind];
                println!("forced clear {}", bufind + 1);
                assert!(addbuffer.height + item > addbuffer.capacity);
                assert!(addbuffer.height >= self.config.minheight);

                result = AddResult::ClearandAddTo(ClearAddToResult {
                    index: bufind,
                    height: addbuffer.height,
                });
                addbuffer.height = 0;
                addbuffer.height += item;
            }
            NotPossible => {
                result = AddResult::NotPossible(0);
                return Ok(serde_wasm_bindgen::to_value(&result)?);
            }
        }

        if addbuffer.height >= self.config.targetheight {
            println!("clear {}", addbufind + 1);
            result = AddResult::ClearandAddTo(ClearAddToResult {
                index: addbufind,
                height: addbuffer.height,
            });
            addbuffer.height = 0;
        }
        return Ok(serde_wasm_bindgen::to_value(&result)?);
    }

    #[wasm_bindgen]
    pub fn stringstate(&self) -> String {
        let mut out: String = String::new();
        for buffer in &self.buffers {
            out += "[";
            for _ in (0..buffer.height) {
                out += "■";
            }
            for _ in (0..(buffer.capacity - buffer.height)) {
                out += " ";
            }
            out += "] ";
            out += &(buffer.capacity - buffer.height).to_string();
            out += "\n";
        }

        return out;
    }

    pub fn reset(&mut self) {
        for buffer in self.buffers.iter_mut() {
            buffer.height = 0;
        }
    }
}
