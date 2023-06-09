use std::thread::{JoinHandle, spawn};
use kanal;

use crate::pins::{PinId, PinState};
use crate::vcd::{VcdTree, VcdWriter, MutexVcdTree};

/// Messages used to communicate with components, working in other threads.
/// 
/// The main communication protocol is the following:
/// ```none
/// loop {
///     Board -> PinChange(component, pin_1, pin_state_1) -> Component
///     Board -> PinChange(component, pin_2, pin_state_2) -> Component
///     ...
///     Board -> PinChange(component, pin_n, pin_state_n) -> Component
///     Board -> Step -> Component
///     [Component advances a step using all the new pin states]
///     Component -> Done(component)
/// }
/// Board -> Die -> Component
/// [Component thread stops]
/// ```
#[derive(Debug, Clone)]
pub enum Message {
    /// Board to Component: advance a single step accounting for all the changes.
    Step,
    /// Component to Board: sent after Step is done.
    Done(ComponentId),
    /// Board to Component: stop component thread.
    Die,
    /// Board to Component: notify component about a pin changing state.
    PinChange(ComponentId, PinId, PinState),
}

/// Index of a pin. Unlike [PinId], this is unique for the whole board, not only for one component.
type PinIndex = usize;

/// A unique identifier for a wire.
#[derive(Debug, Clone, Copy)]
pub struct WireId(usize);
/// A unique identifier for a component.
#[derive(Debug, Clone, Copy)]
pub struct ComponentId(usize);

/// Internal component handle.
struct ComponentHandle {
    /// Unique id of the component.
    id: ComponentId,
    /// Handle for the component thread.
    thread: Option<JoinHandle<()>>,
    /// Channel for transmitting messages into the component.
    input_tx: kanal::Sender<Message>,
    /// Vector of all the unque pin indices.
    pins: Vec<PinIndex>
}

impl ComponentHandle {
    /// Send [Message::PinChange].
    fn notify_on_pin_change(&self, pin: PinId, state: PinState) {
        self.input_tx
            .send(Message::PinChange(self.id, pin, state))
            .expect("Error sending update");
    }

    /// Send [Message::Step].
    fn notify_step(&self) {
        self.input_tx
            .send(Message::Step)
            .expect("Error sending update");
    }
}

/// When a handle is dropped, the corresponding component thread is stopped automatically.
impl Drop for ComponentHandle {
    fn drop(&mut self) {
        self.input_tx.send(Message::Die).expect("Error sending update");
        self.thread.take().unwrap().join().unwrap();
    }
}

/// A representation for a state of a single wire.
/// 
/// Counts how many input signals of each type it has.
struct WireStateCounter {
    low: u8,
    high: u8,
    weak_low: u8,
    weak_high: u8,
}

impl WireStateCounter {
    /// Adds a new input signal into the [WireStateCounter].
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

    /// Removes an input signal into the [WireStateCounter].
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

    /// Reads the current state of the wire.
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

/// A representation for a single wire connecting multiple pins.
struct Wire {
    counter: WireStateCounter,
    pins: Vec<PinIndex>
}

impl Wire {
    /// Reads current wire state.
    fn read(&self) -> PinState {
        self.counter.read()
    }
}

/// A representation for a single pin of a component.
struct Pin {
    /// [PinId] is unique only up to a component.
    id: PinId,
    /// A component to which the pin belongs.
    /// 
    /// There can be pins without components.
    component: Option<ComponentId>,
    /// A wire that is connected to the pin.
    /// 
    /// There can be pins without a connected wire.
    wire: Option<WireId>,
    /// A state the pin is currently outputting.
    /// 
    /// If a pin is an input pin, it is still outputting [PinState::Z].
    out_state: PinState,
}

/// Top-level element of a simulation. A board containing multiple components.
pub struct Board {
    /// Channel for recieving messages from the component.
    output_rx: Option<kanal::Receiver<Message>>,
    /// Transmitter corresponding to `output_rx`, to be cloned into every new component.
    output_tx: Option<kanal::Sender<Message>>,
    /// Vector of all components. Indexed by [ComponentId]
    components: Vec<ComponentHandle>,
    /// Vector of all pins. Indexed by [PinIndex]
    pins: Vec<Pin>,
    /// Vector of all wires. Indexed by [WireId]
    wires: Vec<Wire>,

