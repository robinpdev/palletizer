use crate::environment::SortAction::{AddTo, ClearAndAddTo, NotPossible};

#[derive(Clone)]
pub struct PreSorterConfig {
    targetheight: u32,
    minheight: u32,
}

pub struct PreSorter {
    buffers: Box<[Buffer]>,
    strategy: SortStrategy,
    config: PreSorterConfig,
}

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

pub struct Buffer {
    capacity: u32,
    height: u32,
}

impl PreSorter {
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

    pub fn add(&mut self, item: u32) -> bool {
        let action = self.strategy.add(item, &self.buffers, &self.config);

        let addbuffer: &mut Buffer;
        let addbufind: usize;
        match action {
            AddTo(bufind) => {
                addbufind = bufind;
                println!("adding to {}", bufind + 1);
                addbuffer = &mut self.buffers[bufind];
                assert!(addbuffer.height + item <= addbuffer.capacity);

                addbuffer.height += item;
            }
            ClearAndAddTo(bufind) => {
                addbufind = bufind;
                addbuffer = &mut self.buffers[bufind];
                println!("forced clear {}", bufind + 1);
                assert!(addbuffer.height + item > addbuffer.capacity);
                assert!(addbuffer.height >= self.config.minheight);

                addbuffer.height = 0;
                addbuffer.height += item;
            }
            NotPossible => {
                return false;
            }
        }

        if addbuffer.height >= self.config.targetheight {
            println!("clear {}", addbufind + 1);
            addbuffer.height = 0;
        }
        return true;
    }

    pub fn stringstate(&self) -> String {
        let mut out: String = String::new();
        for buffer in &self.buffers {
            out += "[";
            for _ in (0..buffer.height) {
                out += "O";
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
