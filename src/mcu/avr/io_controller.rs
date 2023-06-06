use std::marker::PhantomData;
use mockall::*;

use super::mcu_model::McuModel;

#[automock]
pub trait IoControllerTrait {
    fn read_internal_u8(&self, id: u8) -> u8;
    fn read_external_u8(&self, addr: u16) -> u8;
    fn write_internal_u8(&mut self, id: u8, val: u8);
    fn write_external_u8(&mut self, addr: u16, val: u8);
}

pub struct IoController<M: McuModel> {
    model: PhantomData<M>
}

impl<M: McuModel + 'static> IoController<M>{
    pub fn new() -> IoController<M> {
        IoController { model: PhantomData }
    }
}

impl<M: McuModel + 'static> IoControllerTrait for IoController<M> {
    fn read_internal_u8(&self, id: u8) -> u8 {
        todo!()
    }

    fn read_external_u8(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write_internal_u8(&mut self, id: u8, val: u8) {
        todo!()
    }

    fn write_external_u8(&mut self, addr: u16, val: u8) {
        todo!()
    }
}