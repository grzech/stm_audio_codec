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
    i2s::I2s,
    spi::Spi3,
};

mod cs43l22;
use cs43l22::CS43L22;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let gpiod = dp.GPIOD.split();
    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
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
    
    let amp_reset = gpiod.pd4.into_push_pull_output();
    let sda = gpiob.pb9;
    let scl = gpiob.pb6;
    let i2c1 = I2c::new(dp.I2C1, (scl, sda), Mode::standard(100.kHz()), &clocks);
    let mck = gpioc.pc7;
    let ck = gpioc.pc10;
    let sd = gpioc.pc12;
    let ws = gpioa.pa4;
    let i2s3 = I2s::new(dp.SPI3, (ws, ck, mck, sd), &clocks);

    blue.set_high();
    red.set_high();
    orange.set_high();
    green.set_high();
    writeln!(&mut usart, "Peripherals are initialized").unwrap();
    blue.set_low();
    red.set_low();
    orange.set_low();
    green.set_low();
    
    let mut amp = CS43L22::new(amp_reset, i2c1, 0x4A, i2s3);
    amp.initialize();
    
    let vol = amp.get_volume();
    writeln!(&mut usart, "Initial volume = {vol}").unwrap();
    amp.change_volume(10);
    let vol = amp.get_volume();
    writeln!(&mut usart, "Volume after +10 = {vol}").unwrap();
    amp.change_volume(30);
    let vol = amp.get_volume();
    writeln!(&mut usart, "Volume after +30 = {vol}").unwrap();
    amp.change_volume(-20);
    let vol = amp.get_volume();
    writeln!(&mut usart, "Volume after -29 = {vol}").unwrap();
    amp.change_volume(-3);
    let vol = amp.get_volume();
    writeln!(&mut usart, "Volume after -3 = {vol}").unwrap();

    writeln!(&mut usart, "Entering mainloop").unwrap();
    loop {        
        blue.toggle();
    };
}
