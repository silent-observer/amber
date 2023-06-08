use std::thread::{JoinHandle, spawn};
use kanal;

use crate::pins::{PinId, PinState};
use crate::vcr::{VcrTree, VcrWriter, MutexVcrTree};

#[derive(Debug, Clone)]
pub enum Message {
    Step,
    Done(ComponentId),
    Die,
    PinChange(ComponentId, PinId, PinState),
}

struct ComponentHandle {
    id: ComponentId,
    thread: Option<JoinHandle<()>>,
    input_tx: kanal::Sender<Message>,
}

impl ComponentHandle {
    fn notify_on_pin_change(&self, pin: PinId, state: PinState) {
        self.input_tx
            .send(Message::PinChange(self.id, pin, state))
            .expect("Error sending update");
    }

    fn notify_step(&self) {
        self.input_tx
            .send(Message::Step)
            .expect("Error sending update");
    }
}

impl Drop for ComponentHandle {
    fn drop(&mut self) {
        self.input_tx.send(Message::Die).expect("Error sending update");
        self.thread.take().unwrap().join().unwrap();
    }
}

type PinIndex = usize;

#[derive(Debug, Clone, Copy)]
pub struct WireId(usize);
#[derive(Debug, Clone, Copy)]
pub struct ComponentId(usize);

struct WireStateCounter {
    low: u8,
    high: u8,
    weak_low: u8,
    weak_high: u8,
}

impl WireStateCounter {
    fn add(&mut self, pin: PinState) {
        match pin {
            PinState::Low => self.low += 1,
            PinState::High => self.high += 1,
            PinState::WeakLow => self.weak_low += 1,
            PinState::WeakHigh => self.weak_high += 1,
            PinState::Z => {},
            PinState::Error => {
                self.low += 1;
                self.high += 1;
            }
        }
    }

    fn remove(&mut self, pin: PinState) {
        match pin {
            PinState::Low => self.low -= 1,
            PinState::High => self.high -= 1,
            PinState::WeakLow => self.weak_low -= 1,
            PinState::WeakHigh => self.weak_high -= 1,
            PinState::Z => {},
            PinState::Error => {
                self.low -= 1;
                self.high -= 1;
            }
        }
    }

    fn read(&self) -> PinState {
        match self {
            WireStateCounter {low: 0, high: 0, weak_low: 0, weak_high: 0} => PinState::Z,
            WireStateCounter {low: 0, high: 0, weak_low: 0, weak_high: _} => PinState::WeakHigh,
            WireStateCounter {low: 0, high: 0, weak_low: _, weak_high: 0} => PinState::WeakLow,
            WireStateCounter {low: 0, high: 0, weak_low: _, weak_high: _} => PinState::Error,
            WireStateCounter {low: 0, high: _, weak_low: _, weak_high: _} => PinState::High,
            WireStateCounter {low: _, high: 0, weak_low: _, weak_high: _} => PinState::Low,
            _ => PinState::Error,
        }
    }
}

struct WireState {
    counter: WireStateCounter,
    pins: Vec<PinIndex>
}

impl WireState {
    fn read(&self) -> PinState {
        self.counter.read()
    }
}

struct Pin {
    id: PinId,
    component: Option<ComponentId>,
    wire: Option<WireId>,
    out_state: PinState,
}

pub struct Board {
    output_rx: Option<kanal::Receiver<Message>>,
    output_tx: Option<kanal::Sender<Message>>,

    component_names: Vec<String>,
    components: Vec<ComponentHandle>,
    pin_table: Vec<Vec<PinIndex>>,
    pins: Vec<Pin>,
    wires: Vec<WireState>,

    vcr_writer: VcrWriter,
}

impl Board {
    pub fn new(vcr_path: &str, freq: f64) -> Board {
        let (output_tx, output_rx) = kanal::unbounded();
        let clock_pin = Pin {
            id: 0,
            component: None,
            wire: None,
            out_state: PinState::Low,
        };
        Board {
            output_rx: Some(output_rx),
            output_tx: Some(output_tx),
            component_names: Vec::new(),
            components: Vec::new(),
            pin_table: Vec::new(),
            pins: vec![clock_pin],
            wires: Vec::new(),
            
            vcr_writer: VcrWriter::new(vcr_path, freq),
        }
    }

    pub fn add_component_fn<F>(&mut self,
                            f: F,
                            vcr: VcrTree,
                            name: &str,
                            pins_count: u16) -> ComponentId
    where
        F: FnOnce(ComponentId, kanal::Sender<Message>, kanal::Receiver<Message>, MutexVcrTree) + Send + 'static
    {
        let (input_tx, input_rx) = kanal::unbounded();
        let output_tx_copy = self.output_tx
            .as_ref()
            .expect("Cannot create a component without output transmitter")
            .clone();
        let component_id = ComponentId(self.components.len());
        let vcr_mutex = self.vcr_writer.add(name, vcr);

        let thread = spawn(move || f(component_id, output_tx_copy, input_rx, vcr_mutex));
        let c = ComponentHandle {
            id: component_id,
            thread: Some(thread),
            input_tx
        };

        

        let mut pin_vec = Vec::with_capacity(pins_count as usize);
        for id in 0..pins_count {
            pin_vec.push(self.pins.len());
            self.pins.push(Pin {
                id,
                component: Some(component_id),
                wire: None,
                out_state: PinState::Z,
            });
            c.notify_on_pin_change(id, PinState::Z);
        }
        self.pin_table.push(pin_vec);
        self.components.push(c);
        self.component_names.push(name.to_string());
        
        component_id
    }

