pub const SUCCESS: usize = 0;
pub const FAIL: usize = 1;
pub const TIMEOUT: isize = -1;

pub const HEADER_SIZE: usize = 8;

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

// Cannot be too large on Windows!
pub const USB_BULK_MAX_SIZE: usize = 0x80000; // 0x4000000;

pub const MSG_TOKEN_OFFSET: usize = 0;

pub const RSP_CRC16_HI_OFFSET: usize = 2;
pub const RSP_CRC16_LO_OFFSET: usize = 3;
pub const RSP_TOKEN_OFFSET: usize = 6;
