#![no_std]
#![no_main]

use core::panic::PanicInfo;

use rp2040_hal as hal;

// Custom Panic handler

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Copy bootloader from rp2040-boot2 into BOOT2 section of memory
#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

// External high-speed crystal on the Raspberry Pi Pico board is 12 MHz.
const XTAL_FREQ_HZ: u32 = 12_000_000;

#[hal::entry]
fn main() -> ! {
    // Get ownership of hardware peripherals
    let mut pac = hal::pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure GPIO15 as an output
    let mut led_pin = pins.gpio15.into_push_pull_output();

    loop {
        led_pin.set_high().unwrap();
        timer.delay_ms(500);
        led_pin.set_low();
        timer.delay_ms(500);
    }
}
