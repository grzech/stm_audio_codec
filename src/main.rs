#![no_std]
#![no_main]

mod can_task;

use core::{sync::atomic::Ordering};
use embassy_executor::Spawner;
use embassy_stm32::{
    can,
    gpio::{AnyPin, Level, Output, Pin, Speed},
    rcc, time::Hertz,
};
use embassy_time::{Duration, Timer};
use panic_halt as _;
use defmt_rtt as _;

use can_task::{can_task, STATUS1, STATUS2};
use can_task::Irqs as CanIrqs;

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

    let mut can_bus = can::Can::new(p.CAN1, p.PD0, p.PD1, CanIrqs);
    spawner.spawn(led_task(p.PD15.degrade())).unwrap();
    can_bus.set_bitrate(500_000);
    can_bus.enable().await;
    spawner.spawn(can_task(can_bus)).unwrap();
    STATUS1.store(0, Ordering::Relaxed);
    STATUS2.store(0xFFFFFFFF, Ordering::Relaxed);
    loop {
        Timer::after(Duration::from_millis(1000)).await;
        STATUS1.fetch_add(1, Ordering::Relaxed);
        STATUS2.fetch_sub(1, Ordering::Relaxed);
    }
}
