#![no_std]
#![no_main]

use core::fmt::Write;

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4xx_hal::{
    pac::{self},
    prelude::*,
    i2s::{self, stm32_i2s_v12x::transfer::*},
};

use cs43l22::{Config, CS43L22};

const SAMPLE_RATE: u32 = 96_000;

const SINE_750: [i16; 64] = [
    0, 3211, 6392, 9511, 12539, 15446, 18204, 20787, 23169, 25329, 27244, 28897, 30272, 31356,
    32137, 32609, 32767, 32609, 32137, 31356, 30272, 28897, 27244, 25329, 23169, 20787, 18204,
    15446, 12539, 9511, 6392, 3211, 0, -3211, -6392, -9511, -12539, -15446, -18204, -20787, -23169,
    -25329, -27244, -28897, -30272, -31356, -32137, -32609, -32767, -32609, -32137, -31356, -30272,
    -28897, -27244, -25329, -23169, -20787, -18204, -15446, -12539, -9511, -6392, -3211,
];

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(168.MHz())
        .i2s_clk(96.MHz())
        .freeze();
    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    let gpiod = dp.GPIOD.split();
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
    let mut i2s_transfer = I2sTransfer::new(i2s, i2s_config);


    codec.play().unwrap();

    let sine_750_1sec = SINE_750
        .iter()
        .map(|&x| {
            let x = (x as i32) << 16;
            (x, x)
        })
        .cycle()
        .take(SAMPLE_RATE as usize);

    loop {
        i2s_transfer.write_iter(sine_750_1sec.clone());
    }
}
