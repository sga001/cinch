use std::str;

use usb;
use parser::usbr;
use parser::Source;
use byteorder::{BigEndian, ByteOrder};

fn check_ieee_1284_string(data: &[u8]) -> bool {

    let s: &str = match str::from_utf8(data) {
        Ok(v) => v,
        Err(e) => {
            error!("[E007-PRINTER] id not valid utf8 string {}", e);
            return false;
        }
    };

    // 3 required keys
    let mut man_found: bool = false;
    let mut com_found: bool = false;
    let mut mod_found: bool = false;

    for lines in s.trim().split(';') {

        let kv: Vec<&str> = lines.split(':').collect();

        if kv.len() == 1 {
            continue;
        }

        if kv.len() != 2 {
            error!("[E008-PRINTER] Invalid string with more than 1 ':' between ';'");
            return false;
        }

        if kv[0].contains("MANUFACTURER") || kv[0].contains("MFG") {
            man_found = true;
            // no checks
        } else if kv[0].contains("COMMAND SET") || kv[0].contains("CMD") {
            com_found = true;
            // check values
            for value in kv[1].split(',') {
                match value {
                    "1284.4" |
                    "ACL" |
                    "BDC" |
                    "BIDI-ECP" |
                    "BJ" |
                    "BJL" |
                    "BJRaster" |
                    "BJRaster3" |
                    "BSCC" |
                    "BJScan2" |
                    "C32" |
                    "CLS" |
                    "CPDNPA001" |
                    "D4" |
                    "DESKJET" |
                    "DW-PCL" |
                    "DYN" |
                    "ECP18" |
                    "EJL" |
                    "EPSON" |
                    "EPSONFX" |
                    "ESCP9-84" |
                    "ESCPAGE-04" |
                    "ESCPAGES-02" |
                    "ESCPL2" |
                    "ESCPL2-00" |
                    "ESC/P2" |
                    "FastRaster" |
                    "HIPERWINDOWS" |
                    "HPGL2-01" |
                    "HPENHANCEDPCL5" |
                    "HPENHANCEDPCL5e" |
                    "HPENHANCEDPCL6" |
                    "HPGDI" |
                    "IBM" |
                    "IBMPPR" |
                    "LAVAFLOW" |
                    "LDL" |
                    "LEXWPS" |
                    "LNPAP" |
                    "LOWENDLEXCPD" |
                    "lpt1" |
                    "LQ" |
                    "MFPDTF1" |
                    "MFPXDMA" |
                    "MICROLINE" |
                    "MIME" |
                    "MLC" |
                    "NA" |
                    "none" |
                    "NPAP" |
                    "MultiPass2.1" |
                    "OAKRAS" |
                    "OPEL" |
                    "PCL" |
                    "PCL3" |
                    "PCL4" |
                    "PCL5" |
                    "PCL5C" |
                    "PCL5e" |
                    "PCL5E2" |
                    "PCL6" |
                    "PCLXL" |
                    "PCL5Emulation" |
                    "PCL-XL" |
                    "PDF" |
                    "PJL" |
                    "PML" |
                    "POSTSCRIPT" |
                    "POSTSCRIPT2" |
                    "PostScriptLevel2Emulation" |
                    "PostScriptLevel3ForMacEmulation" |
                    "PrintGear" |
                    "PRPXL24" |
                    "PRPXL24-01" |
                    "PS" |
                    "PT-CBP" |
                    "RPCS" |
                    "SCP" |
                    "TXT01" |
                    "VLINK" |
                    "WinStyler" |
                    "XHTML" |
                    "ZJS" => {}
                    _ => {
                        error!("[E009-PRINTER] Invalid command set value {}", value);
                        return false;
                    }
                }
            }
        } else if kv[0].contains("MODEL") || kv[0].contains("MDL") {
            mod_found = true;
        }
    }

    if !man_found || !com_found || !mod_found {
        error!("[E010-PRINTER] Required key for 1284 device id not found");
        return false;
    }

    true
}


pub fn check_printer_request(h: &usbr::ControlPacketHeader, data: &[u8], source: Source) -> bool {

    match h.request {

        usb::printer::GET_DEVICE_ID => {
            if source != Source::Blue {

                if data.len() < 2 {
                    error!("[E001-PRINTER] Not enough payload for request");
                    return false;
                }

                // first 2 bytes of data are length. check this is the case.
                // check the rest is a valid IEEE1284 string

                let len: usize = BigEndian::read_u16(&data[0..2]) as usize;

                if len != data.len() {
                    error!("[E002-PRINTER] Invalid length in device id");
                    return false;
                }

                if !check_ieee_1284_string(&data[2..]) {
                    return false;
                }
            }
        }

        usb::printer::GET_PORT_STATUS => {

            if source != Source::Blue {

                if data.len() != 1 {
                    error!("[E003-PRINTER] Invalid length of port status");
                    return false;
                }

                // only bits 3, 4, 5 can be used.
                if data[0] & 0xc7 != 0 {
                    error!("[E004-PRINTER] Stats uses reserved bits");
                    return false;
                }
            }
        }

        usb::printer::SOFT_RESET => {
            if source != Source::Blue {
                error!("[E005-PRINTER] Reset sent by device and not by host");
                return false;
            }
        }

        _ => {
            error!("[E006-PRINTER] Unknown request type 0x{:x}", h.request);
            return false;
        }
    }

    true
}
