use bitfield::Bit;

use crate::pins::{PinId, PinState};

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompareOutputMode {
    Disabled = 0,
    Toggle = 1,
    Clear = 2,
    Set = 3
}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum ClockMode {
    Disabled = 0,
    Clk1 = 1,
    Clk8 = 2,
    Clk64 = 3,
    Clk256 = 4,
    Clk1024 = 5,
    ExternalFalling = 6,
    ExternalRising = 7,
}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum WaveformGenerationMode {
    Normal = 0,
    Pwm8Bit = 1,
    Pwm9Bit = 2,
    Pwm10Bit = 3,

    Ctc = 4,
    FastPwm8Bit = 5,
    FastPwm9Bit = 6,
    FastPwm10Bit = 7,

    PwmPhaseFreqIcr = 8,
    PwmPhaseFreqOcrA = 9,
    PwmPhaseIcr = 10,
    PwmPhaseOcrA = 11,

    CtcIcr = 12,
    Reserved = 13,
    FastPwmIcr = 14,
    FastPwmOcrA = 15,
}

pub struct Timer16Interrupts {
    pub overflow: bool,
    pub oc: [bool; 3],
    pub input_capture: bool,
}

pub struct Timer16 {
    counter: u16,
    pins: [bool; 3],
    reg_ocr: [u16; 3],
    active_ocr: [u16; 3],
    compare_output_mode: [CompareOutputMode; 3],
    pin_ids: [PinId; 3],

    clock_mode: ClockMode,
    waveform_mode: WaveformGenerationMode,

    interrupt_masks: Timer16Interrupts,
    pub interrupt_flags: Timer16Interrupts,
}

impl Timer16 {
    pub fn new(pin_ids: [PinId; 3]) -> Timer16 {
        Timer16 { 
            counter: 0,
            pins: [false; 3],
            reg_ocr: [0, 0, 0],
            active_ocr: [0, 0, 0],
            pin_ids,
            compare_output_mode: [CompareOutputMode::Disabled; 3],
            clock_mode: ClockMode::Disabled,
            waveform_mode: WaveformGenerationMode::Normal,
            interrupt_masks: Timer16Interrupts { 
                overflow: false,
                oc: [false; 3],
                input_capture: false
            },
            interrupt_flags: Timer16Interrupts { 
                overflow: false,
                oc: [false; 3],
                input_capture: false
            }
        }
    }

    #[inline]
    fn top_value(&self) -> u16 {
        match self.waveform_mode {
            WaveformGenerationMode::Normal => 0xFFFF,
            WaveformGenerationMode::Pwm8Bit => 0x00FF,
            WaveformGenerationMode::Pwm9Bit => 0x01FF,
            WaveformGenerationMode::Pwm10Bit => 0x03FF,
            WaveformGenerationMode::Ctc => self.active_ocr[0],
            WaveformGenerationMode::FastPwm8Bit => 0x00FF,
            WaveformGenerationMode::FastPwm9Bit => 0x01FF,
            WaveformGenerationMode::FastPwm10Bit => 0x03FF,
            WaveformGenerationMode::PwmPhaseFreqIcr => todo!(),
            WaveformGenerationMode::PwmPhaseFreqOcrA => self.active_ocr[0],
            WaveformGenerationMode::PwmPhaseIcr => todo!(),
            WaveformGenerationMode::PwmPhaseOcrA => self.active_ocr[0],
            WaveformGenerationMode::CtcIcr => todo!(),
            WaveformGenerationMode::Reserved => unimplemented!(),
            WaveformGenerationMode::FastPwmIcr => todo!(),
            WaveformGenerationMode::FastPwmOcrA => self.active_ocr[0],
        }
    }

    #[inline]
    pub fn update_oc(&mut self, i: usize, output_changes: &mut Vec<(PinId, PinState)>, interrupt: &mut bool) {
        // TODO: This shouldn't work with incorrect DDR
        match self.compare_output_mode[i] {
            CompareOutputMode::Disabled => {},
            CompareOutputMode::Toggle => {
                self.pins[i] = !self.pins[i];
                output_changes.push((self.pin_ids[i], PinState::from_bool(self.pins[i])))
            }
            CompareOutputMode::Clear => {
                self.pins[i] = false;
                output_changes.push((self.pin_ids[i], PinState::Low))
            }
            CompareOutputMode::Set => {
                self.pins[i] = true;
                output_changes.push((self.pin_ids[i], PinState::High))
            }
        }
        if self.interrupt_masks.oc[i] {
            self.interrupt_flags.oc[i] = true;
            *interrupt = true;
        }
    }

