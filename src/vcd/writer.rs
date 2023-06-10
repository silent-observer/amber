use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::{io::BufWriter, fs::File};

use crate::pins::{PinState, PinVec};

use super::{VcdTree, VcdForest, VcdTreeModule, VcdTreeSignal, MutexVcdTree, VcdTreeHandle};

/// Writes VCD signals into a .vcd file.
pub struct VcdWriter {
    /// File writer
    f: BufWriter<File>,
    /// Next VCD short identifier
    wire_id: Vec<u8>,
    /// Current state of VCD signals
    forest: VcdForest,
    /// Current step
    counter: u64,
    /// Nanoseconds per step
    period: f64,
}

impl VcdWriter {
    /// Creates new [VcdWriter].
    /// 
    /// `path` is path to output .vcd file.
    /// `freq` is step frequency in Hz.
    pub fn new(path: &str, freq: f64) -> VcdWriter {
        let f = File::create(path).expect("Couldn't create file");
        VcdWriter { 
            f: BufWriter::new(f),
            wire_id: vec![33],
            forest: VcdForest::new(),
            counter: 1,
            period: 5e8 / freq
        }
    }

    /// Adds a new threaded component with given name into the [VcdWriter].
    pub fn add_threaded(&mut self, name: &str, vcd: VcdTree) -> Arc<Mutex<VcdTreeHandle>> {
        self.forest.add_threaded(name, vcd)
    }

    /// Adds a new threadless component with given name into the [VcdWriter].
    pub fn add_threadless(&mut self, name: &str, vcd: VcdTree) -> Rc<RefCell<VcdTreeHandle>> {
        self.forest.add_threadless(name, vcd)
    }

    /// Generates a new VCD short idenitifer.
    fn next_id(wire_id: &mut Vec<u8>) -> String {
        let result = std::str::from_utf8(&wire_id)
                         .expect("Couldn't convert string")
                         .to_string();
        
        'outer_block:
        {
            for i in (0..wire_id.len()).rev() {
                wire_id[i] += 1;
                if wire_id[i] == 127 {
                    wire_id[i] = 33;
                } else {
                    break 'outer_block;
                }
            }
            wire_id.push(33);
        }

        result
    }

    /// Recursively writes scope section of .vcd file.
    fn write_scope(f: &mut BufWriter<File>, wire_id: &mut Vec<u8>, tree: &mut VcdTree, name: &str) {
        match tree {
            VcdTree::Module(VcdTreeModule(hash_map)) => {
                write!(f, "$scope module {} $end\n", name).expect("Couldn't write scope");
                for (k, v) in hash_map {
                    Self::write_scope(f, wire_id, v, k);
                }
                write!(f, "$upscope $end\n").expect("Couldn't write scope");
            },
            VcdTree::Signal(VcdTreeSignal { size,
                                            id,
                                            ..}) => {
                let new_id = Self::next_id(wire_id);
                if *size == 1 {
                    write!(f, "$var wire {} {} {} $end\n", *size, new_id, name)
                        .expect("Couldn't write scope");
                } else {
                    write!(f, "$var wire {} {} {}[{}:0] $end\n", *size, new_id, name, *size-1)
                        .expect("Couldn't write scope");
                }
                
                *id = Some(new_id);
            },
            VcdTree::Disabled => {}
        }
    }

    /// Writes scope section of .vcd file fom the whole [VcdForest].
    fn write_scope_forest(f: &mut BufWriter<File>, wire_id: &mut Vec<u8>, tree: &mut VcdForest) {
        write!(f, "$scope module TOP $end\n").expect("Couldn't write scope");
        for (k, v) in &mut tree.0 {
            match v {
                MutexVcdTree::Threaded(x) => 
                    Self::write_scope(f, wire_id,
                        &mut x.lock().expect("Couldn't lock mutex").tree,
                        k),
                MutexVcdTree::Threadless(x) => 
                    Self::write_scope(f, wire_id,
                        &mut x.borrow_mut().tree,
                        k),
            }
        }
        write!(f, "$upscope $end\n").expect("Couldn't write scope");
    }

