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
    i2s::I2s,
    dma,
};

mod cs43l22;
use cs43l22::CS43L22;

use stm32_i2s_v12x::driver as i2s;

const SAMPLE_RATE: u32 = 48_000;
const ARRAY_SIZE: usize = 100;

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
    let i2s_pins = (
        gpioa.pa4.into_alternate(),
        gpioc.pc10.into_alternate(),
        gpioc.pc7.into_alternate(),
        gpioc.pc12.into_alternate(),
    );
    let driver_config = i2s::I2sDriverConfig::new_master()
        .direction(i2s::Transmit)
        .standard(i2s::Philips)
        .data_format(i2s::DataFormat::Data24Channel32)
        .master_clock(true)
        .request_frequency(SAMPLE_RATE);
    let hal_i2s = I2s::new(dp.SPI3, i2s_pins, &clocks);
    let mut i2s3 = driver_config.i2s_driver(hal_i2s);
    i2s3.set_tx_dma(true);
    i2s3.enable();
    let dma1_streams = dma::StreamsTuple::new(dp.DMA1);
    let dma_config = dma::config::DmaConfig::default()
        .memory_increment(true)
        .transfer_complete_interrupt(true);
    
    let buffer = cortex_m::singleton!(: [u16; ARRAY_SIZE] = [0x55AA; ARRAY_SIZE]).unwrap();
    let mut dma_transfer = dma::Transfer::init_memory_to_peripheral(dma1_streams.5, i2s3, buffer, None, dma_config);

    dma_transfer.start(|i2s| i2s.enable());

    blue.set_high();
    red.set_high();
    orange.set_high();
    green.set_high();
    writeln!(&mut usart, "Peripherals are initialized").unwrap();
    blue.set_low();
    red.set_low();
    orange.set_low();
    green.set_low();
    
 /*   let mut amp = CS43L22::new(amp_reset, i2c1, 0x4A);
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
*/
    writeln!(&mut usart, "Entering mainloop").unwrap();
    loop {        
        blue.toggle();
    };
}

fn configure_clocks(clocks: RCC) -> Clocks {
    clocks.apb1enr.write(|w| w.spi3en().enabled().usart2en().enabled().i2c1en().enabled());
    clocks.apb2enr.write(|w| w.spi1en().enabled());
    clocks.cr.write(|w| w.plli2son().on());
    clocks.ahb1enr.write(|w| w.dma1en().enabled().dma2en().enabled().gpioaen().enabled()
        .gpioben().enabled().gpioden().enabled());
    return clocks.constrain().cfgr
        .use_hse(8.MHz())
        .sysclk(96.MHz())
        .i2s_clk(61440.kHz())
        .freeze();
}