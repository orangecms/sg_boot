#!/bin/sh
BASE=https://github.com/milkv-duo/duo-buildroot-sdk/raw/develop/build/tools/cv181x/usb_dl/rom_usb_dl/

mkdir -p cv_usb_util

wget -P cv_usb_util $BASE/cv_usb_util/__init__.py
wget -P cv_usb_util $BASE/cv_usb_util/cv_usb.py
wget -P cv_usb_util $BASE/cv_usb_util/cv_usb_libusb.py
wget -P cv_usb_util $BASE/cv_usb_util/cv_usb_pkt.py
wget -P cv_usb_util $BASE/cv_usb_util/cv_usb_pyserial.py

wget $BASE/XmlParser.py
wget $BASE/cv181x_rom_usb_download.py
wget $BASE/cv181x_uboot_usb_download.py
wget $BASE/cv_dl_magic.bin
wget $BASE/usb_script.its

