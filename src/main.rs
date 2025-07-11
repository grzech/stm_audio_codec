#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use embassy_executor::Spawner;
use embassy_stm32::gpio::{AnyPin, Level, Output, Pin, Speed};
use embassy_time::{Duration, Timer};
use panic_halt as _;
use defmt_rtt as _;

static BLINK_MS: AtomicU32 = AtomicU32::new(0);

#[embassy_executor::task]
async fn led_task(led: AnyPin) {
    // Configure the LED pin as a push pull ouput and obtain handler.
    // On the Nucleo FR401 theres an on-board LED connected to pin PA5.
    let mut led = Output::new(led, Level::Low, Speed::Low);

    loop {
        let del = BLINK_MS.load(Ordering::Relaxed);
        Timer::after(Duration::from_millis(del.into())).await;
        led.toggle();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize and create handle for devicer peripherals
    let p = embassy_stm32::init(Default::default());

    
    // Create and initialize a delay variable to manage delay loop
    let del_var = 2000;

    // Publish blink duration value to global context
    BLINK_MS.store(del_var, Ordering::Relaxed);

    // Spawn LED blinking task
    spawner.spawn(led_task(p.PD13.degrade())).unwrap();

    loop {
        Timer::after(Duration::from_millis(1000)).await;
    }
}
