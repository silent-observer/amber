use std::collections::HashMap;

use crate::vcd::{VcdFiller, VcdTree, VcdConfig, MutexVcdTree};
use crate::pins::{PinId, PinState};
use crate::board::{ComponentId, Message, Board, ComponentHandle};
use kanal;

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
                    output_changes.clear();
                    self.advance();
                    self.fill_output_changes(&mut output_changes);
                    self.fill_vcd(&mut vcd.lock().expect("Coundn't take mutex"));
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

impl Board {
    /// Adds a [Component] to the [Board] with the specified [VcdConfig].
    pub fn add_component<T>(&mut self, component: T, name: &str, config: &VcdConfig) -> ComponentHandle
    where
        T: Component + 'static
    {
        let vcd_init: VcdTree = component.init_vcd(config);
        let mut pin_name_lookup = HashMap::new();
        for pin in 0..T::pin_count() {
            pin_name_lookup.insert(T::pin_name(pin as u16), pin as u16);
        }
        self.add_component_fn(move |id, output_tx, input_rx, vcd| {
            let mut c = component;
            c.execute_loop(id, output_tx, input_rx, vcd);
        }, vcd_init, name, T::pin_count() as u16, pin_name_lookup)
    }
}
