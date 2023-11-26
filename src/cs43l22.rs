use embedded_hal::{digital::v2::OutputPin, blocking::i2c};
use stm32f4xx_hal::{spi, i2s};
use crate::i2s::I2s;

pub struct CS43L22<P, I, S>
where P: OutputPin, I: i2c::Write + i2c::WriteRead, S: spi::Instance + i2s::Instance {
    address: u8,
    reset: P,
    i2c: I,
    i2s: I2s<S>,
}

const HEADPHONE_ON_SPEAKER_OFF: u8 = 0xAF;
const I2S_24BIT_DATA_FORMAT: u8 = 0x04;

enum CS43Regs {
    //ID = 0x01,
    PowerCtrl1 = 0x02,
    PowerCtrl2 = 0x03,
    InterfaceControl = 0x06,
    MasterVolumeA = 20,
    MasterVolumeB = 21,
    VolumeA = 0x22,
    VolumeB = 0x23,
    InitReg1 = 0x00,
    InitReg2 = 0x47,
    InitReg3 = 0x32,
}

impl<P, I, S> CS43L22<P, I, S>
where P: OutputPin, I: i2c::Write + i2c::WriteRead, S: spi::Instance + i2s::Instance {
    pub fn new(reset: P, i2c: I, address: u8, i2s: I2s<S>) -> Self {
        CS43L22 {address, reset, i2c, i2s}
    }

    fn set_reset(&mut self, reset: bool) {
        if reset { let _ = self.reset.set_low(); }
        else { let _ = self.reset.set_high(); };
    }

    fn read_register(&mut self, register: CS43Regs, buffer: &mut [u8]) -> bool {
        if let Ok(_) = self.i2c.write_read(self.address, &[register as u8], buffer) {
            return true;
        };
        false
    }

    fn write_register(&mut self, register: CS43Regs, value: u8) -> bool {
        if let Ok(_) = self.i2c.write(self.address, &[register as u8, value]) {
            return true;
        };
        false
    }

    fn init_settings(&mut self) {
        /* Sequence is given in datasheet of CS43L22 section 4.11 */
        self.write_register(CS43Regs::InitReg1, 0x99);
        self.write_register(CS43Regs::InitReg2, 0x80);
        let mut reg_32_val = [0];
        self.read_register(CS43Regs::InitReg3, &mut reg_32_val);
        reg_32_val[0] |= 0x80;
        self.write_register(CS43Regs::InitReg3, reg_32_val[0]);
        self.write_register(CS43Regs::InitReg1, 0x00);
    }

    pub fn initialize(&mut self) {
        self.set_reset(false);
        self.init_settings();
        self.write_register(CS43Regs::PowerCtrl1, 0x9E);
        self.write_register(CS43Regs::PowerCtrl2, HEADPHONE_ON_SPEAKER_OFF);
        self.write_register(CS43Regs::MasterVolumeA, 0);
        self.write_register(CS43Regs::MasterVolumeB, 0);
        self.write_register(CS43Regs::VolumeA, 0);
        self.write_register(CS43Regs::VolumeB, 0);
        self.write_register(CS43Regs::InterfaceControl, I2S_24BIT_DATA_FORMAT);        
    }

    pub fn get_volume(&mut self) -> u8 {
        let mut volume = [0u8];
        self.read_register(CS43Regs::MasterVolumeA, &mut volume);
        volume[0]
    }

    pub fn change_volume(&mut self, volume_change: i8) {
        let volume = (self.get_volume() as i8 + volume_change) as u8;
        self.write_register(CS43Regs::MasterVolumeA, volume);
        self.write_register(CS43Regs::MasterVolumeB, volume);
    }

    pub fn power_down(&mut self) {
        self.write_register(CS43Regs::PowerCtrl1, 0x9F);
        self.set_reset(true);
    }
}