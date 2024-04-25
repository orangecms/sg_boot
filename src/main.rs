use clap::{Parser, Subcommand, ValueEnum};
use log::info;
use std::time::Duration;
use std::{thread, time};

use crate::protocol::Param1;

mod protocol;

// two minutes should be plenty
const POLL_TIMEOUT: Duration = Duration::from_secs(120);
const POLL_PERIOD: Duration = Duration::from_millis(50);
const TEN_SECS: Duration = Duration::from_secs(10);
const HALF_SEC: Duration = Duration::from_millis(500);

const USB_VID_CVITEK: u16 = 0x3346;
const USB_PID_USB_COM: u16 = 0x1000;

const SRAM_BASE: usize = 0x0000_0000;

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

fn print_port_info(info: &serialport::UsbPortInfo) {
    let mf = info.manufacturer.as_ref().map_or("", String::as_str);
    let pi = info.product.as_ref().map_or("", String::as_str);
    let sn = info.serial_number.as_ref().map_or("", String::as_str);
    info!("{mf} {pi} {:04x}:{:04x} ({sn})", info.vid, info.pid);
}

fn poll_dev() -> String {
    let now = time::Instant::now();

    while time::Instant::now() <= now + POLL_TIMEOUT {
        match serialport::available_ports() {
            Ok(ports) => {
                for p in ports {
                    let name = p.port_name;
                    if let serialport::SerialPortType::UsbPort(info) = p.port_type {
                        if info.vid == USB_VID_CVITEK && info.pid == USB_PID_USB_COM {
                            print_port_info(&info);
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

fn connect() -> std::boxed::Box<dyn serialport::SerialPort> {
    let dev = poll_dev();
    match serialport::new(dev.clone(), 115_200)
        .timeout(TEN_SECS)
        .open()
    {
        Ok(d) => d,
        Err(_) => panic!("Failed to open serial port {dev}"),
    }
}

const CRC: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);
fn main() {
    let payload = include_bytes!("../oreboot_x.bin");

    let checksum = CRC.checksum(payload);
    println!("Payload checksum: {checksum:04x}");
    let checksum = checksum.to_le_bytes();

    let param1 = Param1 {
        bl2_img_size: (payload.len() as u32).to_le_bytes(),
        bl2_img_cksum: [checksum[0], checksum[1], 0xfe, 0xca],
        ..Default::default()
    };

    let checksum = param1.checksum();
    println!("Header checksum: {checksum:04x}");
    let checksum = checksum.to_le_bytes();

    let header = crate::protocol::CVITekHeader {
        param1_checksum: [checksum[0], checksum[1], 0xfe, 0xca],
        param1,
        ..Default::default()
    };

    let mut s = header.to_slice().to_vec();
    s.truncate(0x100);
    println!("{:02x?}", s);
    // println!("{header:#02x?}");
    // println!("{:02x?}", header.to_slice());

    let cmd = Cli::parse().cmd;
    env_logger::init();

    println!("Waiting for CVITek USB devices...");
    let mut port = connect();
    crate::protocol::send_magic(&mut port);
    std::thread::sleep(Duration::from_millis(500));

    println!("send HEADER...");
    let mut port = connect();

    // let header = include_bytes!("../header.bin");
    crate::protocol::send_file(&mut port, header.to_slice());
    crate::protocol::send_flag_and_break(&mut port);
    std::thread::sleep(HALF_SEC);

    println!("Waiting for CVITek USB devices...");
    let mut port = connect();
    crate::protocol::send_magic(&mut port);

    println!("send PAYLOAD...");
    crate::protocol::send_file(&mut port, payload);
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