    /// Encodes [PinState] for .vcd.
    fn state_to_char(state: PinState) -> char {
        match state {
            PinState::Z => 'z',
            PinState::Low | PinState::WeakLow => '0',
            PinState::High | PinState::WeakHigh => '1',
            PinState::Error => 'x',
        }
    }

    /// Encodes multiple [PinState] for .vcd.
    fn state_vec_to_char(state: &PinVec) -> String {
        let mut s = String::with_capacity(state.len() as usize);
        for x in state.iter() {
            s.push(Self::state_to_char(x));
        }
        s
    }

    /// Writes data section of a single [VcdTree] for the .vcd file.
    /// 
    /// If `dumpvars` is `true`, then the whole tree is dumped, otherwise only the changes are.
    fn write_data(f: &mut BufWriter<File>, tree: &VcdTree, dumpvars: bool) {
        match tree {
            VcdTree::Module(VcdTreeModule(hash_map)) => {
                for (_k, v) in hash_map {
                    Self::write_data(f, v, dumpvars);
                }
            },
            VcdTree::Signal(VcdTreeSignal { id: Some(id),
                                            size,
                                            old_state,
                                            new_state,
                                            ..}) => {
                if dumpvars || old_state != new_state {
                    if *size == 1 {
                        write!(f, "{}{}\n", Self::state_vec_to_char(new_state), id)
                            .expect("Couldn't write data");
                    } else {
                        write!(f, "b{} {}\n", Self::state_vec_to_char(new_state), id)
                            .expect("Couldn't write data");
                    }
                    
                }
                
            },
            _ => panic!("Invalid VcdTree!")
        }
    }

    /// Writes data section of the whole [VcdForest] for the .vcd file.
    /// 
    /// If `dumpvars` is `true`, then the whole tree is dumped, otherwise only the changes are.
    fn write_data_forest(f: &mut BufWriter<File>, tree: &VcdForest, dumpvars: bool) {
        for (_k, v) in &tree.0 {
            match v {
                MutexVcdTree::Threaded(x) => {
                    let guard = &mut *x.lock().expect("Couldn't lock mutex");
                    if dumpvars || guard.changed {
                        Self::write_data(f,
                            &guard.tree,
                            dumpvars);
                    };
                    guard.changed = false;
                }
                MutexVcdTree::Threadless(x) => {
                    let guard = &mut *x.borrow_mut();
                    if dumpvars || guard.changed {
                        Self::write_data(f,
                            &guard.tree,
                            dumpvars)
                    };
                    guard.changed = false;
                },
            }
        }
    }

    /// Writes .vcd file file header, together with $scope and $dumpvars sections.
    pub fn write_header(&mut self) {
        write!(&mut self.f, "\
            $version Generated by Amber $end\n\
            $date Wed Jun 7 18:38:32 2023 $end\n\
            $timescale 1ns $end\n\
            ").expect("Couldn't write header");
        Self::write_scope_forest(&mut self.f, &mut self.wire_id, &mut self.forest);
        write!(&mut self.f, "$enddefinitions $end\n$dumpvars\n")
            .expect("Couldn't write header");
        Self::write_data_forest(&mut self.f, &mut self.forest, true);
        write!(&mut self.f, "$end\n").expect("Couldn't write header");
    }

    fn has_changed(&self) -> bool {
        for (_k, v) in &self.forest.0 {
            match v {
                MutexVcdTree::Threaded(x) => {
                    let guard = x.lock().expect("Couldn't lock mutex");
                    if guard.changed {
                        return true;
                    };
                }
                MutexVcdTree::Threadless(x) => {
                    let guard = x.borrow();
                    if guard.changed{
                        return true;
                    };
                },
            }
        }
        false
    }

    /// Writes a single step into a .vcd file.
    pub fn write_step(&mut self) {
        if self.has_changed() {
            write!(&mut self.f, "#{}\n", (self.counter as f64 * self.period).round() as u64).expect("Couldn't write timestep");
            self.counter += 1;
            Self::write_data_forest(&mut self.f, &mut self.forest, false);
        } else {
            self.counter += 1;
        }
    }
}