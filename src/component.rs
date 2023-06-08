use std::collections::HashMap;

use crate::{board::{ComponentId, Message, Board}, pins::{PinId, PinState}, vcr::{fillers::VcrFiller, VcrTree, config::VcrConfig, MutexVcrTree}};
use kanal;

pub trait Component: Send + VcrFiller {
    fn pin_count() -> usize;
    fn get_name(&self) -> &str;
    fn advance(&mut self);
    fn set_pin(&mut self, pin: PinId, state: PinState);
    fn fill_output_changes(&mut self, changes: &mut HashMap<PinId, PinState>);

    fn execute_loop(&mut self,
                    id: ComponentId,
                    output_tx: kanal::Sender<Message>,
                    input_rx: kanal::Receiver<Message>,
                    vcr: MutexVcrTree) {
        let mut output_changes = HashMap::new();
        for m in input_rx {
            println!("Component {:?} got a message: {:?}", id, m);
            match m {
                Message::Die => break,
                Message::PinChange(_, pin, state) => self.set_pin(pin, state),
                Message::Done(_) => {},
                Message::Step => {
                    output_changes.clear();
                    self.advance();
                    self.fill_output_changes(&mut output_changes);
                    self.fill_vcr(&mut vcr.lock().expect("Coundn't take mutex"));
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
    pub fn add_component<T>(&mut self, component: T, name: &str, config: &VcrConfig) -> ComponentId
    where
        T: Component + 'static
    {
        let vcr_init: VcrTree = component.init_vcr(config);
        self.add_component_fn(move |id, output_tx, input_rx, vcr| {
            let mut c = component;
            c.execute_loop(id, output_tx, input_rx, vcr);
        }, vcr_init, name, T::pin_count() as u16)
    }
}
