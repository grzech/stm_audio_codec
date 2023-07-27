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
};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let gpiod = dp.GPIOD.split();
    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let clocks = dp.RCC.constrain().cfgr.use_hse(8.MHz()).freeze();
 
    let mut blue = gpiod.pd15.into_push_pull_output();
    let mut red = gpiod.pd14.into_push_pull_output();
    let mut orange = gpiod.pd13.into_push_pull_output();
    let mut green = gpiod.pd12.into_push_pull_output();
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
    
    let mut amp_reset = gpiod.pd4.into_push_pull_output();
    let sda = gpiob.pb9;
    let scl = gpiob.pb6;
    let mut i2c1 = I2c::new(dp.I2C1, (scl, sda), Mode::standard(100.kHz()), &clocks);

    blue.set_high();
    red.set_high();
    orange.set_high();
    green.set_high();
    writeln!(&mut usart, "Peripherals are initialized").unwrap();
    blue.set_low();
    red.set_low();
    orange.set_low();
    green.set_low();
    
    amp_reset.set_high();
    let mut data = [0];
    if let Ok(_) = i2c1.write_read(0x4A, &[0x01], &mut data) {
        writeln!(&mut usart, "Succesfully read data from I2C1: \"{}\"", data[0]).unwrap();
    } else {
        writeln!(&mut usart, "Couldn't read data from I2C1").unwrap();
    };

    writeln!(&mut usart, "Entering mainloop").unwrap();
    loop {        
        blue.toggle();
    };
}