    pub fn tick_prescaler(&mut self, prescaler: u16, output_changes: &mut Vec<(PinId, PinState)>, interrupt: &mut bool) {
        let should_tick = match self.clock_mode {
            ClockMode::Disabled => false,
            ClockMode::Clk1 => true,
            ClockMode::Clk8 => prescaler % 8 == 0,
            ClockMode::Clk64 => prescaler % 64 == 0,
            ClockMode::Clk256 => prescaler % 256 == 0,
            ClockMode::Clk1024 => prescaler == 0,
            ClockMode::ExternalFalling => todo!(),
            ClockMode::ExternalRising => todo!(),
        };
        if should_tick {
            self.tick(output_changes, interrupt)
        }
    }

    fn tick(&mut self, output_changes: &mut Vec<(PinId, PinState)>, interrupt: &mut bool) {
        for i in 0..3 {
            if self.active_ocr[i] == self.counter {
                self.update_oc(i, output_changes, interrupt);
            }
        }

        let top = self.top_value();
        if self.counter == top {
            // TODO: Proper PWM
            self.counter = 0;
            match self.waveform_mode {
                WaveformGenerationMode::Pwm8Bit |
                WaveformGenerationMode::Pwm9Bit |
                WaveformGenerationMode::Pwm10Bit |
                WaveformGenerationMode::PwmPhaseIcr |
                WaveformGenerationMode::PwmPhaseOcrA => self.active_ocr = self.reg_ocr,

                WaveformGenerationMode::FastPwm8Bit |
                WaveformGenerationMode::FastPwm9Bit |
                WaveformGenerationMode::FastPwm10Bit |
                WaveformGenerationMode::FastPwmIcr |
                WaveformGenerationMode::FastPwmOcrA if self.interrupt_masks.overflow => {
                    self.interrupt_flags.overflow = true;
                    *interrupt = true;
                }
                _ => {}
            }
        } else {
            if self.counter == 0xFFFF && self.interrupt_masks.overflow {
                match self.waveform_mode {
                    WaveformGenerationMode::Normal |
                    WaveformGenerationMode::Ctc |
                    WaveformGenerationMode::CtcIcr => {
                        self.interrupt_flags.overflow = true;
                        *interrupt = true;
                    }
                    _ => {}
                }
            }
            self.counter = self.counter.wrapping_add(1);
        }
    }

    #[inline]
    pub fn read_tccra(&self) -> u8 {
        (self.compare_output_mode[0] as u8) << 6 |
        (self.compare_output_mode[1] as u8) << 4 |
        (self.compare_output_mode[2] as u8) << 2 |
        (self.waveform_mode as u8) & 0x3
    }
    #[inline]
    pub fn read_tccrb(&self) -> u8 {
        ((self.waveform_mode as u8) & 0xC) << 1 |
        self.clock_mode as u8
    }

    #[inline]
    pub fn read_tcntl(&self) -> u8 {
        self.counter as u8
    }
    #[inline]
    pub fn read_tcnth(&self) -> u8 {
        (self.counter >> 8) as u8
    }

    #[inline]
    pub fn read_ocral(&self) -> u8 {
        self.reg_ocr[0] as u8
    }
    #[inline]
    pub fn read_ocrah(&self) -> u8 {
        (self.reg_ocr[0] >> 8) as u8
    }

    #[inline]
    pub fn read_ocrbl(&self) -> u8 {
        self.reg_ocr[1] as u8
    }
    #[inline]
    pub fn read_ocrbh(&self) -> u8 {
        (self.reg_ocr[1] >> 8) as u8
    }

    #[inline]
    pub fn read_ocrcl(&self) -> u8 {
        self.reg_ocr[2] as u8
    }
    #[inline]
    pub fn read_ocrch(&self) -> u8 {
        (self.reg_ocr[2] >> 8) as u8
    }

    #[inline]
    pub fn read_timsk(&self) -> u8 {
        (self.interrupt_masks.input_capture as u8) << 5 |
        (self.interrupt_masks.oc[2] as u8) << 3 |
        (self.interrupt_masks.oc[1] as u8) << 2 |
        (self.interrupt_masks.oc[0] as u8) << 1 |
        (self.interrupt_masks.overflow as u8)
    }
    #[inline]
    pub fn read_tifr(&self) -> u8 {
        (self.interrupt_flags.input_capture as u8) << 5 |
        (self.interrupt_flags.oc[2] as u8) << 3 |
        (self.interrupt_flags.oc[1] as u8) << 2 |
        (self.interrupt_flags.oc[0] as u8) << 1 |
        (self.interrupt_flags.overflow as u8)
    }

