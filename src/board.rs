use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::thread::{JoinHandle, spawn};
use kanal;

use crate::component::{Component, ComponentId, Message, ThreadlessComponent};
use crate::pins::{PinId, PinState};
use crate::vcd::{VcdTree, VcdWriter, VcdConfig, VcdTreeHandle};

/// Index of a pin. Unlike [PinId], this is unique for the whole board, not only for one component.
type PinIndex = usize;

/// A unique identifier for a wire.
#[derive(Debug, Clone, Copy)]
pub struct WireId(usize);

/// Internal component data.
struct ThreadedComponentData {
    /// Unique id of the component.
    id: ComponentId,
    /// Handle for the component thread.
    thread: Option<JoinHandle<()>>,
    /// Channel for transmitting messages into the component.
    input_tx: kanal::Sender<Message>,
}

impl ThreadedComponentData {
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
impl Drop for ThreadedComponentData {
    fn drop(&mut self) {
        self.input_tx.send(Message::Die).expect("Error sending update");
        self.thread.take().unwrap().join().unwrap();
    }
}

struct ThreadlessComponentData {
    /// Component data
    component: Box<dyn ThreadlessComponent>,
    /// VCD subtree
    vcd: Rc<RefCell<VcdTreeHandle>>
}

impl ThreadlessComponentData {
    /// Send [Message::PinChange].
    fn set_pin(&mut self, pin: PinId, state: PinState) {
        self.component.set_pin(pin, state);
    }

    /// Send [Message::Step].
    fn execute_step(&mut self, output_changes: &mut HashMap<PinId, PinState>) {
        self.component.execute_step_threadless(&self.vcd, output_changes);
    }
}

enum ComponentData {
    Threaded(ThreadedComponentData),
    Threadless(ThreadlessComponentData)
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
    components: Vec<ComponentData>,
    /// Vector of all pins. Indexed by [PinIndex]
    pins: Vec<Pin>,
    /// Vector of all wires. Indexed by [WireId]
    wires: Vec<Wire>,

    pin_mapping: Vec<Vec<PinIndex>>,

    changed_components: Vec<bool>,

    /// A VCD file writer.
    vcd_writer: VcdWriter,
}
pub struct ComponentHandle {
    id: ComponentId,
    pin_name_lookup: HashMap<String, PinId>,
}

impl ComponentHandle {
    pub fn pin(&self, name: &str) -> (ComponentId, PinId) {
        (
            self.id,
            *self.pin_name_lookup
                 .get(name)
                 .expect("Such pin doesn't exist!")
        )
    }
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
            pin_mapping: Vec::new(),
            changed_components: Vec::new(),
            
