use embassy_executor;
use embassy_stm32::{
    bind_interrupts,
    can::{self, frame::Header, Frame, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, StandardId, TxInterruptHandler},
    peripherals,
};
use embassy_time::{Duration, Timer};
use core::sync::atomic::{AtomicU32, Ordering};

pub static STATUS1: AtomicU32 = AtomicU32::new(0);
pub static STATUS2: AtomicU32 = AtomicU32::new(0);

bind_interrupts!(pub struct Irqs {
    CAN1_SCE => SceInterruptHandler<peripherals::CAN1>;
    CAN1_TX => TxInterruptHandler<peripherals::CAN1>;
    CAN1_RX0 => Rx0InterruptHandler<peripherals::CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<peripherals::CAN1>;
});

trait IntoBytes {
    fn to_bytes(&self) -> [u8; 4];
}

impl IntoBytes for AtomicU32 {
    fn to_bytes(&self) -> [u8; 4] {
        let val = self.load(Ordering::Relaxed);
        [((val >> 24) & 0xFF) as u8, ((val >> 16) & 0xFF) as u8, ((val >> 8) & 0xFF) as u8, (val & 0xFF) as u8]
    }
}

#[embassy_executor::task]
pub async fn can_task(mut can: can::Can<'static>) {
    let header = Header::new(can::Id::Standard(StandardId::new(0x555).unwrap()), 8, false);
    loop {
        let mut data : [u8; 8] = [0; 8];
        for (i, &d) in STATUS1.to_bytes().iter().chain(&STATUS2.to_bytes()).enumerate() {
            data[i] = d;
        }
        can.write(&Frame::new(header, &data).unwrap()).await;

        Timer::after(Duration::from_millis(1000)).await;
    }
    
}