    #[inline]
    pub fn write_tccra(&mut self, val: u8, output_changes: &mut Vec<(PinId, PinState)>, gpio_pins: &mut [(bool, PinState); 86]) {
        unsafe{
            self.compare_output_mode[0] = std::mem::transmute((val >> 6) & 0x3);
            self.compare_output_mode[1] = std::mem::transmute((val >> 4) & 0x3);
            self.compare_output_mode[2] = std::mem::transmute((val >> 2) & 0x3);
            self.waveform_mode = std::mem::transmute(self.waveform_mode as u8 & 0xC | val & 0x3);
        }
        for i in 0..3 {
            let gpio_pin = &mut gpio_pins[self.pin_ids[i] as usize];
            if self.compare_output_mode[i] != CompareOutputMode::Disabled {
                output_changes.push((self.pin_ids[i], PinState::from_bool(self.pins[i])));
                gpio_pin.0 = false;
            } else {
                output_changes.push((self.pin_ids[i], gpio_pin.1));
                gpio_pin.0 = true;
            }
        }
    }
    #[inline]
    pub fn write_tccrb(&mut self, val: u8) {
        unsafe{
            self.waveform_mode = std::mem::transmute(self.waveform_mode as u8 & 0x3 | (val & 0x18) >> 1);
            self.clock_mode = std::mem::transmute(val & 0x7);
        }
    }

    #[inline]
    pub fn write_tcntl(&mut self, val: u8) {
        self.counter = self.counter & 0xFF00 | val as u16;
    }
    #[inline]
    pub fn write_tcnth(&mut self, val: u8) {
        self.counter = self.counter & 0x00FF | (val as u16) << 8;
    }

    #[inline]
    pub fn write_ocral(&mut self, val: u8) {
        self.reg_ocr[0] = self.reg_ocr[0] & 0xFF00 | val as u16;
        match self.waveform_mode {
            WaveformGenerationMode::Normal |
            WaveformGenerationMode::Ctc |
            WaveformGenerationMode::CtcIcr => self.active_ocr[0] = self.reg_ocr[0],
            _ => {}
        }
    }
    #[inline]
    pub fn write_ocrah(&mut self, val: u8) {
        self.reg_ocr[0] = self.reg_ocr[0] & 0x00FF | (val as u16) << 8;
        match self.waveform_mode {
            WaveformGenerationMode::Normal |
            WaveformGenerationMode::Ctc |
            WaveformGenerationMode::CtcIcr => self.active_ocr[0] = self.reg_ocr[0],
            _ => {}
        }
    }


    #[inline]
    pub fn write_ocrbl(&mut self, val: u8) {
        self.reg_ocr[1] = self.reg_ocr[1] & 0xFF00 | val as u16;
        match self.waveform_mode {
            WaveformGenerationMode::Normal |
            WaveformGenerationMode::Ctc |
            WaveformGenerationMode::CtcIcr => self.active_ocr[1] = self.reg_ocr[1],
            _ => {}
        }
    }
    #[inline]
    pub fn write_ocrbh(&mut self, val: u8) {
        self.reg_ocr[1] = self.reg_ocr[1] & 0x00FF | (val as u16) << 8;
        match self.waveform_mode {
            WaveformGenerationMode::Normal |
            WaveformGenerationMode::Ctc |
            WaveformGenerationMode::CtcIcr => self.active_ocr[1] = self.reg_ocr[1],
            _ => {}
        }
    }

    #[inline]
    pub fn write_ocrcl(&mut self, val: u8) {
        self.reg_ocr[2] = self.reg_ocr[2] & 0xFF00 | val as u16;
        match self.waveform_mode {
            WaveformGenerationMode::Normal |
            WaveformGenerationMode::Ctc |
            WaveformGenerationMode::CtcIcr => self.active_ocr[2] = self.reg_ocr[2],
            _ => {}
        }
    }
    #[inline]
    pub fn write_ocrch(&mut self, val: u8) {
        self.reg_ocr[2] = self.reg_ocr[2] & 0x00FF | (val as u16) << 8;
        match self.waveform_mode {
            WaveformGenerationMode::Normal |
            WaveformGenerationMode::Ctc |
            WaveformGenerationMode::CtcIcr => self.active_ocr[2] = self.reg_ocr[2],
            _ => {}
        }
    }

    #[inline]
    pub fn write_timsk(&mut self, val: u8) {
        self.interrupt_masks.input_capture = val.bit(5);
        self.interrupt_masks.oc[2] = val.bit(3);
        self.interrupt_masks.oc[1] = val.bit(2);
        self.interrupt_masks.oc[0] = val.bit(1);
        self.interrupt_masks.overflow = val.bit(0);
    }
    #[inline]
    pub fn write_tifr(&mut self, val: u8) {
        if val.bit(5) {
            self.interrupt_flags.input_capture = false;
        }
        if val.bit(3) {
            self.interrupt_flags.oc[2] = false;
        }
        if val.bit(2) {
            self.interrupt_flags.oc[1] = false;
        }
        if val.bit(1) {
            self.interrupt_flags.oc[0] = false;
        }
        if val.bit(0) {
            self.interrupt_flags.overflow = false;
        }
    }
}