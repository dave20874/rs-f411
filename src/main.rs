#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_stm32::gpio::{Level, Output, Speed, AnyPin, Input, Pull};
use embassy_stm32::exti::ExtiInput;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};
use heapless::Arc;
use event_listener::Event;


static PERIOD_MS: Mutex<ThreadModeRawMutex, u64> = Mutex::new(200);

#[embassy_executor::task]
async fn blinky(pin: AnyPin) {
    // set pin for output
    let mut led = Output::new(pin, Level::High, Speed::Low);

    loop {
        let delay_ms = {
            let shared = PERIOD_MS.lock().await;
            *shared
        };
        
        // info!("high");
        // info!("LED Delay: {}", delay_ms);
        led.set_high();
        Timer::after(Duration::from_millis(delay_ms)).await;

        // info!("low");
        led.set_low();
        Timer::after(Duration::from_millis(delay_ms)).await;
    }
}

#[embassy_executor::task]
async fn talky(event: Event) {
    let listener = event.listen();

    loop {
        listener.await;

        info!("That tickles!");
    }
}



#[embassy_executor::main]
async fn main(spawner: Spawner) {

    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut fast: bool = false;
    let event = Event::new();

    spawner.spawn(talky(event));
    spawner.spawn(blinky(p.PA5.into())).unwrap();

    let button = Input::new(p.PC13, Pull::Up);
    let mut ibutton = ExtiInput::new(button, p.EXTI13);
    loop {
        // Asynchronously wait for GPIO events, allowing other tasks
        // to run, or the core to sleep.
        ibutton.wait_for_falling_edge().await;

        // signal another task
        event.notify();

        info!("Button pressed!");
        fast = !fast;
        {
            let mut period_ms = PERIOD_MS.lock().await;

            match fast {
                true => {
                    *period_ms = 200;
                }
                false => {
                    *period_ms = 500;
                }
            }
            info!("Delay: {}", *period_ms);
        }
        
        Timer::after(Duration::from_millis(10)).await;  // debounce
    }
}
