use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, BinaryHeap};
use std::rc::Rc;
use std::thread::{JoinHandle, self};
use kanal;

use crate::component::{Component, ComponentId, Message, ThreadlessComponent, ExecuteStepResult};
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

    is_changed: bool,
}

impl ThreadedComponentData {
    /// Send [Message::PinChange].
    fn notify_on_pin_change(&mut self, pin: PinId, state: PinState) {
        self.is_changed = true;
        self.input_tx
            .send(Message::PinChange(self.id, pin, state))
            .expect("Error sending update");
    }

    /// Send [Message::Step].
    fn notify_step(&self,time_ns: f64) {
        self.input_tx
            .send(Message::Step(time_ns))
            .expect("Error sending update");
    }
}
/// When a handle is dropped, the corresponding component thread is stopped automatically.
impl Drop for ThreadedComponentData {
    fn drop(&mut self) {
        println!("Dropping {}", self.id.0);
        self.input_tx.send(Message::Die).expect("Error sending update");
        self.thread.take().unwrap().join().unwrap();
    }
}

struct ThreadlessComponentData {
    id: ComponentId,
    /// Component data
    component: Box<dyn ThreadlessComponent>,
    /// VCD subtree
    vcd: Rc<RefCell<VcdTreeHandle>>,
}

impl ThreadlessComponentData {
    /// Send [Message::PinChange].
    fn set_pin(&mut self, pin: PinId, state: PinState) {
        self.component.set_pin(pin, state);
    }

    /// Send [Message::ClockRising].
    fn clock_rising_edge(&mut self) -> ExecuteStepResult {
        self.component.clock_rising_edge_threadless(&self.vcd)
    }

    /// Send [Message::ClockFalling].
    fn clock_falling_edge(&mut self) -> ExecuteStepResult {
        self.component.clock_falling_edge_threadless(&self.vcd)
    }
}

struct CommonComponentData {
    index: usize,
    is_threaded: bool,
    pins: Vec<PinIndex>
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

#[derive(Debug, Clone, Copy)]
pub struct PingEvent(ComponentId, f64);

impl PartialEq for PingEvent {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl Eq for PingEvent {}

impl Ord for PingEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        f64::total_cmp(&self.1, &other.1)
    }
}

impl PartialOrd for PingEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Top-level element of a simulation. A board containing multiple components.
pub struct Board {
    threaded_components: Vec<ThreadedComponentData>,
    threaded_components_changed: Vec<usize>,
    threadless_components: Vec<ThreadlessComponentData>,

    /// Channel for recieving messages from the component.
    output_rx: Option<kanal::Receiver<Message>>,
    /// Transmitter corresponding to `output_rx`, to be cloned into every new component.
    output_tx: Option<kanal::Sender<Message>>,
    /// Vector of all pins. Indexed by [PinIndex]
    pins: Vec<Pin>,
    /// Vector of all wires. Indexed by [WireId]
    wires: Vec<Wire>,
    /// Binary heap for storing all the pending events.
    events: BinaryHeap<PingEvent>,

    common_component_data: Vec<CommonComponentData>,

    clock_pin: bool,

    /// A VCD file writer.
    vcd_writer: VcdWriter,

