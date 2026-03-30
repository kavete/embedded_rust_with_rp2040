#![no_std]
#![no_main]

use core::panic::PanicInfo;

use rp2040_hal as hal;

use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

// Custom Panic handler

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

    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("Fake company")
            .product("Serial port")
            .serial_number("TEST")])
        .unwrap()
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build();

    // Read buffer
    let mut rx_buf = [0u8; 64];
    let mut timestamp = timer.get_counter();
    loop {
        // Needs to be called at least every 10 ms
        if usb_dev.poll(&mut [&mut serial]) {
            match serial.read(&mut rx_buf) {
                Ok(0) => {}
                Ok(count) => {
                    // Challenge for student!
                    rx_buf[..count].iter_mut().for_each(|byte| {
                        *byte = byte.to_ascii_uppercase();
                    });
                    let _ = serial.write(&rx_buf[0..count]);
                }
                Err(_e) => {}
            }
        }

        // Send message every second (non-blocking)
        if (timer.get_counter() - timestamp).to_millis() >= 1_000 {
            timestamp = timer.get_counter();
            let _ = serial.write(b"hello!\r\n");
        }
    }
}
