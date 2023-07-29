use embedded_hal::{digital::v2::OutputPin, blocking::i2c::{Write, WriteRead}};

pub struct CS43L22<P, I> where P: OutputPin, I: Write + WriteRead {
    address: u8,
    reset: P,
    i2c: I,
}

impl<P, I> CS43L22<P, I> where P: OutputPin, I: Write + WriteRead {
    pub fn new(reset: P, i2c: I, address: u8) -> CS43L22<P, I> {
        CS43L22 {address, reset, i2c}
    }

    pub fn reset_on_off(&mut self, reset: bool) {
        if reset { let _ = self.reset.set_low(); }
        else { let _ = self.reset.set_high(); };
    }

    pub fn read_register(&mut self, register: u8, buffer: &mut [u8]) -> bool {
        if let Ok(_) = self.i2c.write_read(self.address, &[register], buffer) {
            return true;
        };
        false
    }

    pub fn write_register(&mut self, register: u8, value: u8) -> bool {
        if let Ok(_) = self.i2c.write(self.address, &[register, value]) {
            return true;
        };
        false
    }
}