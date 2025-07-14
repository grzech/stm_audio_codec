#![no_std]
#![no_main]

mod can_task;

use core::sync::atomic::Ordering;
use embassy_executor::Spawner;
use embassy_stm32::{
    can,
    gpio::{AnyPin, Level, Output, Pin, Speed},
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
        Timer::after(Duration::from_millis(500)).await;
        led.toggle();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut can_bus = can::Can::new(p.CAN1, p.PD0, p.PD1, CanIrqs);

    spawner.spawn(led_task(p.PD13.degrade())).unwrap();
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
