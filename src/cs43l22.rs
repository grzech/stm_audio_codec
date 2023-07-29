use embedded_hal::{digital::v2::OutputPin, blocking::i2c::{Write, WriteRead}};

pub struct CS43L22<P, I> where P: OutputPin, I: Write + WriteRead {
    address: u8,
    reset: P,
    i2c: I,
}

pub enum CS43Regs {
    ID = 0x01,
    PowerCtrl1 = 0x02,
    PowerCtrl2 = 0x03,
    InterfaceControl = 0x06,
    VolumeA = 0x22,
    VolumeB = 0x23,

}

impl<P, I> CS43L22<P, I> where P: OutputPin, I: Write + WriteRead {
    pub fn new(reset: P, i2c: I, address: u8) -> CS43L22<P, I> {
        CS43L22 {address, reset, i2c}
    }

    pub fn reset_on_off(&mut self, reset: bool) {
        if reset { let _ = self.reset.set_low(); }
        else { let _ = self.reset.set_high(); };
    }

    pub fn read_register(&mut self, register: CS43Regs, buffer: &mut [u8]) -> bool {
        if let Ok(_) = self.i2c.write_read(self.address, &[register as u8], buffer) {
            return true;
        };
        false
    }

    pub fn write_register(&mut self, register: CS43Regs, value: u8) -> bool {
        if let Ok(_) = self.i2c.write(self.address, &[register as u8, value]) {
            return true;
        };
        false
    }

    pub fn initialize(&mut self) {

    }
}