    /// Nanoseconds per step.
    clock_period: f64,
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
        Board {
            output_rx: Some(output_rx),
            output_tx: Some(output_tx),
            threaded_components: Vec::new(),
            threaded_components_changed: Vec::new(),
            threadless_components: Vec::new(),
            pins: Vec::new(),
            wires: Vec::new(),
            common_component_data: Vec::new(),
            clock_pin: false,
            
            vcd_writer: VcdWriter::new(vcd_path),
            clock_period: 5e8 / freq,
            events: BinaryHeap::new(),
        }
    }

    fn add_pins(&mut self, pins_count: u16, component_id: ComponentId) {
        let pins = &mut self.common_component_data[component_id.0].pins;
        for id in 0..pins_count {
            pins.push(self.pins.len());
            self.pins.push(Pin {
                id,
                component: Some(component_id),
                wire: None,
                out_state: PinState::Z,
            });
        }
    }

    /// Adds a [Component] to the [Board] with the specified [VcdConfig].
    pub fn add_component_threaded<T>(&mut self, component: T, name: &str, config: &VcdConfig) -> ComponentHandle
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
        let component_id = ComponentId(self.common_component_data.len());
        let vcd_mutex = self.vcd_writer.add_threaded(name, vcd_init);
        let pins_count = T::pin_count() as PinId;

        self.common_component_data.push(CommonComponentData { 
            index: self.threaded_components.len(),
            is_threaded: true,
            pins: Vec::with_capacity(pins_count as usize),
        });

        let thread = thread::Builder::new().name(name.to_string()).spawn(move || {
            let mut c = component;
            c.execute_loop(component_id, output_tx_copy, input_rx, vcd_mutex);
        }).unwrap();
        self.add_pins(pins_count, component_id);

        let mut c = ThreadedComponentData {
            id: component_id,
            thread: Some(thread),
            input_tx,
            is_changed: true,
        };

        for id in 0..pins_count {
            c.notify_on_pin_change(id, PinState::Z);
        }
        self.threaded_components.push(c);
        
        ComponentHandle {
            id: component_id,
            pin_name_lookup,
        }
    }

    /// Adds a [Component] to the [Board] with the specified [VcdConfig].
    pub fn add_component_clocked<T>(&mut self, component: T, name: &str, config: &VcdConfig) -> ComponentHandle
    where
        T: Component + 'static
    {
        let vcd_init: VcdTree = component.init_vcd(config);
        let pin_name_lookup = T::get_pin_name_lookup();

        let component_id = ComponentId(self.common_component_data.len());
        let vcd = self.vcd_writer.add_threadless(name, vcd_init);
        let pins_count = T::pin_count() as PinId;

        self.common_component_data.push(CommonComponentData { 
            index: self.threadless_components.len(),
            is_threaded: false,
            pins: Vec::with_capacity(pins_count as usize),
        });

        self.add_pins(pins_count, component_id);

        let mut c = ThreadlessComponentData {
            id: component_id,
            component: Box::new(component),
            vcd,
        };
        for id in 0..pins_count {
            c.set_pin(id, PinState::Z);
        }

        self.threadless_components.push(c);

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
            let index = self.common_component_data[component_id.0].pins[pin_id as usize];
            let pin = &mut self.pins[index];
            wire.pins.push(index);
            wire.counter.add(pin.out_state);
            assert!(pin.wire.is_none(), "Cannot connect two wires to the same pin!");
            pin.wire = Some(wire_id);
        }
        let wire_state = wire.read();
        self.wires.push(wire);
        for &(ComponentId(component_id), pin_id) in pins {
            let data = &self.common_component_data[component_id];
            if data.is_threaded {
                self.threaded_components[data.index].notify_on_pin_change(pin_id, wire_state);
            } else {
                self.threadless_components[data.index].set_pin(pin_id, wire_state);
            }
        }
        wire_id
    }

    /// Set a new pin state and propagate the updates through wires.
    pub fn set_pin(&mut self,
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
                    let data = &mut self.common_component_data[index];
                    if data.is_threaded {
                        if !self.threaded_components_changed.contains(&data.index) {
                            self.threaded_components_changed.push(data.index);
                        }
                        self.threaded_components[data.index].notify_on_pin_change(pin_id, new_out_state)
                    } else {
                        self.threadless_components[data.index].set_pin(pin_id, new_out_state)
                    }
                }
            }
        }
    }

    /// Handle [messages](Message) for a single step of simulation.
    fn handle_messages(&mut self, mut done_counter: i32) {
        if done_counter == 0 {return;}

        let mut output_rx = self.output_rx.take().expect("Has to have output reciever");
        'outer_loop:
        loop {
            for m in &mut output_rx {
                match m {
                    Message::Step(_) |
                    Message::ClockFalling |
                    Message::ClockRising |
                    Message::Die => panic!("This shouldn't happen"),
                    Message::Done(component_id, vcd_changed) => {
                        done_counter -= 1;
                        if vcd_changed {
                            self.vcd_writer.set_change(component_id.0);
                        }
                        if done_counter == 0 {break 'outer_loop;}
                    }
                    Message::PinChange(component_id, pin_id, state) => {
                        let index = self.common_component_data[component_id.0].pins[pin_id as usize];
                        self.set_pin(index, state);
                    }
                    Message::PingMeAt(id, time) => {
                        self.events.push(PingEvent(id, time));
                    },
                }
            }
        }
        self.output_rx = Some(output_rx);
    }

    #[inline]
    fn handle_events(&mut self, current_time: f64) {
        // Check if there are any pending events ready
        while let Some(PingEvent(id, time_ns)) = self.events.peek().copied() {
            if time_ns <= current_time {
                self.events.pop();
                let data = &self.common_component_data[id.0];
                if data.is_threaded {
                    self.threaded_components[data.index].is_changed = true;
                }
            } else {
                break;
            }
        }
    }

    /// Toggle a clock pin once.
    /// 
    /// Returns whether VCD has changed.
    #[inline]
    fn toggle_clock(&mut self, global_output_changes: &mut Vec<(PinIndex, PinState)>, time_ns: f64) {
        self.clock_pin = !self.clock_pin;

        self.handle_events(time_ns);

        global_output_changes.clear();
        let done_counter = self.threaded_components_changed.len() as i32;
        for index in self.threaded_components_changed.drain(..) {
            self.threaded_components[index].notify_step(time_ns)
        }
        for c in self.threadless_components.iter_mut() {
            let id = c.id.0;
            let result =
                if self.clock_pin {
                    c.clock_rising_edge()
                } else {
                    c.clock_falling_edge()
                };

            if let Some(time_ns) = result.time_ns {
                self.events.push(PingEvent(ComponentId(id), time_ns));
            }

            if result.changed {
                self.vcd_writer.set_change(id);
            }
            for &(pin_id, state) in result.output_changes.iter() {
                let index = self.common_component_data[id].pins[pin_id as usize];
                global_output_changes.push((index, state));
            }
        }

        for &(index, state) in global_output_changes.iter() {
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
                ProgressBar::new(cycles / 1_000_000)
            };
        
        let mut global_output_changes = Vec::new();

        let mut time_ns = 0.0;
        for i in 0..cycles*2 {
            self.toggle_clock(&mut global_output_changes, time_ns);
            self.vcd_writer.write_step(time_ns + self.clock_period);
            time_ns += self.clock_period;
            if (i+1) % 2_000_000 == 0 {
                progress.inc(1);
            }
        }
        progress.finish();
    }
}