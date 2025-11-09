use embassy_executor;
use embassy_stm32::{
    bind_interrupts,
    can::{self, Frame, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, StandardId, TxInterruptHandler, frame::Header},
    peripherals,
};
use embassy_time::{Duration, Timer};
use core::sync::atomic::{AtomicU32, Ordering};
use crate::{LCD_DATA, IntoBytes};

pub static STATUS1: AtomicU32 = AtomicU32::new(0);
pub static STATUS2: AtomicU32 = AtomicU32::new(0);

bind_interrupts!(pub struct Irqs {
    CAN1_SCE => SceInterruptHandler<peripherals::CAN1>;
    CAN1_TX => TxInterruptHandler<peripherals::CAN1>;
    CAN1_RX0 => Rx0InterruptHandler<peripherals::CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<peripherals::CAN1>;
});

#[embassy_executor::task]
pub async fn can_task(mut can: can::Can<'static>) {
    let header = Header::new(can::Id::Standard(StandardId::new(0x555).unwrap()), 8, false);
    let status_msg = Header::new(can::Id::Standard(StandardId::new(0x556).unwrap()), 8, false);
    loop {
        if let Ok(env) = can.try_read() {
            let received = env.frame.data();
            let val = (received[3] as u32) | ((received[2] as u32) << 8) | ((received[1] as u32) << 16) | ((received[0] as u32) << 24);
            LCD_DATA.store(val, Ordering::Relaxed);
            let mut data : [u8; 8] = [0, 0, 0, 0, received[0], received[1], received[2], received[3]];
            for (i, &d) in STATUS1.to_bytes().iter().chain(&STATUS2.to_bytes()).enumerate() {
                data[i] = d;
            }
            can.write(&Frame::new(status_msg, &data).unwrap()).await;
        }
        let mut data : [u8; 8] = [0; 8];
        for (i, &d) in STATUS1.to_bytes().iter().chain(&STATUS2.to_bytes()).enumerate() {
            data[i] = d;
        }
        can.write(&Frame::new(header, &data).unwrap()).await;

        Timer::after(Duration::from_millis(1000)).await;
    }
    
}