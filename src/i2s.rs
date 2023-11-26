use stm32f4xx_hal::{i2s, rcc::Clocks, spi};

pub struct I2s<SPI: spi::Instance + i2s::Instance>
{
    i2s: SPI,
    clock: u32,
}

impl<SPI: spi::Instance + i2s::Instance> I2s<SPI> {
    pub fn new(
        spi: SPI,
        _pins: (impl Into<SPI::Ws>, impl Into<SPI::Ck>, impl Into<SPI::Mck>, impl Into<SPI::Sd>),
        clocks: & Clocks,) -> Self {
        I2s{i2s: spi, clock: clocks.pclk1().raw()}
    }

    pub fn configure(&mut self, baud_rate: u32) {
        let prescaler = self.clock/baud_rate as u32;
        self.i2s.cr2.write(|w| w.txdmaen().enabled());
        self.i2s.i2scfgr.write(|w| w.i2smod().i2smode()
                                            .i2scfg().master_tx()
                                            .i2sstd().msb()
                                            .ckpol().idle_high()
                                            .datlen().twenty_four_bit()
                                            .chlen().thirty_two_bit());
        self.i2s.i2spr.write(|w| w.mckoe().enabled()
                                          .odd().odd()
                                          .i2sdiv().variant(prescaler as u8));
        self.i2s.i2scfgr.write(|w| w.i2se().enabled());
    }
}