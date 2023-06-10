use std::collections::HashMap;

use crate::vcd::{VcdFiller, MutexVcdTree};
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
    Step,
    /// Component to Board: sent after Step is done.
    Done(ComponentId),
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
    /// Get updates of output pin values.
    /// 
    /// Updates must be added into the `changes` [HashMap].
    fn fill_output_changes(&mut self, changes: &mut HashMap<PinId, PinState>);
    /// Advance the simulation through one step.
    /// 
    /// After this step all the pin value changes must be accounted for.
    fn advance(&mut self);

    /// Execute a single step and output all changes
    fn execute_step(&mut self, vcd: &MutexVcdTree, output_changes: &mut HashMap<PinId, PinState>) {
        output_changes.clear();
        self.advance();
        self.fill_output_changes(output_changes);
        self.fill_vcd(&mut vcd.lock().expect("Coundn't take mutex"));
    }

    /// Main loop of the component.
    /// Reacts to [messages](Message) passed through kanal channels.
    fn execute_loop(&mut self,
                    id: ComponentId,
                    output_tx: kanal::Sender<Message>,
                    input_rx: kanal::Receiver<Message>,
                    vcd: MutexVcdTree) {
        let mut output_changes = HashMap::new();
        for m in input_rx {
            match m {
                Message::Die => break,
                Message::PinChange(_, pin, state) => self.set_pin(pin, state),
                Message::Done(_) => {},
                Message::Step => {
                    self.execute_step(&vcd, &mut output_changes);
                    for (&pin, &state) in output_changes.iter() {
                        output_tx.send(Message::PinChange(id, pin, state))
                                 .expect("Cannot send update");
                    }
                    output_tx.send(Message::Done(id))
                             .expect("Cannot send update");
                },
            }
        }
    }
}

pub trait ThreadlessComponent {
    fn set_pin(&mut self, pin: PinId, state: PinState);
    fn execute_step(&mut self, vcd: &MutexVcdTree, output_changes: &mut HashMap<PinId, PinState>);
}

impl<T: Component> ThreadlessComponent for T {
    fn set_pin(&mut self, pin: PinId, state: PinState) {
        self.set_pin(pin, state);
    }

    fn execute_step(&mut self, vcd: &MutexVcdTree, output_changes: &mut HashMap<PinId, PinState>) {
        self.execute_step(vcd, output_changes);
    }
}