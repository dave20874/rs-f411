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
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_stm32::peripherals::PC13;

static PERIOD_MS: Mutex<ThreadModeRawMutex, u64> = Mutex::new(200);
static TICKLE: Signal<CriticalSectionRawMutex, ()> = Signal::new();

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
async fn talky() {
    let mut fast: bool = false;

    loop {
        TICKLE.wait().await;
        // Timer::after(Duration::from_millis(1000)).await;

        info!("That tickles!");

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
    }
}

// I don't like that PC13 had to be hard coded here.  Is there a more generic type that could be used?
#[embassy_executor::task]
async fn button_mon(mut button: ExtiInput<'static, PC13>) {

    loop {
        let x = button.wait_for_falling_edge();
        x.await;

        info!("The button!");
        TICKLE.signal(());  // send the tickle signal
         
        Timer::after(Duration::from_millis(10)).await;  // debounce
    }
}


#[embassy_executor::main]
async fn main(spawner: Spawner) {

    let p = embassy_stm32::init(Default::default());
    
    let button = Input::new(p.PC13, Pull::Up);
    let ibutton = ExtiInput::new(button, p.EXTI13);

    info!("Hello World!");

    spawner.spawn(talky()).unwrap();
    spawner.spawn(blinky(p.PA5.into())).unwrap();
    spawner.spawn(button_mon(ibutton)).unwrap();

    loop {
        Timer::after(Duration::from_millis(1000)).await;  // debounce
    }
}
