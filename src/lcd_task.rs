use embassy_executor;
use crate::{Ili9163, IntoBytes, StmSpi};

use embassy_stm32::gpio::Output;
use embassy_time::{Delay, Duration, Timer};
use ili9163_driver::RGB;
use core::sync::atomic::AtomicU32;

pub static LCD_DATA: AtomicU32 = AtomicU32::new(0x39393939);

#[embassy_executor::task]
pub async fn lcd_task(lcd: Option<Ili9163<StmSpi, Output<'static>, Delay, u16>>) {
    if let Some(mut ili) = lcd {
        let _ = ili.turn_backlight(true);
        let _ = ili.print_text("Hello World", (10, 10), (RGB(0xFF, 0x00, 0xFF), RGB(0, 0, 0)));
        let _ = ili.print_text("From Embassy", (10, 20), (RGB(0xFF, 0x00, 0xAA), RGB(0xBB, 0xAA, 0xFF)));
        let _ = ili.print_text("on STM32F407VG", (10, 30), (RGB(0xAA, 0x00, 0xFF), RGB(0xFF, 0xAA, 0xBB)));
        
        loop {
            Timer::after(Duration::from_millis(1000)).await;
            let _ = ili.print_text("###", (50, 50), (RGB(0xFF, 0x00, 0x00), RGB(0x00, 0x00, 0xFF)));
            let _ = ili.print_text("###", (50, 61), (RGB(0xFF, 0x00, 0x00), RGB(0x00, 0x00, 0xFF)));
            let _ = ili.print_text("###", (50,72), (RGB(0xFF, 0x00, 0x00), RGB(0x00, 0x00, 0xFF)));
            Timer::after(Duration::from_millis(1000)).await;
            let _ = ili.print_text("###", (50, 50), (RGB(0x00, 0x00, 0xFF), RGB(0xFF, 0x00, 0x00)));
            let _ = ili.print_text("###", (50, 61), (RGB(0x00, 0x00, 0xFF), RGB(0xFF, 0x00, 0x00)));
            let _ = ili.print_text("###", (50, 72), (RGB(0x00, 0x00, 0xFF), RGB(0xFF, 0x00, 0x00)));
            let _ = ili.print_text(str::from_utf8(&LCD_DATA.to_bytes()).unwrap(), (50, 90), (RGB(0xFF, 0xFF, 0xFF), RGB(0x00, 0x00, 0x00)));
        }
    }
}