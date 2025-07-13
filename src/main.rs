#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    can::{self, frame::Header, Frame, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, StandardId, TxInterruptHandler},
    gpio::{AnyPin, Level, Output, Pin, Speed},
    peripherals,
};
use embassy_time::{Duration, Timer};
use panic_halt as _;
use defmt_rtt as _;

static BLINK_MS: AtomicU32 = AtomicU32::new(0);

bind_interrupts!(struct Irqs {
    CAN1_SCE => SceInterruptHandler<peripherals::CAN1>;
    CAN1_TX => TxInterruptHandler<peripherals::CAN1>;
    CAN1_RX0 => Rx0InterruptHandler<peripherals::CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<peripherals::CAN1>;
});

#[embassy_executor::task]
async fn led_task(led: AnyPin) {
    let mut led = Output::new(led, Level::Low, Speed::Low);

    loop {
        let del = BLINK_MS.load(Ordering::Relaxed);
        Timer::after(Duration::from_millis(del.into())).await;
        led.toggle();
    }
}

#[embassy_executor::task]
async fn can_task(mut can: can::Can<'static>) {
    loop {
        can.write(&Frame::new(Header::new(can::Id::Standard(StandardId::new(0x555).unwrap()), 8, false), &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap()).await;
        Timer::after(Duration::from_millis(1000)).await;
    }
    
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let del_var = 2000;
    BLINK_MS.store(del_var, Ordering::Relaxed);
    spawner.spawn(led_task(p.PD13.degrade())).unwrap();
    let mut can_bus = can::Can::new(p.CAN1, p.PD0, p.PD1, Irqs);
    can_bus.set_bitrate(500_000);
    can_bus.enable().await;
    spawner.spawn(can_task(can_bus)).unwrap();
    loop {
        Timer::after(Duration::from_millis(1000)).await;
    }
}
