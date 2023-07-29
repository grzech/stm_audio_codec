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
    InitReg1 = 0x00,
    InitReg2 = 0x47,
    InitReg3 = 0x32,
}

impl<P, I> CS43L22<P, I> where P: OutputPin, I: Write + WriteRead {
    pub fn new(reset: P, i2c: I, address: u8) -> CS43L22<P, I> {
        CS43L22 {address, reset, i2c}
    }

    fn set_reset(&mut self, reset: bool) {
        if reset { let _ = self.reset.set_low(); }
        else { let _ = self.reset.set_high(); };
    }

    pub fn read_register(&mut self, register: CS43Regs, buffer: &mut [u8]) -> bool {
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
    }

    pub fn power_down(&mut self) {
        self.set_reset(true);
    }
}