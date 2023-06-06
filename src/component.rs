use crate::{board::{ComponentId, Message, Board}, pins::{PinId, PinState}};
use kanal;

pub trait Component: Send {
    fn pin_count() -> usize;
    fn advance(&mut self);
    fn set_pin(&mut self, pin: PinId, state: PinState);
    fn get_pin_output_changes(&self) -> &[(PinId, PinState)];
    fn reset_pins(&mut self);

    fn execute_loop(&mut self,
                    id: ComponentId,
                    output_tx: kanal::Sender<Message>,
                    input_rx: kanal::Receiver<Message>) {
        for m in input_rx {
            match m {
                Message::Die => break,
                Message::PinChange(_, pin, state) => self.set_pin(pin, state),
                Message::Done => {},
                Message::Step => {
                    self.advance();
                    for &(pin, state) in self.get_pin_output_changes() {
                        output_tx.send(Message::PinChange(id, pin, state))
                                 .expect("Cannot send update");
                    }
                    self.reset_pins();
                    output_tx.send(Message::Done).expect("Cannot send update");
                },
            }
        }
    }
}

impl Board {
    pub fn add_component<T>(&mut self, component: T) -> ComponentId
    where
        T: Component + 'static
    {
        self.add_component_fn(move |id, output_tx, input_rx| {
            let mut c = component;
            c.execute_loop(id, output_tx, input_rx);
        }, T::pin_count() as u16)
    }
}
