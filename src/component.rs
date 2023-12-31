use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::vcd::{VcdFiller, VcdTreeHandle, VcdTree};
use crate::pins::{PinId, PinState};
use kanal;

/// A unique identifier for a component.
#[derive(Debug, Clone, Copy)]
pub struct ComponentId(pub usize);

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
    Step(f64),
    ClockRising,
    ClockFalling,
    
    PingMeAt(ComponentId, f64),
    /// Component to Board: sent after Step is done. Contains whether VCD has changed
    Done(ComponentId, bool),
    /// Board to Component: stop component thread.
    Die,
    /// Board to Component: notify component about a pin changing state.
    PinChange(ComponentId, PinId, PinState),
}

/// Top level component which can be placed on the [Board].
/// 
/// Every component is run in a separate thread, and must implement [VcdFiller] trait
/// to extract VCD data.
/// Implement this to make something addable to the simulation.
pub trait Component: Send + VcdFiller {
    /// Total number of external pins in the component.
    fn pin_count() -> usize;
    /// Find a pin given a name.
    fn pin_name(pin: PinId) -> String;

    /// Get lookup table
    fn get_pin_name_lookup() -> HashMap<String, u16> {
        let mut pin_name_lookup = HashMap::new();
        for pin in 0..Self::pin_count() {
            pin_name_lookup.insert(Self::pin_name(pin as u16), pin as u16);
        }
        pin_name_lookup
    }

    /// Set external pin value.
    /// 
    /// Component can use data set through this method as input.
    fn set_pin(&mut self, pin: PinId, state: PinState);
    
    fn clock_rising_edge(&mut self);
    fn clock_falling_edge(&mut self);
    /// Get updates of output pin values.
    /// 
    /// Updates must be added into the `changes` [HashMap].
    fn get_output_changes(&mut self) -> &[(PinId, PinState)];
    /// Advance the simulation through one step.
    /// 
    /// After this step all the pin value changes must be accounted for.
    fn advance(&mut self, time_ns: f64) -> Option<f64>;

    /// Execute a single step and output all changes
    /// 
    /// Returns whether VCD have changed
    
    fn fill_everything_threadless(&mut self, vcd: &Rc<RefCell<VcdTreeHandle>>) -> (bool, &[(PinId, PinState)]) {
        let borrowed = &mut *vcd.borrow_mut();
        let vcd_changed = match &mut borrowed.tree {
            VcdTree::Disabled => {false},
            tree => self.fill_vcd(tree),
        };
        (vcd_changed, self.get_output_changes())
    }

    fn fill_everything_threaded(&mut self, vcd: &Arc<Mutex<VcdTreeHandle>>) -> (bool, &[(PinId, PinState)]) {
        let guard = &mut *vcd.lock().expect("Coundn't take mutex");
        let vcd_changed = match &mut guard.tree {
            VcdTree::Disabled => {false},
            tree => self.fill_vcd(tree),
        };
        (vcd_changed, self.get_output_changes())
    }

    /// Main loop of the component.
    /// Reacts to [messages](Message) passed through kanal channels.
    fn execute_loop(&mut self,
                    id: ComponentId,
                    output_tx: kanal::Sender<Message>,
                    input_rx: kanal::Receiver<Message>,
                    vcd: Arc<Mutex<VcdTreeHandle>>) {
        for m in input_rx {
            match m {
                Message::Die => break,
                Message::PinChange(_, pin, state) => self.set_pin(pin, state),
                Message::Done(_, _) | Message::PingMeAt(_, _) => {},
                Message::Step(_) | Message::ClockRising | Message::ClockFalling => {
                    let ping = match m {
                        Message::Step(x) => self.advance(x),
                        Message::ClockRising => {self.clock_rising_edge(); None},
                        Message::ClockFalling => {self.clock_falling_edge(); None},
                        _ => panic!("Impossible!")
                    };
                    let (changed, output_changes) = self.fill_everything_threaded(&vcd);
                    for &(pin, state) in output_changes.iter() {
                        output_tx.send(Message::PinChange(id, pin, state))
                                 .expect("Cannot send up .date");
                    }
                    if let Some(time) = ping {
                        output_tx.send(Message::PingMeAt(id, time))
                                 .expect("Cannot send update");
                    }
                    output_tx.send(Message::Done(id, changed))
                             .expect("Cannot send update");
                },
            }
        }
    }
}

pub struct ExecuteStepResult<'a> {
    pub changed: bool,
    pub output_changes: &'a [(PinId, PinState)],
    pub time_ns: Option<f64>
}

pub trait ThreadlessComponent {
    fn set_pin(&mut self, pin: PinId, state: PinState);
    fn execute_step_threadless(&mut self, vcd: &Rc<RefCell<VcdTreeHandle>>, time_ns: f64) -> ExecuteStepResult;
    fn clock_rising_edge_threadless(&mut self, vcd: &Rc<RefCell<VcdTreeHandle>>) -> ExecuteStepResult;
    fn clock_falling_edge_threadless(&mut self, vcd: &Rc<RefCell<VcdTreeHandle>>) -> ExecuteStepResult;
}

impl<T: Component> ThreadlessComponent for T {
    fn set_pin(&mut self, pin: PinId, state: PinState) {
        self.set_pin(pin, state);
    }

    fn execute_step_threadless(&mut self, vcd: &Rc<RefCell<VcdTreeHandle>>, time_ns: f64) -> ExecuteStepResult {
        let ping = self.advance(time_ns);
        let (changed, output_changes) = self.fill_everything_threadless(vcd);
        ExecuteStepResult {
            changed,
            output_changes,
            time_ns: ping,
        }
    }

    fn clock_rising_edge_threadless(&mut self, vcd: &Rc<RefCell<VcdTreeHandle>>) -> ExecuteStepResult {
        self.clock_rising_edge();
        let (changed, output_changes) = self.fill_everything_threadless(vcd);
        ExecuteStepResult {
            changed,
            output_changes,
            time_ns: None,
        }
    }

    fn clock_falling_edge_threadless(&mut self, vcd: &Rc<RefCell<VcdTreeHandle>>) -> ExecuteStepResult {
        self.clock_falling_edge();
        let (changed, output_changes) = self.fill_everything_threadless(vcd);
        ExecuteStepResult {
            changed,
            output_changes,
            time_ns: None,
        }
    }
}