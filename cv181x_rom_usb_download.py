#!/usr/bin/python3
from cv_usb_util.cv_usb_pyserial import cv_usb_pyserial
import argparse
import logging
import os
import sys
import cv_usb_util.cv_usb_pkt as pkt
from array import array
import time

parser = argparse.ArgumentParser(description="usb download scripts for CV181X")


def parse_Args():
    cur_dir = os.path.abspath(os.getcwd())
    parser.add_argument(
        "--image_dir",
        metavar="path",
        type=str,
        default=cur_dir,
        help="the folder path to dir inclued fip,rootfs kernel and xml",
    )
    parser.add_argument(
        "-v", "--verbose", help="increase output verbosity", action="store_true"
    )
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--serial", action="store_true", default=False)
    group.add_argument("--libusb", action="store_true", default=False)

    args = parser.parse_args()

    return args


def usage():
    parser.print_usage()


def resource_path(relative_path):
    """Get absolute path to resource, works for dev and for PyInstaller"""
    try:
        # PyInstaller creates a temp folder and stores path in _MEIPASS
        base_path = sys._MEIPASS
    except Exception:
        base_path = os.path.dirname(os.path.realpath(__file__))
        return os.path.join(base_path, relative_path)


# This is also found in FSBL code itself; grep for MGN1:
#   #define MAGIC_NUM_USB_DL 0x4D474E31 // MGN1
flag = array("B", [ord(c) for c in "1NGM"])

# 1. send magic
# 2. send chunk
# 3. send flag
# 4. send break
def main():
    args = parse_Args()
    image_dir = args.image_dir
    fip_path = os.path.join(image_dir, "fip.bin")
    fip_tx_size = 4 * 1024

    magic = open(resource_path("cv_dl_magic.bin"), "rb")
    magic_size = os.path.getsize(resource_path("cv_dl_magic.bin"))

    print("CV181X USB download start")
    cv_usb_serial = cv_usb_pyserial()
    cv_usb_serial.serial_query([pkt.rom_vidpid])
    cv_usb_serial.usb_send_chunk(magic, magic_size, pkt.DUMMY_ADDR, 0, Type="magic")

    cv_usb_serial.serial_query([pkt.rom_vidpid])

    print("Send %dB fip.bin..." % fip_tx_size)
    fip_bin = open(fip_path, "rb")
    cv_usb_serial.usb_send_chunk(fip_bin, fip_tx_size, 0, 0, Type="file")
    cv_usb_serial.usb_send_req_data(pkt.CVI_USB_TX_FLAG, 0x0E000004, 12, flag)
    cv_usb_serial.usb_send_req_data(pkt.CV_USB_BREAK, pkt.DUMMY_ADDR, 0, None)

    # Add delay to avoid open the previous disconnected com port.
    time.sleep(0.1)
    cv_usb_serial.serial_query([pkt.rom_vidpid, pkt.uboot_cvi_vidpid], 1)

    magic = open(resource_path("cv_dl_magic.bin"), "rb")
    magic_size = os.path.getsize(resource_path("cv_dl_magic.bin"))
    cv_usb_serial.usb_send_chunk(magic, magic_size, pkt.DUMMY_ADDR, 0, Type="magic")

    if True:
        fip_bin.seek(pkt.FIP_TX_OFFSET)
        cv_usb_serial.usb_send_chunk(fip_bin, pkt.FIP_TX_SIZE, 0, 0, Type="file")
    else:
        ore_path = os.path.join(".", "oreboot_x.bin")
        ore_bin = open(ore_path, "rb")
        ore_size = os.path.getsize(ore_path)
        cv_usb_serial.usb_send_chunk(ore_bin, ore_size, 0, 0, Type="file")

    cv_usb_serial.usb_send_req_data(pkt.CVI_USB_TX_FLAG, 0x0E000004, 12, flag)
    cv_usb_serial.usb_send_req_data(pkt.CV_USB_BREAK, pkt.DUMMY_ADDR, 0, None)

if __name__ == "__main__":
    main()
