#![no_std]
#![no_main]

mod can_task;
mod lcd_task;

use lcd_task::lcd_task;
use embedded_hal;
use core::sync::atomic::{AtomicU32, Ordering};
use embassy_executor::Spawner;
use embassy_stm32::{
    can::{self, filter, Fifo}, gpio::{AnyPin, Level, Output, Pin, Speed}, mode::Blocking, rcc, spi, time::Hertz
};
use embassy_time::{Duration, Timer, Delay};
use panic_halt as _;
use defmt_rtt as _;

use can_task::{can_task, STATUS1, STATUS2};
use can_task::Irqs as CanIrqs;
use ili9163_driver::Ili9163;
use lcd_task::LCD_DATA;

struct StmSpi (spi::Spi<'static, Blocking>, Output<'static>);

impl embedded_hal::blocking::spi::Write<u8> for StmSpi {
    type Error = u8;
    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.1.set_low();
        self.0.write(words).unwrap();
        self.1.set_high();
        Ok(())
    }
}

#[embassy_executor::task]
async fn led_task(led: AnyPin) {
    let mut led = Output::new(led, Level::Low, Speed::Low);

    loop {
        Timer::after(Duration::from_millis(700)).await;
        led.set_high();
        Timer::after(Duration::from_millis(100)).await;
        led.set_low();
        Timer::after(Duration::from_millis(100)).await;
        led.set_high();
        Timer::after(Duration::from_millis(100)).await;
        led.set_low();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let pll = rcc::Pll {
        prediv: rcc::PllPreDiv::DIV8,
        mul: rcc::PllMul::MUL336,
        divp: Some(rcc::PllPDiv::DIV2),
        divq: None,
        divr: None,
    };
    let i2s_pll = rcc::Pll {
        prediv: rcc::PllPreDiv::DIV8,
        mul: rcc::PllMul::MUL258,
        divp: None,
        divq: None,
        divr: Some(rcc::PllRDiv::DIV3),
    };
    let mut config = rcc::Config::default();
    config.hsi = false;
    config.hse = Some(rcc::Hse{freq: Hertz(8_000), mode: rcc::HseMode::Oscillator });
    config.sys = rcc::Sysclk::HSE;
    config.pll_src = rcc::PllSource::HSE;
    config.pll = Some(pll); 
    config.plli2s = Some(i2s_pll);

    let lcd_comm: spi::Spi<'static, Blocking> = spi::Spi::new_blocking(
        p.SPI1,
        p.PB3,
        p.PB5,
        p.PA6,
        spi::Config::default());
    let ili= Ili9163::<StmSpi, Output<'_>, Delay, ()>::new(
        StmSpi(lcd_comm, Output::new(p.PB6, Level::High, Speed::VeryHigh)),
        Output::new(p.PB4, Level::Low, Speed::VeryHigh),
        Delay{},
        Output::new(p.PB7, Level::Low, Speed::VeryHigh),
        Output::new(p.PB8, Level::Low, Speed::VeryHigh),
    ).initialize_for_16bit_pixel().ok();
    
    let mut can_bus = can::Can::new(p.CAN1, p.PD0, p.PD1, CanIrqs);
    can_bus.modify_filters().enable_bank(0, Fifo::Fifo0, filter::Mask32::accept_all());
    spawner.spawn(led_task(p.PD15.degrade())).unwrap();
    can_bus.set_bitrate(500_000);
    can_bus.enable().await;
    spawner.spawn(can_task(can_bus)).unwrap();
    spawner.spawn(lcd_task(ili)).unwrap();
    STATUS1.store(0, Ordering::Relaxed);
    STATUS2.store(0xFFFFFFFF, Ordering::Relaxed);
    loop {
        Timer::after(Duration::from_millis(1000)).await;
        STATUS1.fetch_add(1, Ordering::Relaxed);
        STATUS2.fetch_sub(1, Ordering::Relaxed);
    }
}

trait IntoBytes {
    fn to_bytes(&self) -> [u8; 4];
}

impl IntoBytes for AtomicU32 {
    fn to_bytes(&self) -> [u8; 4] {
        let val = self.load(Ordering::Relaxed);
        [((val >> 24) & 0xFF) as u8, ((val >> 16) & 0xFF) as u8, ((val >> 8) & 0xFF) as u8, (val & 0xFF) as u8]
    }
}
