use stm32f4xx_hal::pac::{spi1::i2spr::ODD_A, SPI2, SPI3};

pub struct I2sTransmitter {
    spi: SPI3,
}

impl I2sTransmitter {
    pub fn new(spi: SPI3) -> Self {
        spi.i2spr.write(|w| w.mckoe().enabled()
                                     .i2sdiv().variant(3)
                                     .odd().odd());
        spi.i2scfgr.write(|w| w.i2smod().i2smode()
                                       .i2sstd().msb()
                                       .i2scfg().master_tx()
                                       .datlen().thirty_two_bit()
                                       .chlen().thirty_two_bit()
                                       .i2se().enabled());
        
        I2sTransmitter{spi}
    }
    
    pub fn send_i2s_data(&mut self, data: u16) {
        self.spi.dr.write(|w| w.dr().variant(data));
    }

    pub fn is_busy(&self) -> bool {
        self.spi.sr.read().txe().bit_is_clear()
    }
}


pub struct PdmReceiver {
    spi: SPI2,
}

impl PdmReceiver {
    pub fn new(spi: SPI2) -> Self {
        spi.i2spr.write(|w| w.mckoe().enabled()
                                     .i2sdiv().variant(6)
                                     .odd().even());
        spi.i2scfgr.write(|w| w.i2smod().i2smode()
                                       .i2sstd().msb()
                                       .i2scfg().master_rx()
                                       .datlen().twenty_four_bit()
                                       .chlen().thirty_two_bit()
                                       .i2se().enabled());
        
        PdmReceiver{spi}
    }
    
    pub fn receive_pdm_data(&mut self) -> u16 {
        self.spi.dr.read().dr().bits()
    }

    pub fn is_ready(&self) -> bool {
        self.spi.sr.read().rxne().is_not_empty()
    }
}
