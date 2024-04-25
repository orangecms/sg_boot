use log::{debug, info};

const NO_MAGIC: &[u8] = include_bytes!("../nomagic.bin");

pub const SUCCESS: usize = 0;
pub const FAIL: usize = 1;

const CHUNK_SIZE: usize = 256;
const FLAG_ADDR: u64 = 0x0E000004;
const FLAG: [u8; 4] = *b"1NGM";

// Memory addresses
pub const DUMMY_ADDR: u64 = 0xFF;
pub const DDR_FIP_ADDR: u64 = 0x8080_0000;
pub const IMG_ADDR: u64 = 0x8394_0000;

// ROM USB command;
pub const CVI_USB_TX_DATA_TO_RAM: u8 = 0;
pub const CVI_USB_TX_FLAG: u8 = 1;
// Common command;
pub const CV_USB_BREAK: u8 = 2;
pub const CV_USB_KEEP_DL: u8 = 3;
pub const CV_USB_UBREAK: u8 = 4;
pub const CV_USB_PRG_CMD: u8 = 6;
pub const CVI_USB_REBOOT: u8 = 22;
pub const CVI_USB_PROGRAM: u8 = 0x83;

pub const MSG_TOKEN_OFFSET: usize = 0;

type Port = std::boxed::Box<dyn serialport::SerialPort>;

#[derive(Debug)]
struct Header {
    cmd: u8,
    size: u16,
    addr: u64,
}

const HEADER_SIZE: usize = 8;

impl Header {
    fn to_slice(&self) -> [u8; HEADER_SIZE] {
        // add header's own size
        let sz = self.size + HEADER_SIZE as u16;
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

const NO_MAGIC_HEADER: Header = Header {
    cmd: CV_USB_KEEP_DL,
    size: NO_MAGIC.len() as u16,
    addr: DUMMY_ADDR,
};

const FLAG_HEADER: Header = Header {
    cmd: CVI_USB_TX_FLAG,
    size: FLAG.len() as u16,
    addr: FLAG_ADDR,
};

const BREAK_HEADER: Header = Header {
    cmd: CV_USB_BREAK,
    size: 0,
    addr: DUMMY_ADDR,
};

const RESPONSE_SIZE: usize = 16;

// CRC variant figured out by trying all the CRC16s offered by crc crate :D
const CRC: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);

const RSP_CRC16_HI_OFFSET: usize = 2;
const RSP_CRC16_LO_OFFSET: usize = 3;
const RSP_TOKEN_OFFSET: usize = 6;

fn check_response(data: &[u8], resp: &[u8]) {
    info!("response: {resp:x?}");

    let crc_hi = resp[RSP_CRC16_HI_OFFSET];
    let crc_lo = resp[RSP_CRC16_LO_OFFSET];
    let rsp_checksum = ((crc_hi as u16) << 8) | crc_lo as u16;

    let exp_checksum = CRC.checksum(data);

    if exp_checksum != rsp_checksum {
        panic!("Checksum mismatch: got {rsp_checksum:04x}, expected {exp_checksum:04x}");
    }

    info!("checksum {rsp_checksum:04x} == {exp_checksum:04x}");

    let rsp_token = resp[RSP_TOKEN_OFFSET];
    info!("token: {rsp_token}");
}

fn send(port: &mut Port, data: &[u8]) {
    let sent = port.write(data).expect("Write failed!");
    let mut resp: Vec<u8> = vec![0; RESPONSE_SIZE];
    let read = port.read(resp.as_mut_slice()).expect("Found no data!");
    check_response(data, &resp);
    info!("sent {sent} bytes, read {read} bytes");
}

pub fn concat(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().chain(b).copied().collect()
}

pub fn send_magic(port: &mut Port) {
    println!("send NO MAGIC...");
    info!("{NO_MAGIC_HEADER:?}");
    let data = concat(&NO_MAGIC_HEADER.to_slice(), NO_MAGIC);
    info!("{data:x?}");
    send(port, &data);
}

pub fn send_file(port: &mut Port, f: &[u8]) {
    for (i, chunk) in f.chunks(CHUNK_SIZE).enumerate() {
        let h = Header {
            cmd: CVI_USB_TX_DATA_TO_RAM,
            size: chunk.len() as u16,
            addr: (i * CHUNK_SIZE) as u64,
        };
        info!("{h:?}");
        let data = concat(&h.to_slice(), chunk);
        debug!("{data:x?}");
        send(port, &data);
    }
}

pub fn send_flag_and_break(port: &mut Port) {
    let data = concat(&FLAG_HEADER.to_slice(), &FLAG);
    debug!("{data:x?}");
    send(port, &data);
    debug!("{BREAK_HEADER:?}");
    let data = BREAK_HEADER.to_slice();
    send(port, &data);
}