            vcd_writer: VcdWriter::new(vcd_path, freq),
        }
    }

    fn add_pins(&mut self, pins_count: u16, component_id: ComponentId) {
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
        self.pin_mapping.push(pins);
    }

    /// Adds a [Component] to the [Board] with the specified [VcdConfig].
    pub fn add_component<T>(&mut self, component: T, name: &str, config: &VcdConfig) -> ComponentHandle
    where
        T: Component + 'static
    {
        let vcd_init: VcdTree = component.init_vcd(config);
        let pin_name_lookup = T::get_pin_name_lookup();

        let (input_tx, input_rx) = kanal::unbounded();
        let output_tx_copy = self.output_tx
            .as_ref()
            .expect("Cannot create a component without output transmitter")
            .clone();
        let component_id = ComponentId(self.components.len());
        let vcd_mutex = self.vcd_writer.add_threaded(name, vcd_init);
        let pins_count = T::pin_count() as PinId;

        let thread = spawn(move || {
            let mut c = component;
            c.execute_loop(component_id, output_tx_copy, input_rx, vcd_mutex);
        });
        self.add_pins(pins_count, component_id);

        let c = ThreadedComponentData {
            id: component_id,
            thread: Some(thread),
            input_tx
        };

        for id in 0..pins_count {
            c.notify_on_pin_change(id, PinState::Z);
        }
        self.components.push(ComponentData::Threaded(c));
        self.changed_components.push(true);
        
        ComponentHandle {
            id: component_id,
            pin_name_lookup,
        }
    }

    /// Adds a [Component] to the [Board] with the specified [VcdConfig].
    pub fn add_component_threadless<T>(&mut self, component: T, name: &str, config: &VcdConfig) -> ComponentHandle
    where
        T: Component + 'static
    {
        let vcd_init: VcdTree = component.init_vcd(config);
        let pin_name_lookup = T::get_pin_name_lookup();

        let component_id = ComponentId(self.components.len());
        let vcd = self.vcd_writer.add_threadless(name, vcd_init);
        let pins_count = T::pin_count() as PinId;
        self.add_pins(pins_count, component_id);

        let mut c = ThreadlessComponentData {
            component: Box::new(component),
            vcd,
        };
        for id in 0..pins_count {
            c.set_pin(id, PinState::Z);
        }

        self.components.push(ComponentData::Threadless(c));
        self.changed_components.push(true);

        ComponentHandle {
            id: component_id,
            pin_name_lookup,
        }
    }

    /// Adds a new wire to the board, connecting specified pins.
    pub fn add_wire(&mut self, pins: &[(ComponentId, PinId)]) -> WireId {
        let mut wire = Wire {
            counter: WireStateCounter { low: 0, high: 0, weak_low: 0, weak_high: 0 },
            pins: Vec::with_capacity(pins.len())
        };

        let wire_id = WireId(self.wires.len());
        for &(component_id, pin_id) in pins {
            let index = self.pin_mapping[component_id.0][pin_id as usize];
            let pin = &mut self.pins[index];
            wire.pins.push(index);
            wire.counter.add(pin.out_state);
            assert!(pin.wire.is_none(), "Cannot connect two wires to the same pin!");
            pin.wire = Some(wire_id);
        }
        let wire_state = wire.read();
        self.wires.push(wire);
        for &(ComponentId(component_id), pin_id) in pins {
            match &mut self.components[component_id] {
                ComponentData::Threaded(c) =>
                    c.notify_on_pin_change(pin_id, wire_state),
                ComponentData::Threadless(c) =>
                    c.set_pin(pin_id, wire_state),
            }
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
            match &mut self.components[component_id] {
                ComponentData::Threaded(c) =>
                    c.notify_on_pin_change(pin_id, wire_state),
                ComponentData::Threadless(c) =>
                    c.set_pin(pin_id, wire_state),
            }
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
                    let pin_id = self.pins[pin_index].id;
                    match &mut self.components[index] {
                        ComponentData::Threaded(c) =>
                            c.notify_on_pin_change(pin_id, new_out_state),
                        ComponentData::Threadless(c) =>
                            c.set_pin(pin_id, new_out_state),
                    }
                    self.changed_components[index] = true;
                }
            }
        }
    }

    /// Handle [messages](Message) for a single step of simulation.
    pub fn handle_messages(&mut self, mut done_counter: i32) {
        if done_counter == 0 {return;}

        let mut output_rx = self.output_rx.take().expect("Has to have output reciever");
        'outer_loop:
        loop {
            for m in &mut output_rx {
                match m {
                    Message::Step | Message::Die => {}
                    Message::Done(_component_id) => {
                        done_counter -= 1;
                        if done_counter == 0 {break 'outer_loop;}
                    }
                    Message::PinChange(component_id, pin_id, state) => {
                        let index = self.pin_mapping[component_id.0][pin_id as usize];
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
        let mut output_changes = HashMap::new();
        let mut global_output_changes = HashMap::new();
        for (component_id, c) in self.components.iter_mut().enumerate() {
            if !self.changed_components[component_id] {continue;}
            match c {
                ComponentData::Threaded(c) => c.notify_step(),
                ComponentData::Threadless(c) => {
                    output_changes.clear();
                    c.execute_step(&mut output_changes);
                    for (&pin_id, &state) in &output_changes {
                        let index = self.pin_mapping[component_id][pin_id as usize];
                        global_output_changes.insert(index, state);
                    }
                }
            }
        }

        let mut done_counter = 0;
        for i in 0..self.components.len() {
            if self.changed_components[i] {
                if let ComponentData::Threaded(_) = self.components[i] {
                    done_counter += 1;
                }
            }
            self.changed_components[i] = false;
        }

        for (index, state) in global_output_changes {
            self.set_pin(index, state);
        }
        
        self.handle_messages(done_counter);
    }

    /// Run the simulation for specified number of cycles.
    pub fn simulate(&mut self, cycles: u64) {
        use indicatif::ProgressBar;

        self.vcd_writer.write_header();
        let progress = if cycles < 1000 {
                ProgressBar::hidden()
            } else {
                ProgressBar::new(cycles)
            };
        for _ in 0..cycles {
            self.toggle_clock();
            self.vcd_writer.write_step();
            self.toggle_clock();
            self.vcd_writer.write_step();
            progress.inc(1);
        }
        progress.finish();
    }
}