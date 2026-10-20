mod physical_device;
mod virtual_device;

use signal_hook::consts::signal::*;
use signal_hook::flag::register;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use physical_device::PhysicalDevice;
use rusb::Error as RusbError;
use virtual_device::{DeviceDispatcher, RawDataReader};

const VID: u16 = 0x08f2;
const PID: u16 = 0x6811;

fn main() {
    println!("moons-ink driver daemon started.");

    let shutdown_signal = Arc::new(AtomicBool::new(false));
    for signal in [SIGINT, SIGTERM, SIGQUIT] {
        register(signal, Arc::clone(&shutdown_signal))
            .expect("Error registering interrupt signal handlers.");
    }

    let mut device_dispatcher = match DeviceDispatcher::try_new() {
        Ok(dispatcher) => dispatcher,
        Err(err) => {
            eprintln!("Error initializing virtual uinput devices: {err}");
            eprintln!("Please verify permissions on /dev/uinput or install udev rules.");
            std::process::exit(1);
        }
    };

    let mut data_reader = RawDataReader::new();
    let mut waiting_printed = false;

    while !shutdown_signal.load(Ordering::Relaxed) {
        match PhysicalDevice::open(VID, PID) {
            Ok(physical_device) => {
                println!("10moons tablet connected (08f2:6811). Driver active.");
                waiting_printed = false;
                device_dispatcher.reset_state();

                // Active loop: reads device reports until unplugged or shutdown signal
                while !shutdown_signal.load(Ordering::Relaxed) {
                    match physical_device.read_device_responses(&mut data_reader.data) {
                        Ok(_) => {
                            device_dispatcher.dispatch(&data_reader);
                            if let Err(e) = device_dispatcher.syn() {
                                eprintln!("Warning: error emitting SYN report: {e}");
                            }
                        }
                        Err(RusbError::Timeout) => {
                            // Normal timeout when pen is idle; continue listening
                            continue;
                        }
                        Err(err) => {
                            eprintln!("Tablet disconnected or read error: {err}");
                            device_dispatcher.reset_state();
                            break;
                        }
                    }
                }
            }
            Err(RusbError::NoDevice) | Err(RusbError::NotFound) => {
                if !waiting_printed {
                    println!("Waiting for 10moons tablet (08f2:6811) to be connected...");
                    waiting_printed = true;
                }
                sleep(Duration::from_millis(1000));
            }
            Err(RusbError::Access) => {
                eprintln!("Permission denied accessing USB device (08f2:6811).");
                eprintln!("Ensure udev rules are installed in /etc/udev/rules.d/99-moons-ink.rules");
                sleep(Duration::from_millis(2000));
            }
            Err(err) => {
                eprintln!("Error connecting to tablet: {err}");
                sleep(Duration::from_millis(1500));
            }
        }
    }

    device_dispatcher.reset_state();
    println!("moons-ink driver daemon has exited cleanly.");
}
