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

const NO_MAGIC: &[u8] = include_bytes!("../nomagic.bin");
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

// CRC variant figured out by trying all the CRC16s offered by crc crate :D
const CRC: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);

// fixture obtained from dumping intermediate values in vendor tool
#[test]
fn test_crc() {
    // 0x42ca
    let data = vec![
        3, 0, 136, 0, 0, 0, 0, 255, 8, 0, 0, 20, 0, 192, 24, 213, 0, 6, 160, 210, 64, 16, 24, 213,
        64, 0, 0, 148, 64, 0, 0, 53, 0, 0, 0, 20, 160, 63, 0, 16, 8, 0, 0, 20, 0, 192, 24, 213, 0,
        6, 160, 210, 64, 16, 24, 213, 64, 0, 0, 148, 64, 0, 0, 53, 0, 0, 0, 20, 160, 63, 0, 16, 0,
        17, 62, 213, 0, 12, 64, 178, 0, 17, 30, 213, 95, 17, 30, 213, 0, 16, 62, 213, 161, 0, 128,
        210, 0, 0, 33, 138, 0, 16, 30, 213, 16, 0, 0, 20, 0, 192, 28, 213, 224, 127, 134, 210, 64,
        17, 28, 213, 0, 16, 60, 213, 161, 0, 128, 210, 0, 0, 33, 138, 0, 16, 28, 213,
    ];
    let sum = CRC.checksum(&data);
    assert!(sum == 0x42ca);
    let data = vec![
        0, 1, 0, 0, 0, 0, 23, 64, 54, 56, 54, 57, 55, 48, 55, 49, 55, 50, 55, 51, 55, 52, 55, 53,
        55, 54, 55, 55, 55, 56, 55, 57, 56, 48, 56, 49, 56, 50, 56, 51, 56, 52, 56, 53, 56, 54, 56,
        55, 56, 56, 56, 57, 57, 48, 57, 49, 57, 50, 57, 51, 57, 52, 57, 53, 57, 54, 57, 55, 57, 56,
        57, 57, 114, 97, 110, 103, 101, 32, 115, 116, 97, 114, 116, 32, 105, 110, 100, 101, 120,
        32, 32, 111, 117, 116, 32, 111, 102, 32, 114, 97, 110, 103, 101, 32, 102, 111, 114, 32,
        115, 108, 105, 99, 101, 32, 111, 102, 32, 108, 101, 110, 103, 116, 104, 32, 0, 0, 0, 0,
        128, 23, 0, 12, 0, 0, 0, 0, 18, 0, 0, 0, 0, 0, 0, 0, 146, 23, 0, 12, 0, 0, 0, 0, 34, 0, 0,
        0, 0, 0, 0, 0, 142, 16, 0, 12, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0,
        144, 16, 0, 12, 0, 0, 0, 0, 178, 16, 0, 12, 0, 0, 0, 0, 112, 17, 0, 12, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let sum = CRC.checksum(&data);
    assert!(sum == 0x6b31);
}

#[derive(Debug)]
struct Header {
    cmd: u8,
    size: u16,
    addr: u64,
}

impl Header {
    fn to_slice(&self) -> [u8; 8] {
        // add header's own size
        let sz = self.size + 8;
        let l1 = (sz >> 8) as u8;
        let l0 = sz as u8;
        let a4 = (self.addr >> 32) as u8;
        let a3 = (self.addr >> 24) as u8;
        let a2 = (self.addr >> 16) as u8;
        let a1 = (self.addr >> 8) as u8;
        let a0 = self.addr as u8;
        [self.cmd, l1, l0, a4, a3, a2, a1, a0]
    }
}

fn check_response(data: &[u8], resp: &[u8]) {
    info!("response: {resp:x?}");

    let crc_hi = resp[crate::protocol::RSP_CRC16_HI_OFFSET];
    let crc_lo = resp[crate::protocol::RSP_CRC16_LO_OFFSET];
    let rsp_checksum = ((crc_hi as u16) << 8) | crc_lo as u16;

    let exp_checksum = CRC.checksum(&data);

    if exp_checksum != rsp_checksum {
        panic!("Checksum mismatch: got {rsp_checksum:04x}, expected {exp_checksum:04x}");
    }

    info!("checksum {rsp_checksum:04x} == {exp_checksum:04x}");

    let rsp_token = resp[crate::protocol::RSP_TOKEN_OFFSET];
    info!("token: {rsp_token}");
}

const CHUNK_SIZE: usize = 256;
const FLAG_ADDR: u64 = 0x0E000004;
const FLAG: [u8; 4] = *b"1NGM";

const PORT_TIMEOUT: Duration = Duration::from_secs(10);

fn send(port: &mut std::boxed::Box<dyn serialport::SerialPort>, data: &[u8]) {
    let sent = port.write(data).expect("Write failed!");
    let mut resp: Vec<u8> = vec![0; 16];
    let read = port.read(resp.as_mut_slice()).expect("Found no data!");
    check_response(data, &resp);
    info!("sent {sent} bytes, read {read} bytes");
}

fn send_file(port: &mut std::boxed::Box<dyn serialport::SerialPort>, f: &[u8]) {
    for (i, chunk) in f.chunks(CHUNK_SIZE).enumerate() {
        let h = Header {
            cmd: protocol::CVI_USB_TX_DATA_TO_RAM,
            size: chunk.len() as u16,
            addr: (i * CHUNK_SIZE) as u64,
        };
        info!("{h:?}");
        let data = h
            .to_slice()
            .iter()
            .chain(chunk)
            .copied()
            .collect::<Vec<u8>>();
        info!("{data:x?}");
        send(port, &data);
    }
}

fn send_magic(port: &mut std::boxed::Box<dyn serialport::SerialPort>) {
    println!("\nsend NO MAGIC...\n");
    let h = Header {
        cmd: protocol::CV_USB_KEEP_DL,
        size: NO_MAGIC.len() as u16,
        addr: protocol::DUMMY_ADDR,
    };
    info!("{h:?}");
    let data = h
        .to_slice()
        .iter()
        .chain(NO_MAGIC)
        .copied()
        .collect::<Vec<u8>>();
    info!("{data:x?}");
    send(port, &data);
}

fn send_flag_and_break(port: &mut std::boxed::Box<dyn serialport::SerialPort>) {
    let h = Header {
        cmd: protocol::CVI_USB_TX_FLAG,
        size: FLAG.len() as u16,
        addr: FLAG_ADDR,
    };
    info!("{h:?}");
    let data = h
        .to_slice()
        .iter()
        .chain(&FLAG)
        .copied()
        .collect::<Vec<u8>>();
    info!("{data:x?}");
    send(port, &data);
    let h = Header {
        cmd: protocol::CV_USB_BREAK,
        size: 0,
        addr: protocol::DUMMY_ADDR,
    };
    info!("{h:?}");
    let data = h.to_slice();
    send(port, &data);
}

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
    send_magic(&mut port);
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
    send_file(&mut port, HEADER);
    send_flag_and_break(&mut port);
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
    send_magic(&mut port);

    println!("send PAYLOAD...");
    send_file(&mut port, OREBOOT);
    send_flag_and_break(&mut port);

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
