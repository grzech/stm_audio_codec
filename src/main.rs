#![no_std]
#![no_main]

use core::fmt::Write;

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4xx_hal::{
    pac::{self},
    prelude::*,
    serial::Config,
    i2c::{Mode, I2c},
    pac::RCC,
    rcc::Clocks,
};

mod cs43l22;
mod i2s;
use cs43l22::CS43L22;
use i2s::{I2sTransmitter, PdmReceiver};

const SAMPLE_RATE: u32 = 48_000;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let gpiod = dp.GPIOD.split();
    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    let clocks = configure_clocks(dp.RCC);
    let mut blue = gpiod.pd15.into_push_pull_output();
    let mut red = gpiod.pd14.into_push_pull_output();
    let mut orange = gpiod.pd13.into_push_pull_output();
    let mut green = gpiod.pd12.into_push_pull_output();

    blue.set_low();
    red.set_low();
    orange.set_low();
    green.set_low();
    let _button = gpioa.pa0;

    let usart_tx = gpioa.pa2.into_alternate::<7>();
    let usart_rx = gpioa.pa3.into_alternate::<7>();
    let mut usart = dp.USART2.serial((usart_tx, usart_rx),
                                                     Config::default()
                                                        .baudrate(9600.bps())
                                                        .parity_none()
                                                        .wordlength_8(),
                                                     &clocks)
                                        .unwrap();
    
    let amp_reset = gpiod.pd4.into_push_pull_output();
    let sda = gpiob.pb9;
    let scl = gpiob.pb6;
    let i2c1 = I2c::new(dp.I2C1, (scl, sda), Mode::standard(100.kHz()), &clocks);
    let mut amp = CS43L22::new(amp_reset, i2c1, 0x4A);
    amp.initialize();
    green.set_high();
    let i2s3_pins = (
        gpioa.pa4.into_push_pull_output().into_alternate::<6>(),
        gpioc.pc10.into_push_pull_output().into_alternate::<6>(),
        gpioc.pc7.into_push_pull_output().into_alternate::<6>(),
        gpioc.pc12.into_push_pull_output().into_alternate::<6>(),
    );
    let i2s2_pins = (
        gpiob.pb10.into_push_pull_output().into_alternate::<5>(),
        gpioc.pc3.into_push_pull_output().into_alternate::<5>(),
    );
    let mut i2s = I2sTransmitter::new(dp.SPI3);
    //let mut pdm = PdmReceiver::new(dp.SPI2);
    red.set_high();
    
    writeln!(&mut usart, "Peripherals are initialized").unwrap();
       
    orange.set_high();
    
    writeln!(&mut usart, "Entering mainloop").unwrap();
    let mut val : u32 = 0;
    loop {
        
        //writeln!(&mut usart, "Send data via I2S").unwrap();
        if !i2s.is_busy() {
            //red.toggle();
            i2s.send_i2s_data((val & 0xFFFF) as u16);
            val += 0x10;
        } else {
            writeln!(&mut usart, "Waiting for flag").unwrap();
        }
        //blue.toggle();
        
    };
}

fn configure_clocks(clocks: RCC) -> Clocks {
    clocks.apb1enr.write(|w| w.spi3en().enabled().usart2en().enabled().i2c1en().enabled());
    clocks.apb2enr.write(|w| w.spi1en().enabled());
    clocks.cr.write(|w| w.plli2son().on());
    clocks.ahb1enr.write(|w| w.dma1en().enabled().dma2en().enabled().gpioaen().enabled()
        .gpioben().enabled().gpioden().enabled().gpiocen().enabled());
    clocks.plli2scfgr.write(|w| w.plli2sn().variant(271).plli2sr().variant(2));
    return clocks.constrain().cfgr
        .use_hse(8.MHz())
        .sysclk(96.MHz())
        .i2s_clk(61440.kHz())
        .freeze();
}