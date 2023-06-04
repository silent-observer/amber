use std::marker::PhantomData;

use super::mcu_model::McuModel;

pub struct IoController<M: McuModel> {
    model: PhantomData<M>
}

impl<M: McuModel> IoController<M> {
    pub fn new() -> IoController<M> {
        IoController { model: PhantomData }
    }

    pub fn read_internal_u8(&self, id: u8) -> u8 {
        todo!()
    }

    pub fn read_external_u8(&self, addr: u16) -> u8 {
        todo!()
    }

    pub fn write_internal_u8(&mut self, id: u8, val: u8) {
        todo!()
    }

    pub fn write_external_u8(&mut self, addr: u16, val: u8) {
        todo!()
    }
}