    /// A VCD file writer.
    vcd_writer: VcdWriter,
}

impl Board {
    /// Creates a new board.
    /// 
    /// `vcd_path` is an output path for .vcd file.
    /// `freq` is the clock frequency in Hz.
    pub fn new(vcd_path: &str, freq: f64) -> Board {
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
            components: Vec::new(),
            pins: vec![clock_pin],
            wires: Vec::new(),
            
            vcd_writer: VcdWriter::new(vcd_path, freq),
        }
    }

    /// Adds a new component to the board using a closure.
    /// 
    /// Using this you can add custom components without `Component` structure.
    /// The component closure must accept component id, two channel sides for
    /// transmitting and receiving messages, and also a VcdTree behind a mutex,
    /// to be updated after every [Message::Step].
    /// 
    /// This also requires already initialized VcdTree.
    pub fn add_component_fn<F>(&mut self,
                            f: F,
                            vcd: VcdTree,
                            name: &str,
                            pins_count: u16) -> ComponentId
    where
        F: FnOnce(ComponentId, kanal::Sender<Message>, kanal::Receiver<Message>, MutexVcdTree) + Send + 'static
    {
        let (input_tx, input_rx) = kanal::unbounded();
        let output_tx_copy = self.output_tx
            .as_ref()
            .expect("Cannot create a component without output transmitter")
            .clone();
        let component_id = ComponentId(self.components.len());
        let vcd_mutex = self.vcd_writer.add(name, vcd);

        let thread = spawn(move || f(component_id, output_tx_copy, input_rx, vcd_mutex));
        let mut pins = Vec::with_capacity(pins_count as usize);
        for id in 0..pins_count {
            pins.push(self.pins.len());
            self.pins.push(Pin {
                id,
                component: Some(component_id),
                wire: None,
                out_state: PinState::Z,
            });
        }

        let c = ComponentHandle {
            id: component_id,
            thread: Some(thread),
            pins,
            input_tx
        };

        for id in 0..pins_count {
            c.notify_on_pin_change(id, PinState::Z);
        }
        self.components.push(c);
        
        component_id
    }

    /// Gets [PinIndex] from [ComponentId] and [PinId].
    fn get_pin_index(&self, component_id: ComponentId, pin_id: PinId) -> PinIndex {
        self.components[component_id.0].pins[pin_id as usize]
    }

    /// Adds a new wire to the board, connecting specified pins.
    pub fn add_wire(&mut self, pins: &[(ComponentId, PinId)]) -> WireId {
        let mut wire = Wire {
            counter: WireStateCounter { low: 0, high: 0, weak_low: 0, weak_high: 0 },
            pins: Vec::with_capacity(pins.len())
        };

        let wire_id = WireId(self.wires.len());
        for &(component_id, pin_id) in pins {
            let index = self.get_pin_index(component_id, pin_id);
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

    /// Adds a new clock wire to the board, connecting specified pins.
    /// 
    /// It is automatically connected to the internal clock pin.
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

    /// Set a new pin state and propagate the updates through wires.
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

    /// Set a new pin state.
    fn update_pin_output(&mut self,
                         index: PinIndex,
                         state: PinState) -> (PinState, Option<WireId>) {
        let pin = &mut self.pins[index];
        let old_state = pin.out_state;
        pin.out_state = state;
        (old_state, pin.wire)
    }

    /// Update wire internal counters.
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

    /// Notify all the components connected to the wire about the pin change.
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

    /// Handle [messages](Message) for a single step of simulation.
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
                    Message::PinChange(component_id, pin_id, state) => {
                        let index = self.get_pin_index(component_id, pin_id);
                        self.set_pin(index, state);
                    }
                }
            }
        }
        self.output_rx = Some(output_rx);
    }

    /// Toggle a clock pin once.
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

    /// Run the simulation for specified number of cycles.
    pub fn simulate(&mut self, cycles: u64) {
        self.vcd_writer.write_header();
        for _ in 0..cycles {
            self.toggle_clock();
            self.vcd_writer.write_step();
            self.toggle_clock();
            self.vcd_writer.write_step();
        }
    }
}