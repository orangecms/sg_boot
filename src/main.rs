use clap::{Parser, Subcommand, ValueEnum};
use log::info;
use std::io::Write;
use std::time::Duration;
use std::{thread, time};

const POLL_PERIOD: time::Duration = time::Duration::from_millis(50);
// two minutes should be plenty
const TIMEOUT: time::Duration = time::Duration::from_secs(120);

mod protocol;

// https://docs.rs/crc/latest/crc/constant.CRC_16_KERMIT.html

const HEADER: &[u8] = include_bytes!("../header.bin");
const OREBOOT: &[u8] = include_bytes!("../oreboot_x.bin");

const USB_VID_CVITEK: u16 = 0x3346;
const USB_PID_USB_COM: u16 = 0x1000;

#[allow(non_camel_case_types)]
#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum Board {
    MilkV_DuoS,
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    Info,
    /// Write file to SRAM and execute (S905D3 only for now, needs header)
    #[clap(verbatim_doc_comment)]
    Run {
        file_name: String,
    },
}

/// Sopho/CVITek mask ROM loader tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Command to run
    #[command(subcommand)]
    cmd: Command,
}

fn poll_dev() -> String {
    let now = time::Instant::now();

    while time::Instant::now() <= now + TIMEOUT {
        match serialport::available_ports() {
            Ok(ports) => {
                for p in ports {
                    let name = p.port_name;
                    if let serialport::SerialPortType::UsbPort(ref info) = p.port_type {
                        if info.vid == USB_VID_CVITEK && info.pid == USB_PID_USB_COM {
                            let sn = info.serial_number.as_ref().map_or("", String::as_str);
                            let mf = info.manufacturer.as_ref().map_or("", String::as_str);
                            let pi = info.product.as_ref().map_or("", String::as_str);
                            info!("{mf} {pi} {:04x}:{:04x} ({sn})", info.vid, info.pid);
                            return name;
                        }
                    }
                    thread::sleep(POLL_PERIOD);
                }
            }
            Err(_e) => {
                thread::sleep(POLL_PERIOD);
            }
        }
    }
    panic!("timeout waiting for CVITek USB device");
}

const SRAM_BASE: usize = 0x0000_0000;

const PORT_TIMEOUT: Duration = Duration::from_secs(10);

fn main() {
    let cmd = Cli::parse().cmd;
    env_logger::init();

    println!("Waiting for CVITek USB devices...");
    let dev = poll_dev();
    let mut port = match serialport::new(dev.clone(), 115_200)
        .timeout(PORT_TIMEOUT)
        .open()
    {
        Ok(d) => d,
        Err(_) => panic!("Failed to open serial port {dev}"),
    };
    crate::protocol::send_magic(&mut port);
    std::thread::sleep(Duration::from_millis(500));

    let dev = poll_dev();
    let mut port = match serialport::new(dev.clone(), 115_200)
        .timeout(PORT_TIMEOUT)
        .open()
    {
        Ok(d) => d,
        Err(_) => panic!("Failed to open serial port {dev}"),
    };

    println!("send HEADER...");
    crate::protocol::send_file(&mut port, HEADER);
    crate::protocol::send_flag_and_break(&mut port);
    std::thread::sleep(Duration::from_millis(500));

    println!("Waiting for CVITek USB devices...");
    let dev = poll_dev();
    let mut port = match serialport::new(dev.clone(), 115_200)
        .timeout(PORT_TIMEOUT)
        .open()
    {
        Ok(d) => d,
        Err(_) => panic!("Failed to open serial port {dev}"),
    };
    crate::protocol::send_magic(&mut port);

    println!("send PAYLOAD...");
    crate::protocol::send_file(&mut port, OREBOOT);
    crate::protocol::send_flag_and_break(&mut port);

    match cmd {
        Command::Run { file_name } => {
            let file = std::fs::read(file_name).unwrap();
            let addr = SRAM_BASE;
            // protocol::write(&port, timeout, &file, addr);
            // protocol::exec(&port, timeout, addr).unwrap();
        }
        Command::Info => {
            println!();
        }
    }
}