    pub fn add_wire(&mut self, pins: &[(ComponentId, PinId)]) -> WireId {
        let mut wire = WireState {
            counter: WireStateCounter { low: 0, high: 0, weak_low: 0, weak_high: 0 },
            pins: Vec::with_capacity(pins.len())
        };

        let wire_id = WireId(self.wires.len());
        for &(ComponentId(component_id), pin_id) in pins {
            let index = self.pin_table[component_id][pin_id as usize];
            let pin = &mut self.pins[index];
            wire.pins.push(index);
            wire.counter.add(pin.out_state);
            assert!(pin.wire.is_none(), "Cannot connect two wires to the same pin!");
            pin.wire = Some(wire_id);
        }
        let wire_state = wire.read();
        self.wires.push(wire);
        for &(ComponentId(component_id), pin_id) in pins {
            self.components[component_id].notify_on_pin_change(pin_id, wire_state);
        }
        wire_id
    }

    pub fn add_clock_wire(&mut self, pins: &[(ComponentId, PinId)]) -> WireId {
        let wire_id = self.add_wire(pins);
        self.wires[wire_id.0].pins.push(0);
        self.pins[0].wire = Some(wire_id);
        self.wires[wire_id.0].counter.add(self.pins[0].out_state);

        let wire_state = self.wires[wire_id.0].read();
        for &(ComponentId(component_id), pin_id) in pins {
            self.components[component_id].notify_on_pin_change(pin_id, wire_state);
        }

        wire_id
    }

    fn set_pin(&mut self,
               pin_index: PinIndex,
               state: PinState) {
        let (old_state, wire) = self.update_pin_output(pin_index, state);

        if let Some(wire_index) = wire {
            let (old_out_state, new_out_state) = 
                self.update_wire(wire_index, old_state, state);
            self.update_pins_inputs(wire_index, old_out_state, new_out_state)
        }
    }

    fn update_pin_output(&mut self,
                         index: PinIndex,
                         state: PinState) -> (PinState, Option<WireId>) {
        let pin = &mut self.pins[index];
        let old_state = pin.out_state;
        pin.out_state = state;
        (old_state, pin.wire)
    }

    fn update_wire(&mut self,
                   WireId(wire_index): WireId,
                   old_state: PinState,
                   new_state: PinState) -> (PinState, PinState) {
        let wire = &mut self.wires[wire_index];
        let old_out_state = wire.read();
        if old_state != new_state {
            wire.counter.remove(old_state);
            wire.counter.add(new_state);
        }
        let new_out_state = wire.read();
        (old_out_state, new_out_state)
    }

    fn update_pins_inputs(&mut self,
                          WireId(wire_index): WireId,
                          old_out_state: PinState,
                          new_out_state: PinState) {
        if new_out_state != old_out_state {
            for &pin_index in &self.wires[wire_index].pins {
                if let Some(ComponentId(index)) = self.pins[pin_index].component {
                    self.components[index].notify_on_pin_change(self.pins[pin_index].id, new_out_state);
                }
            }
        }
    }

    pub fn handle_messages(&mut self) {
        let mut done_counter = self.components.len();
        let mut output_rx = self.output_rx.take().expect("Has to have output reciever");
        'outer_loop:
        loop {
            for m in &mut output_rx {
                println!("Got message: {:?}", m);
                match m {
                    Message::Step | Message::Die => {}
                    Message::Done(_component_id) => {
                        done_counter -= 1;
                        if done_counter == 0 {break 'outer_loop;}
                    }
                    Message::PinChange(ComponentId(component_id), pin_id, state) => {
                        let index = self.pin_table[component_id][pin_id as usize];
                        self.set_pin(index, state);
                    }
                }
            }
        }
        self.output_rx = Some(output_rx);
    }

    pub fn toggle_clock(&mut self) {
        const CLOCK_PIN: PinIndex = 0;
        if self.pins[CLOCK_PIN].out_state == PinState::High {
            self.set_pin(CLOCK_PIN, PinState::Low);
        } else {
            self.set_pin(CLOCK_PIN, PinState::High);
        }
        for c in &self.components {
            c.notify_step();
        }
        
        self.handle_messages();
    }

    pub fn simulate(&mut self, clocks: u64) {
        self.vcr_writer.write_header();
        for _ in 0..clocks {
            self.toggle_clock();
            self.vcr_writer.write_step();
        }
    }
}