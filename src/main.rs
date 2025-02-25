#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4xx_hal::{
    pac::{self},
    prelude::*,
    i2s::{self, stm32_i2s_v12x::transfer::*},
    rcc,
    pac::RCC
};

use cs43l22::{Config, CS43L22};

const SAMPLE_RATE: u32 = 44_100;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let clocks = configure_clocks(dp.RCC);

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    let gpiod = dp.GPIOD.split();

    let mut blue = gpiod.pd15.into_push_pull_output();
    let mut red = gpiod.pd14.into_push_pull_output();
    let mut orange = gpiod.pd13.into_push_pull_output();
    let mut green = gpiod.pd12.into_push_pull_output();

    let mut audio_reset = gpiod.pd4.into_push_pull_output();

    audio_reset.set_high();

    let pins = (gpiob.pb6, gpiob.pb9);
    let i2c = dp.I2C1.i2c(pins, 100.kHz(), &clocks);
    let mut codec = CS43L22::new(i2c, 0x4A, Config::new().volume(100).verify_write(true)).unwrap();
    let pins = (gpioa.pa4, gpioc.pc10, gpioc.pc7, gpioc.pc12);
    let i2s = i2s::I2s::new(dp.SPI3, pins, &clocks);
    let i2s_config = I2sTransferConfig::new_master()
        .transmit()
        .master_clock(true)
        .standard(Philips)
        .data_format(Data32Channel32)
        .request_frequency(SAMPLE_RATE);
    let mut sound_out = I2sTransfer::new(i2s, i2s_config);

    let pins = (gpiob.pb12, gpiob.pb10, gpioc.pc6, gpioc.pc3);
    let i2s = i2s::I2s::new(dp.SPI2, pins, &clocks);
    let i2s_config = I2sTransferConfig::new_master()
        .receive()
        .master_clock(true)
        .standard(Philips)
        .data_format(Data32Channel32)
        .request_frequency(SAMPLE_RATE);
    let mut sound_in = I2sTransfer::new(i2s, i2s_config);
 
    codec.play().unwrap();

    loop {
        green.toggle();
        match sound_in.read::<(i32, i32)>() {
            Ok(_) => red.toggle(),
            Err(_) => orange.toggle(),
        };
        match sound_out.write((0xAAAA, 0x5555)) {
            Ok(_) => blue.toggle(),
            Err(_) => red.toggle(),
        };
    }
}

fn configure_clocks(clocks: RCC) -> rcc::Clocks {
    clocks.apb1enr.write(|w| w.usart2en().enabled().can1en().enabled());
    clocks.cr.write(|w| w.plli2son().on());
    clocks.ahb1enr.write(|w| w.gpioaen().enabled().gpioden().enabled());
    return clocks.constrain().cfgr.use_hse(8.MHz())
                                  .sysclk(168.MHz())
                                  .i2s_clk(96.MHz())
                                  .freeze();
}
