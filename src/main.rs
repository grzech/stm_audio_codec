#![no_std]
#![no_main]

use core::fmt::Write;

// Imports
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4xx_hal::{
    gpio::Pin,
    pac::{self},
    prelude::*,
    serial::Config,
};

#[entry]
fn main() -> ! {
    // Setup handler for device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Configure the LED pin as a push pull ouput and obtain handler.
    // On the Nucleo FR401 theres an on-board LED connected to pin PA5.
    let gpiod = dp.GPIOD.split();
    let mut led = gpiod.pd15.into_push_pull_output();
    let mut led2 = gpiod.pd14.into_push_pull_output();
    let clocks = dp.RCC.constrain().cfgr.use_hse(8.MHz()).freeze();

    // Configure the button pin (if needed) and obtain handler.
    // On the Nucleo FR401 there is a button connected to pin PC13.
    // Pin is input by default
    let gpioa = dp.GPIOA.split();
    let button = gpioa.pa0;

    let usart_tx = gpioa.pa2.into_alternate::<7>();
    let usart_rx = gpioa.pa3.into_alternate::<7>();
    let mut usart = dp.USART2.serial((usart_tx, usart_rx),
                                                     Config::default()
                                                        .baudrate(9600.bps())
                                                        .parity_none()
                                                        .wordlength_8(),
                                                     &clocks)
                                        .unwrap();
    
    // Create and initialize a delay variable to manage delay loop
    let mut del_var = 10_0000_u32;

    // Initialize LED to on or off
    led.set_low();
    led2.set_high();
    // Application Loop
    loop {
        // Call delay function and update delay variable once done
        del_var = loop_delay(del_var, &button);

        // Toggle LED
        led.toggle();
        led2.toggle();
        writeln!(&mut usart, "Some text to send").unwrap();

    }
}

// Delay Function
fn loop_delay<const P: char, const N: u8>(mut del: u32, but: &Pin<P, N>) -> u32 {
    // Loop for until value of del
    for _i in 1..del {
        // Check if button got pressed
        if but.is_low() {
            // If button pressed decrease the delay value
            del = del - 2_5000_u32;
            // If updated delay value reaches zero then reset it back to starting value
            if del < 2_5000 {
                del = 10_0000_u32;
            }
            // Exit function returning updated delay value if button pressed
            return del;
        }
    }
    // Exit function returning original delay value if button no pressed (for loop ending naturally)
    del
}

