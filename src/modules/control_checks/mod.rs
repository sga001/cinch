use std::sync::RwLock;
use std::collections::HashMap;
use byteorder::{ByteOrder, LittleEndian};
use conv::TryFrom;

// Cinch libraries
use parser::usbr;
use parser::{HasHandlers, Request, Source};
use usb;
use util::lockext::RwLockExt;


macro_rules! parse_descriptor {

    // Descriptor header
    (0, $data:expr) => (
        &*($data[..2].as_ptr() as *const usb::DescriptorHeader);
    );

    (usb::DT_DEVICE, $data:expr) => (
        usb::DeviceDescriptor {
            bcd_usb: LittleEndian::read_u16(&$data[0..2]),
            device_class: $data[2],
            device_subclass: $data[3],
            device_protocol: $data[4],
            max_packet_size0: $data[5],
            id_vendor: LittleEndian::read_u16(&$data[6..8]),
            id_product: LittleEndian::read_u16(&$data[8..10]),
            bcd_device: LittleEndian::read_u16(&$data[10..12]),
            manufacturer: $data[12],
            product: $data[13],
            serial_number: $data[14],
            num_configurations: $data[15],
        }
    );

    (usb::DT_CONFIG, $data:expr) => (
        usb::ConfigDescriptor {
            total_length: LittleEndian::read_u16(&$data[0..2]),
            num_interfaces: $data[2],
            configuration_value: $data[3],
            configuration: $data[4],
            attributes: $data[5],
            max_power: $data[6],
        }
    );

    (usb::DT_INTERFACE, $data:expr) => (
        usb::InterfaceDescriptor {
            interface_number: $data[0],
            alternate_setting: $data[1],
            num_endpoints: $data[2],
            interface_class: $data[3],
            interface_subclass: $data[4],
            interface_protocol: $data[5],
            interface: $data[6],
        }
    );

    (usb::DT_ENDPOINT, $data:expr) => (
        usb::EndpointDescriptor {
           endpoint_address: $data[0],
           attributes: $data[1],
           max_packet_size: LittleEndian::read_u16(&$data[2..4]),
           interval: $data[4],
        }
    );

    (usb::DT_SS_ENDPOINT_COMP, $data:expr) => (
        usb::SsEpCompDescriptor {
            max_burst: $data[0],
            attributes: $data[1],
            bytes_per_interval: LittleEndian::read_u16(&$data[2..4]),
        }
    );

    (usb::DT_PIPE_USAGE, $data:expr) => (
        usb::PipeUsageDescriptor {
            pipe_id: $data[0],
            reserved: $data[1],
        }
    );

    (usb::DT_INTERFACE_ASSOCIATION, $data:expr) => (
        &*($data[..6].as_ptr() as *const usb::InterfaceAssocDescriptor);
    );

    (usb::DT_DEVICE_QUALIFIER, $data:expr) => {
        usb::DeviceQualifierDescriptor {
            bcd_usb: LittleEndian::read_u16(&$data[0..2]),
            device_class: $data[2],
            device_subclass: $data[3],
            device_protocol: $data[4],
            max_packet_size0: $data[5],
            num_configurations: $data[6],
            reserved: $data[7],
        }
    };

    (usb::DT_OTG, $data:expr) => {
        &*($data[..1].as_ptr() as *const usb::OtgDescriptor);
    };

    (usb::DT_BOS, $data:expr) => {
        usb::BosDescriptor {
            total_length: LittleEndian::read_u16(&$data[0..2]),
            num_device_caps: $data[2],
        }
    };

    (usb::DT_DEBUG, $data:expr) => {
        &*($data[..2].as_ptr() as *const usb::DebugDescriptor);
    };

    (usb::DT_DEVICE_CAPABILITY, $data:expr) => {
      &*($data[..1].as_ptr() as *const usb::DevCapHeader);
    };

    // Capability Type Descriptors
    (usb::CAP_TYPE_SS, $data:expr) => {
        usb::SsCapDescriptor {
            attributes: $data[0],
            speed_supported: LittleEndian::read_u16(&$data[1..3]),
            functionality_support: $data[3],
            u1_dev_exit_lat: $data[4],
            u2_dev_exit_lat: LittleEndian::read_u16(&$data[5..7]),
        }
    };

    (usb::CAP_TYPE_EXT, $data:expr) => {
        usb::ExtCapDescriptor {
            attributes: LittleEndian::read_u32(&$data[0..4]),
        }
    };

    (usb::CAP_TYPE_CONTAINER_ID, $data:expr) => {
        {
            let mut con = usb::ContainerIdDescriptor {
                reserved: $data[0],
                container_id: [0; 16],
            };

            con.container_id.clone_from_slice(&$data[1..17]);
            con
        }
    };
}

macro_rules! has_interface {
    ($vdev:ident, $i:expr, $inum:expr) => {
        $vdev.configs.get(&$i).unwrap().interfaces.contains_key(&$inum)
    }
}

macro_rules! has_alternate {
    ($vdev:ident, $i:expr, $inum:expr, $alt:expr) => {
        $vdev.configs.get(&$i).unwrap().interfaces.get(&$inum).unwrap().contains_key(&$alt)
    }
}


macro_rules! get_current_interface {
    ($vdev:ident, $i:expr) => {
        {
            let valid = if $vdev.chosen_conf.is_none() {
                error!("[E153] no chosen configuration available");
                false
            } else if !$vdev.configs.contains_key(&$vdev.chosen_conf.unwrap()) {
                error!("[E154] configuration list not available");
                false
            } else if !$vdev.configs.get(&$vdev.chosen_conf.unwrap()).unwrap().interfaces.contains_key(&$i){
                error!("[E155] interface not available");
                false
            } else if !$vdev.chosen_interfaces.contains_key(&$i) {
                error!("[E156] chosen interface not properly set up");
                false
            } else {
                true
            };

            if valid {
                let alt = $vdev.chosen_interfaces.get(&$i).unwrap();

                let i_map = &$vdev.configs.get(&$vdev.chosen_conf.unwrap()).unwrap().interfaces.get(&$i).unwrap();

                if !i_map.contains_key(&alt) {
                    error!("[E157] alternate interface not avaialble in model");
                    None
                } else {
                   Some(i_map.get(&alt).unwrap())
                }

            } else {
                None
            }
        }
    }
}

// Import class-specific modules
mod hid;
mod bbb;
mod printer;
mod third_party;

const NO_MATCH: u8 = 0; // request is valid
const MATCH: u8 = 1;

struct ConfigNode {
    desc: usb::ConfigDescriptor,
    interfaces: HashMap<u8, HashMap<u8, InterfaceNode>>,
}

#[allow(dead_code)]
struct InterfaceNode {
    desc: usb::InterfaceDescriptor,
    endpoints: HashMap<u8, EndpointNode>,
}

#[allow(dead_code)]
struct EndpointNode {
    desc: usb::EndpointDescriptor,
    ss_desc: Option<usb::SsEpCompDescriptor>,
    pipe_desc: Option<usb::PipeUsageDescriptor>,
}

struct StringNode {
    desc: Vec<u8>,
}

struct VirtualDevice {
    desc: Option<usb::DeviceDescriptor>,
    configs: HashMap<u8, ConfigNode>,
    strings: HashMap<u8, StringNode>,

    chosen_conf: Option<u8>, // currently chosen configuration
    chosen_interfaces: HashMap<u8, u8>, // currently chosen alternative for each interface
}

impl VirtualDevice {
    fn new() -> VirtualDevice {
        VirtualDevice {
            desc: None,
            configs: HashMap::new(),
            strings: HashMap::new(),
            chosen_conf: None,
            chosen_interfaces: HashMap::new(),
        }
    }
}


pub struct ControlCheck {
    vdev: RwLock<VirtualDevice>,
    hid_checks: RwLock<hid::HidControlCheck>,
    bbb_checks: RwLock<bbb::BBBControlCheck>,
    third_party: RwLock<third_party::Patcher>,
}


macro_rules! control_match {

    ($req:expr, $msg:expr) => {
        error!("[E000a] Invalid {}", $msg);
        return (MATCH, vec![$req]);
    };

    ($req:expr, $fmt:expr, $($args:tt)*) => {
        error!(concat!("[E000b] Invalid ", $fmt), $($args)*);
        return (MATCH, vec![$req]);
    };

}

macro_rules! usb_v {
    ($bcd:expr, 1) => ($bcd >= usb::V1 && $bcd < usb::V2);
    ($bcd:expr, 2) => ($bcd >= usb::V2 && $bcd < usb::V3);
    ($bcd:expr, 3) => ($bcd >= usb::V3 && $bcd < 0x0400);
}


fn check_bcd(bcd: u16) -> bool {

    // from http://homepage.cs.uiowa.edu/~jones/bcd/bcd.html#packed

    let bcd32: u32 = bcd as u32;
    ((((bcd32 + 0x06666666) ^ bcd32) & 0x11111110) == 0)
}


fn check_get_status(h: &usbr::ControlPacketHeader, data: &[u8]) -> bool {

    // From Section 9.4.5, Pages 361-363, spec/usb3.pdf

    if data.len() != 2 {
        error!("[E001] Invalid size of return to get status");
        return false;
    }

    // For device, only [0:4] can be used
    if (h.requesttype & usb::RECIP_MASK) == usb::RECIP_DEVICE && (LittleEndian::read_u16(data) & 0xffe0) != 0 {

        error!("[E002] Device get status uses reserved bits");
        return false;

    }

    // For interface, only [0:1] can be used
    if (h.requesttype & usb::RECIP_MASK) == usb::RECIP_INTERFACE && (LittleEndian::read_u16(data) & 0xfffc) != 0 {

        error!("[E003] Interface get status uses reserved bits");
        return false;
    }

    // For endpoint, only the last bit can be used
    if (h.requesttype & usb::RECIP_MASK) == usb::RECIP_ENDPOINT && (LittleEndian::read_u16(data) & 0xfffe != 0) {

        error!("[E004] Endpoint get status uses reserved bits");
        return false;
    }

    true
}

fn check_device_csp(class: u8, sc: u8, prot: u8) -> bool {

    let (sflag, mut pflag) = match class {

        usb::CLASS_PER_INTERFACE |
        usb::CLASS_BILLBOARD => (sc == 0x00, prot == 0x00),
        usb::CLASS_COMM => (sc > 0x00 && sc < 0x0e, prot == 0x00),
        usb::CLASS_HUB => (sc == 0x00, prot >= 0x01 && prot <= 0x03),
        usb::CLASS_DIAGNOSTIC => (sc == 0x01, prot == 0x01),
        usb::CLASS_MISC => (sc == 0x01 || sc == 0x02, prot == 0x01 || prot == 0x02),
        usb::CLASS_VENDOR_SPEC => (true, true),
        _ => (false, false),
    };

    pflag = pflag || (prot == 0xff);

    sflag && pflag
}

fn check_interface_csp(iface: &usb::InterfaceDescriptor, dev: &usb::DeviceDescriptor) -> bool {

    let d_class = dev.device_class;
    let d_sc = dev.device_subclass;
    let d_proto = dev.device_protocol;

    let i_class = iface.interface_class;
    let i_sc = iface.interface_subclass;
    let i_proto = iface.interface_protocol;


    let mut sflag: bool = false; // subclass flag
    let mut pflag: bool = false; // protocol flag
    let mut dflag: bool = d_class == usb::CLASS_PER_INTERFACE ||
                          (d_class == usb::CLASS_MISC && d_sc == 0x02 && d_proto == 0x01);
    match i_class {

        usb::CLASS_HUB => {
            dflag = d_class == usb::CLASS_HUB;
            sflag = i_sc == 0;
            pflag = i_proto == 0;
        }

        usb::CLASS_AUDIO => {
            sflag = i_sc <= 0x03;
            pflag = i_proto == 0x00 || i_proto == 0x20;
        }

        usb::CLASS_COMM => {
            dflag = d_class == usb::CLASS_COMM;
            sflag = (i_sc >= 0x01 && i_sc <= 0x0d) || (i_sc >= 0x80 && i_sc <= 0xfe);
            pflag = i_proto <= 0x07 || i_proto == 0xfe;
        }

        usb::CLASS_HID => {
            sflag = i_sc <= 0x01;
            pflag = i_proto <= 0x02;
        }

        usb::CLASS_HEALTH |
        usb::CLASS_PHYSICAL |
        usb::CLASS_CONTENT_SEC => {
            sflag = i_sc == 0x00;
            pflag = i_proto == 0x00;
        }

        usb::CLASS_STILL_IMAGE |
        usb::CLASS_DIAGNOSTIC => {
            sflag = i_sc == 0x01;
            pflag = i_proto == 0x01;
        }

        usb::CLASS_PRINTER => {
            sflag = i_sc == 0x01;
            pflag = i_proto <= 0x03;
        }

        // i_sc
        // 0x01 Reduced block command (RBC)
        // 0x02 MMC-5 (ATAPI)
        // 0x04 UFI (Floppy disk drives to USB)
        // 0x06 SCSI command
        // 0x07 LSD FS (specifies how host negotiates access before SCSI)
        // 0x08 IEEE 1667
        //
        // i_proto
        // 0x00 Control-bulk-interrupt (CBI) with command completion interrupt
        // 0x01 CB with no command completion interrupt
        // 0x50 BBB (Bulk only)
        // 0x62 UAS (USB attached SCSI)
        usb::CLASS_MASS_STORAGE => {
            sflag = i_sc <= 0x02 || i_sc == 0x04 || i_sc == 0xff || (i_sc >= 0x06 && i_sc <= 0x08);
            pflag = i_proto <= 0x01 || i_proto == 0x50 || i_proto == 0x62;
        }

        usb::CLASS_CDC_DATA => {
            dflag = d_class == usb::CLASS_COMM;
            sflag = i_sc == 0x00;
            pflag = i_proto <= 0x01 || (i_proto >= 0x30 && i_proto <= 0x32) ||
                    (i_proto >= 0x50 && i_proto <= 0x52) ||
                    (i_proto >= 0x90 && i_proto <= 0x93) ||
                    (i_proto >= 0xfd && i_proto <= 0xfe);
        }

        usb::CLASS_CSCID => {
            sflag = i_sc == 0x00;
            pflag = i_proto == 0x02;
        }

        usb::CLASS_VIDEO => {
            sflag = i_sc <= 0x03;
            pflag = i_proto <= 0x01;
        }

        usb::CLASS_AUDIO_VIDEO => {
            sflag = i_sc <= 0x03;
            pflag = i_proto == 0x00 || i_proto == 0x10;
        }

        usb::CLASS_WIRELESS_CONTROLLER => {
            sflag = i_sc == 0x01 || i_sc == 0x02;
            pflag = (i_proto >= 0x01 && i_proto <= 0x03) || (i_sc == 0x01 && i_proto == 0x04);
        }

        usb::CLASS_MISC => {
            match i_sc {
                x if x == 0x01 || x == 0x05 => {
                    sflag = true;
                    pflag = i_proto == 0x01 || i_proto == 0x02;
                }

                0x03 => {
                    sflag = true;
                    pflag = i_proto == 0x01;
                }

                0x04 => {
                    sflag = true;
                    pflag = i_proto >= 0x01 && i_proto <= 0x07;
                }

                _ => {}
            }
        }

        usb::CLASS_APP_SPEC => {
            match i_sc {
                0x01 => {
                    sflag = true;
                    pflag = i_proto == 0x01;
                }

                0x02 => {
                    sflag = true;
                    pflag = i_proto == 0x00;
                }

                0x03 => {
                    sflag = true;
                    pflag = i_proto == 0x00 || i_proto == 0x01;
                }

                _ => {}
            }
        }

        usb::CLASS_VENDOR_SPEC => {
            sflag = true;
            pflag = true;
        }

        _ => {}
    }

    pflag = pflag || i_proto == 0xff;

    (dflag && sflag && pflag)
}

fn check_device_fields(data: &[u8]) -> bool {

    // From Section 9.6.1, pages 371-373 in spec/usb3.pdf
    // From Section 9.6.1, pages 261-264 in spec/usb2.pdf
    //
    // (1) Is the descriptor length correct?
    // (2) Is it a valid string encoding for USB version and device id
    // (3) Are the device class, subclass, and protocol consistent
    // (4) Is the max packet size valid
    // (5) Ensure it has at least one configuration

    // These checks must account for any prefix of the request received.
    // This is because usb_reset_and_verify_device() in drivers/usb/core/hub.c
    // has this crazy behavior where it sometimes asks for 64 bytes (in an
    // attempt to do what Windows does because "compatibility"...), sometimes
    // for 18 (the actual size), and sometimes for 8 bytes (based on timeouts
    // and device quirks).

    let mut bcd_usb: u16 = 0;

    if data.len() > usb::DEVICE_DESC_SIZE {
        error!("[E005] invalid length");
        return false;
    }

    if data.len() >= 2 {
        bcd_usb = LittleEndian::read_u16(&data[0..2]);

        if !check_bcd(bcd_usb) {
            error!("[E006] Invalid bcd_usb: 0x{:x}", bcd_usb);
            return false;
        }
    }


    if data.len() >= 5 {
        let class: u8 = data[2];
        let sc: u8 = data[3];
        let proto: u8 = data[4];

        if !check_device_csp(class, sc, proto) {
            error!("[E007] Invalid device class / subclass / proto: 0x{:x} / 0x{:x} / 0x{:x}",
                   class,
                   sc,
                   proto);
            return false;
        }

    }


    if data.len() >= 6 {
        // max_packet_size0 should be 8, 16, 32, or 64. It can be 9 if it is USB 3 and the
        // device uses superspeed

        let max_size: u8 = data[5];

        if !(max_size == 8 || max_size == 16 || max_size == 32 || max_size == 64 ||
             (max_size == 9 && bcd_usb == 0x0300)) {
            error!("[E008] Invalid device max_packet_size0 {}, bcd_device {:x}",
                   max_size,
                   bcd_usb);
            return false;
        }
    }

    if data.len() >= 12 {

        let bcd_device: u16 = LittleEndian::read_u16(&data[10..12]);

        if !check_bcd(bcd_device) {
            error!("[E009] invalid bcd_device: 0x{:4.4x}.", bcd_device);
            return false;
        }
    }


    if data.len() == usb::DEVICE_DESC_SIZE {

        let num_configs: u8 = data[usb::DEVICE_DESC_SIZE - 1];

        if num_configs == 0 {
            error!("[E010] Invalid number of configurations");
            return false;
        }

    }

    true
}

fn check_string_fields(data: &[u8], index: u8) -> bool {

    // From Section 9.6.8, pages 388-389 in spec/usb3.pdf
    // From Section 9.6.7, pages 273-274 in spec/usb2.pdf
    // From spec/string-langids.pdf

    if data.len() % 2 == 1 || data.len() < 2 {
        error!("[E011] String descriptor with odd or invalid length");
        return false;
    }

    if index == 0 {

        let mut i: usize = 0;

        while i < data.len() - 1 {
            let lang_id: u16 = LittleEndian::read_u16(&data[i..i + 2]);

            if usb::LangIds::try_from(lang_id).is_err() {
                error!("[E124] Invalid language id in string descriptor 0x{:x}", lang_id);
                return false;
            }

            i += 2;
        }

        return true;
    }


    // valid UTF-16:
    // - normal codepoints are [0x0000, 0xD7FF] or [0xE000, 0xFFFF]
    // - surrogate pairs are in high-low order
    //   - high surrogate is [0xD800, 0xDBFF]
    //   - low surrogate is [0xDC00, 0xDFFF]

    let mut i: usize = 0;

    while i < data.len() - 1 {

        let mut codepoint: u16 = LittleEndian::read_u16(&data[i..i + 2]);

        // normal codepoints
        if codepoint <= 0xd7ff || codepoint >= 0xe000 {
            i += 2;
            continue;
        }

        // otherwise this better be a high surrogate
        // which means this better not be the last character
        if i + 2 == data.len() {
            return false;
        }

        // a valid surrogate
        if codepoint >= 0xd800 && codepoint <= 0xdbff {
            i += 2; // this is valid because of high surrogate check

            // valid low surrogate
            codepoint = LittleEndian::read_u16(&data[i..i + 2]);

            if codepoint >= 0xdc00 && codepoint <= 0xdfff {
                i += 2;
                continue;
            }
        }

        // if we fell through, this is an invalid UTF-16 string
        error!("[E013] String descriptor is not valid UTF-16");
        return false;
    }

    true
}


fn check_config_fields(config: &usb::ConfigDescriptor, dev: &usb::DeviceDescriptor) -> bool {

    // From Section 9.6.3, pages 377-378 in spec/usb3.pdf
    // From Section 9.6.3, pages 264-266 in spec/usb2.pdf

    // min length = 1 config, 1 interface, 0 endpoints

    if (config.total_length as usize) < usb::CONFIG_DESC_SIZE + usb::INTERFACE_DESC_SIZE + 2 * usb::HEADER_SIZE {

        error!("[E014] Invalid config descriptor length {}", config.total_length);
        return false;

    }

    if config.num_interfaces < 1 {
        error!("[E015] Invalid number of interfaces {}", config.num_interfaces);
        return false;
    }

    if dev.device_class == usb::CLASS_COMM && config.num_interfaces < 2 {
        error!("[E166] Invalid number of interfaces {} for cdc device",
               config.num_interfaces);
        return false;
    }

    // attributes[7] must be 1; attributes[4:0] must be 0

    if (config.attributes & 0x9f) != 0x80 {
        error!("[E016] Invalid config attributes: 0x{:x}", config.attributes);
        return false;
    }

    if config.max_power > 250 {
        error!("[E017] Invalid max power (>250 mA): {}", config.max_power);
        return false;
    }

    true
}


fn check_other_speed_fields(desc: &usb::OtherSpeedDescriptor) -> bool {

    // From Section 9.6.4, pages 266-267 in spec/usb2.pdf

    if (desc.attributes & 0x9f) != 0x80 {
        error!("[E116] Other speed attribute[4:0] must be 0");
        return false;
    }

    if desc.max_power > 250 {
        error!("[E117] Invalid other speed max power (>250 mA): {}", desc.max_power);
        return false;
    }

    true
}

fn check_interface_fields(iface: &usb::InterfaceDescriptor, dev: &usb::DeviceDescriptor) -> bool {

    // From Section 9.6.5, pages 380-382 in spec/usb3.pdf
    // From Section 9.6.5, pages 267-269 in spec/usb2.pdf

    if iface.num_endpoints > 30 {
        error!("[E018] Invalid number of endpoints (>30): {}", iface.num_endpoints);
        return false;
    }

    if !check_interface_csp(iface, dev) {
        error!("[E019] Invalid interface class/sclass/proto: {:x}/{:x}/{:x}",
               iface.interface_class,
               iface.interface_subclass,
               iface.interface_protocol);
        return false;
    }

    true
}

fn check_interface_assoc_fields(iface: &usb::InterfaceAssocDescriptor) -> bool {

    // From spec/interface-assoc.pdf

    if iface.function_class == 0 {
        error!("[E099] Invalid interface associatoin function class value");
        return false;
    }

    true
}

fn check_endpoint_fields(ep: &usb::EndpointDescriptor, dev: &usb::DeviceDescriptor) -> bool {

    // From Section 9.6.6, pages 382-384 in spec/usb3.pdf
    // From Section 9.6.6, pages 269-273 in spec/usb2.pdf

    if (ep.endpoint_address & 0x70) != 0 {
        error!("[E020] Invalid endpoint number: {:?}", ep.endpoint_address);
        return false;
    }

    // bits 6 and 8 are reserved
    if (ep.attributes & 0xc0) != 0 {
        error!("[E021] Invalid endpoint with attributes[6:7] set");
        return false;
    }

    if !usb_v!(dev.bcd_usb, 3) && (ep.max_packet_size & 0xe000) != 0 {
        // top 3 bits of max packet size must be clear
        error!("[E022] Invalid endpoint with packet_size[13:15] set");
        return false;
    }

    // checks based on endpoint type
    match ep.attributes & usb::ENDPOINT_XFERTYPE_MASK {

        usb::ENDPOINT_XFER_CONTROL => {

            if (ep.attributes & 0x3c) != 0 {
                error!("[E023] Invalid control endpoint with attributes[2:5] set");
                return false;
            }

            if (usb_v!(dev.bcd_usb, 1) || usb_v!(dev.bcd_usb, 2)) && ep.max_packet_size & 0x1800 != 0 {

                error!("[E024] Invalid control ep with max_packet_size[11:12] set");
                return false;
            }

            // USB 3
            if ep.max_packet_size != 512 {
                error!("[E025] Invalid control ep with max_packet_size of {}",
                       ep.max_packet_size);
                return false;
            }

            if ep.interval != 0 {
                error!("[E026] Invalid control endpoint with interval set");
                return false;
            }
        }

        usb::ENDPOINT_XFER_ISOC => {

            if (ep.attributes & 0x30) == 0x30 {

                error!("[E027] Invalid iso ep with attributes[4:5] both set");
                return false;

            }

            if usb_v!(dev.bcd_usb, 1) && (ep.max_packet_size & 0x1800) != 0 {

                error!("[E028] Invalid iso ep with max_packet_size[11:12] set");
                return false;

            } else if usb_v!(dev.bcd_usb, 2) {

                // Table 9-14 in USB2 spec

                if (ep.max_packet_size & 0x1800) == 0 &&
                   ((ep.max_packet_size & 0x07ff) < 1 || (ep.max_packet_size & 0x07ff) > 1024) {

                    error!("[E029] Invalid iso ep with max_p_size[11:12] = 00b and inv \
                            max_p_size[0:10]");
                    return false;
                }

                if (ep.max_packet_size & 0x1800) == 0x0800 &&
                   ((ep.max_packet_size & 0x07ff) < 513 || (ep.max_packet_size & 0x07ff) > 1024) {

                    error!("[E030] Invalid iso ep with max_p_size[11:12] = 01b and inv \
                            max_p_size[0:10]");
                    return false;
                }

                if (ep.max_packet_size & 0x1800) == 0x1000 &&
                   ((ep.max_packet_size & 0x07ff) < 683 || (ep.max_packet_size & 0x07ff) > 1024) {

                    error!("[E031] Invalid iso ep with max_p_size[11:12] = 10b and inv \
                            max_p_size[0:10]");
                    return false;
                }

                if (ep.max_packet_size & 0x1800) == 0x1800 {
                    error!("[E032] Invalid iso ep with m_p_size[11:12] both set");
                    return false;
                }

                if ep.interval < 1 || ep.interval > 16 {
                    error!("[E033] Invalid iso endpoint with interval of {}", ep.interval);
                    return false;
                }

                // USB 3
            } else if ep.max_packet_size > 1024 {

                error!("[E034] Invalid iso endpoint with max_packet_size of {}",
                       ep.max_packet_size);
                return false;

            } else if ep.interval < 1 || ep.interval > 16 {

                error!("[E035] Invalid iso endpoint with interval of {}", ep.interval);
                return false;

            }
        }

        usb::ENDPOINT_XFER_BULK => {

            if ep.attributes & 0x3c != 0 {

                error!("[E036] Invalid iso ep with attributes[2:5] set");
                return false;

            }

            if usb_v!(dev.bcd_usb, 1) || usb_v!(dev.bcd_usb, 2) {

                if ep.max_packet_size & 0x1800 != 0 {

                    error!("[E037] Invalid bulk ep with max_packet_size[11:12] set");
                    return false;
                }

                // USB 3
            } else if ep.max_packet_size != 1024 {

                error!("[E038] Invalid bulk ep with max_packet_size of {}",
                       ep.max_packet_size);
                return false;

            } else if ep.interval != 0 {

                error!("[E039] Invalid bulk endpoint with interval set");
                return false;

            }

        }

        usb::ENDPOINT_XFER_INT => {

            if usb_v!(dev.bcd_usb, 1) {

                if (ep.attributes & 0x3c) != 0 {
                    error!("[E040] Invalid interrupt ep with attributes[2:5] set");
                    return false;
                }

                if (ep.max_packet_size & 0x1800) != 0 {
                    error!("[E041] Invalid interrupt endpoint with max_p_size[11:12] set");
                    return false;
                }

                if ep.interval == 0 {
                    error!("[E042] Invalid USB1 interrupt endpoint with 0 interval");
                    return false;
                }

            } else if usb_v!(dev.bcd_usb, 2) {

                if (ep.attributes & 0x30) != 0 {
                    error!("[E043] Invalid interrupt ep with attributes[4:5] set");
                    return false;
                }

                // Table 9-14 in USB2 spec

                if (ep.max_packet_size & 0x1800) == 0 &&
                   ((ep.max_packet_size & 0x07ff) < 1 || (ep.max_packet_size & 0x07ff) > 1024) {

                    error!("[E044] Invalid int ep with max_p_size[11:12] = 00b and inv \
                            max_p_size[0:10]");
                    return false;
                }

                if (ep.max_packet_size & 0x1800) == 0x0800 &&
                   ((ep.max_packet_size & 0x07ff) < 513 || (ep.max_packet_size & 0x07ff) > 1024) {

                    error!("[E045] Invalid int ep with max_p_size[11:12] = 01b and inv \
                            max_p_size[0:10]");
                    return false;
                }

                if (ep.max_packet_size & 0x1800) == 0x1000 &&
                   ((ep.max_packet_size & 0x07ff) < 683 || (ep.max_packet_size & 0x07ff) > 1024) {

                    error!("[E046] Invalid int ep with max_p_size[11:12] = 10b and inv \
                            max_p_size[0:10]");
                    return false;
                }

                if (ep.max_packet_size & 0x1800) == 0x1800 {
                    error!("[E047] Invalid int ep with m_p_size[11:12] both set");
                    return false;
                }

                if ep.interval < 1 || ep.interval > 16 {
                    error!("[E048] Invalid int endpoint with interval of {}", ep.interval);
                    return false;
                }

                // USB 3
            } else if (ep.attributes & 0x0c) != 0 {

                error!("[E049] Invalid int ep with attributes[2:3] set.");
                return false;


            } else if ((ep.attributes & 0x30) >> 1) > 1 {

                error!("[E050] Invalid int ep with attributes[4:5] set to 10b or 11b");
                return false;

            } else if ep.max_packet_size < 1 || ep.max_packet_size > 1024 {

                error!("[E051] Invalid int ep with max packet size of {}", ep.max_packet_size);
                return false;

            } else if ep.interval < 1 || ep.interval > 16 {

                error!("[E052] Invalid int ep with interval of {}", ep.interval);
                return false;

            }

        }

        _ => {
            error!("[E053] Invalid transfer type");
            return false;
        }

    }

    true
}

fn check_ss_ep_comp_fields(ss: &usb::SsEpCompDescriptor, ep: &usb::EndpointDescriptor) -> bool {

    // From Section 9.6.7, pages 386-388 in spec/usb3.pdf

    if ss.max_burst > 15 {
        error!("[E083] Invalid ep companion with max burst {} > 15", ss.max_burst);
        return false;
    }

    // cross reference max burst with ep.max_packet_size
    if ss.max_burst > 0 && ep.max_packet_size != 1024 {
        error!("[E084] Invalid ep and ss comp max burst > 0, but max packet size != 1024");
        return false;
    }

    match ep.attributes & usb::ENDPOINT_XFERTYPE_MASK {

        usb::ENDPOINT_XFER_CONTROL => {

            if ss.max_burst != 0 {
                error!("[E085] Invalid ss ep comp with max burst {} for control ep",
                       ss.max_burst);
                return false;
            }

            if ss.bytes_per_interval != 0 {
                error!("[E086] Invalid ss ep comp with >0 bytes per interval for control ep");
                return false;
            }

            if ss.attributes != 0 {
                error!("[E087] Ss ep comp attributes are being used but they are reserved");
                return false;
            }
        }

        usb::ENDPOINT_XFER_INT => {
            if ss.attributes != 0 {
                error!("[E088] Ss ep comp attributes are being used but they are reserved");
                return false;
            }
        }

        usb::ENDPOINT_XFER_ISOC => {

            if (ss.attributes & 0xfc) != 0 {
                error!("[E089] Ss ep comp attributes[2:7] are being used but they are reserved");
                return false;
            }

            if (ss.attributes & 0x03) > 2 {
                error!("[E090] Mult (ss ep comp's attributes[0:1]) is greater than 2");
                return false;
            }

            if ss.max_burst == 0 && (ss.attributes & 0x03) != 0 {
                error!("[E091] Invalid ss ep comp with max burst 0 but attributes[0:1] != 0");
                return false;
            }

        }

        usb::ENDPOINT_XFER_BULK => {

            if (ss.attributes & 0xe0) != 0 {
                error!("[E092] Invalid ss ep comp with attributes[5:7] being used");
                return false;
            }

            if (ss.attributes & 0x1f) > 16 {
                error!("[E093] Max streams (ss ep comp's attributes[0:4]) is greater than 16");
                return false;
            }

            if ss.bytes_per_interval != 0 {
                error!("[E094] Invalid ss ep comp with >0 bytes per interval for bulk endpoint");
                return false;
            }
        }

        _ => {
            error!("[095] Invalid transfer type for endpoint ({:x})",
                   ep.attributes & usb::ENDPOINT_XFERTYPE_MASK);
            return false;
        }
    }

    true
}


fn check_device_qualifier_fields(desc: &usb::DeviceQualifierDescriptor) -> bool {

    // From Section 9.6.2, page 264 in spec/usb2.pdf

    // must be USB2
    if !(usb_v!(desc.bcd_usb, 2) && check_bcd(desc.bcd_usb)) {
        error!("[E105] Invalid bcdUSB in device qualifier");
        return false;
    }

    if !check_device_csp(desc.device_class, desc.device_subclass, desc.device_protocol) {
        return false;
    }

    match desc.max_packet_size0 {

        x if x == 8 || x == 16 || x == 32 || x == 64 => {}
        _ => {
            error!("[E106] Invalid device max packet size0 ({})", desc.max_packet_size0);
            return false;
        }
    }


    if desc.reserved != 0 {
        error!("[E107] Use of reserved field");
        return false;
    }

    true
}

#[allow(unused_variables)]
fn check_pipe_usage_fields(pipe: &usb::PipeUsageDescriptor) -> bool {
    // TODO: the needed info is probably in one of the following standards
    // Unfortunately, none of them are freely available... so yeah..

    // ANSI INCITS 471-2010
    // ISO/IEC 14776-251
    // ISO/IEC 14776-252
    // T10/BSR INCITS 520
    true
}


fn check_otg_fields(otg: &usb::OtgDescriptor) -> bool {

    // From spec/otg.pdf

    if (otg.attributes & 0xfc) != 0 {
        error!("[E129] Invalid otg descriptor uses reserved bits");
        return false;
    }

    true
}

fn check_cap_ext_fields(ext: &usb::ExtCapDescriptor, bcd_usb: u16) -> bool {

    // From Section 9.6.2.1, pages 374-375 in spec/usb3.pdf

    if (ext.attributes & 0xfffffffd) != 0 {
        error!("[E145] USB2 extension description using reserved bits");
        return false;
    }

    if usb_v!(bcd_usb, 3) && (ext.attributes & 0x00000002) == 0 {
        error!("[E146] USB3 device failed to set LPM bit in usb ext descriptor");
        return false;
    }

    true
}

fn check_cap_ss_fields(ss: &usb::SsCapDescriptor) -> bool {

    // From Section 9.6.2.2, pages 375-376 in spec/usb3.pdf

    if (ss.attributes & 0xfd) != 0 {
        error!("[E147] SS cap desc using resrved bits");
        return false;
    }

    if (ss.speed_supported & 0xfff0) != 0 {
        error!("[E148] SS cap desc using reserved speed bits");
        return false;
    }

    if (ss.speed_supported & (1 << ss.functionality_support)) == 0 {
        error!("[E149] Inconsistent SS cap descriptor speed and func");
        return false;
    }

    if ss.u1_dev_exit_lat > 0x0a {
        error!("[E150] SS cap descriptor using reserved u1 latency value");
        return false;
    }

    if ss.u2_dev_exit_lat > 0x07ff {
        error!("[E151] SS cap descriptor using reserved u2 latency value");
        return false;
    }

    true
}

fn check_cap_container_fields(con: &usb::ContainerIdDescriptor) -> bool {

    // From Section 9.6.2.3, page 377 in spec/usb3.pdf

    if con.reserved != 0 {
        error!("[E152] Container cap descriptor using reserved bit");
        return false;
    }

    true
}

impl ControlCheck {
    pub fn new(third_party_folder: &str) -> ControlCheck {
        ControlCheck {
            vdev: RwLock::new(VirtualDevice::new()),
            hid_checks: RwLock::new(hid::HidControlCheck::new()),
            bbb_checks: RwLock::new(bbb::BBBControlCheck::new()),
            third_party: RwLock::new(third_party::Patcher::new(third_party_folder)),
        }
    }


    fn check_get_config(&self, data: &[u8]) -> bool {

        if data.len() != 1 {
            error!("[E054] Incorrect packet size for get config");
            return false;
        }


        let vdev = self.vdev.read().unwrap();

        if !vdev.configs.contains_key(&data[0]) {
            error!("[E055] Incorrect configuration value returned");
            return false;
        }

        true
    }

    fn update_config(&self, h: &usbr::ControlPacketHeader) {

        let vdev: &mut VirtualDevice = &mut self.vdev.write().unwrap();

        for (key, val) in &vdev.configs {
            if val.desc.configuration_value == (h.value as u8) {
                info!("Updating the value of the selected configuration to {}", *key);
                vdev.chosen_conf = Some(*key);
            }
        }
    }

    fn check_get_interface(&self, h: &usbr::ControlPacketHeader, data: &[u8]) -> bool {

        if data.len() > 0 {
            error!("[E056] Incorrect data size for get interface");
            return false;
        }

        let vdev = self.vdev.read().unwrap();

        if vdev.chosen_conf.is_none() {
            error!("[E078] vdev.chosen_conf not yet set");
            return false;
        }

        let interfaces = &vdev.configs.get(&vdev.chosen_conf.unwrap()).unwrap().interfaces;

        if !interfaces.contains_key(&(h.index as u8)) {
            error!("[E057] Requested interface number is invalid");
            return false;
        }

        if !interfaces.get(&(h.index as u8)).unwrap().contains_key(&data[0]) {
            error!("[E058] Invalid alternate interface value");
            return false;
        }

        true
    }

    fn update_interface(&self, h: &usbr::ControlPacketHeader) {

        let vdev = self.vdev.read().unwrap();

        if vdev.chosen_interfaces.contains_key(&(h.index as u8)) {

            // No need to worry about TOCTOU attack here because there is only 1 thread that
            // can update this function. The other thread processes data in the other direction
            // so it won't contain the descriptor, meaning data.len() will be 0.

            let vdev: &mut VirtualDevice = &mut self.vdev.upgrade(vdev).unwrap();
            vdev.chosen_interfaces.insert(h.index as u8, h.value as u8);
        }
    }

    fn check_device_descriptor(&self, h: &usbr::ControlPacketHeader, data: &[u8]) -> bool {

        if (data.len() != usb::DEVICE_DESC_SIZE + usb::HEADER_SIZE && data.len() != h.length as usize) ||
           data.len() < usb::HEADER_SIZE {
            error!("[E059] Invalid device descriptor length: {}. Expected Min({}, {})",
                   data.len(),
                   usb::DEVICE_DESC_SIZE + usb::HEADER_SIZE,
                   h.length);
            return false;
        }

        let header: &usb::DescriptorHeader = unsafe { parse_descriptor!(0, data) };

        if header.length as usize != usb::DEVICE_DESC_SIZE + usb::HEADER_SIZE ||
           header.descriptor_type != usb::DT_DEVICE {
            error!("[E060] Incorrect length ({}) or descriptor type (0x{:x}) for device desc",
                   header.length,
                   header.descriptor_type);
            return false;
        }

        if !check_device_fields(&data[2..]) {
            return false;
        }

        // If this is the first time we've seen this descriptor, use it to construct our virtual
        // device model.

        let vdev = self.vdev.read().unwrap();

        if vdev.desc.is_none() && data.len() == usb::DEVICE_DESC_SIZE + usb::HEADER_SIZE {

            let vdev: &mut VirtualDevice = &mut self.vdev.upgrade(vdev).unwrap();

            // No need to worry about TOCTOU attack here because there is only 1 thread that
            // can update this function. The other thread processes data in the other direction
            // so it won't contain the descriptor, meaning data.len() will be 0.

            vdev.desc = Some(parse_descriptor!(usb::DT_DEVICE, &data[2..]));
        }

        true
    }

    fn check_config_descriptor(&self, h: &usbr::ControlPacketHeader, data: &[u8]) -> bool {

        // This descriptor is tricky because it is closely tied to interface and
        // endpoint descriptors too.
        //
        // (1) Blue machine first asks the device for the configuration descriptor and asks the
        // device
        // to respond with at most h.length bytes (typically usb::DT_CONFIG_SIZE + 2).
        // The device then responds with the configuration descriptor that states the total
        // size of the descriptor (including the interface and endpoint descriptors too).
        //
        // (2) The host then send another request with h.length updated.
        // The device then responds with the config descriptor + endpoints and interfaces.
        //
        // Checks that need to be carried out are:
        // (1) Is the descriptor length (for case 1 and, where applicable, each of the descs in
        // case 2) correct?
        // (2) Are the field in the config descriptor valid?
        // (3) Are the interface descriptor and endpoint descriptors valid and included in the
        // right order (after every iface descriptor, endpoints should immediately follow)?
        // (4) After processing all ifaces, are they complete and their ids are consecutive (0
        // to n)?
        //

        if data.len() < usb::CONFIG_DESC_SIZE + usb::HEADER_SIZE {
            error!("[E061] Invalid payload length ({})", data.len());
            return false;
        }

        let header: &usb::DescriptorHeader = unsafe { parse_descriptor!(0, data) };

        if header.length as usize != usb::CONFIG_DESC_SIZE + usb::HEADER_SIZE ||
           header.descriptor_type != usb::DT_CONFIG {

            error!("[E062] Incorrect length ({}) or descriptor type (0x{:x}) for config desc",
                   header.length,
                   header.descriptor_type);
            return false;
        }

        // This is safe because of data.len() check above
        let config = parse_descriptor!(usb::DT_CONFIG, &data[2..]);
        let index: u8 = h.value as u8; // index of USB function of interest

        // case (1)
        if h.length as usize == usb::CONFIG_DESC_SIZE + usb::HEADER_SIZE {

            if !(data.len() == usb::CONFIG_DESC_SIZE + usb::HEADER_SIZE) {
                error!("[E063] Invalid size of config descriptor. Expected {}, got {}",
                       usb::CONFIG_DESC_SIZE + usb::HEADER_SIZE,
                       data.len());
                return false;
            }


            let vdev = self.vdev.read().unwrap();

            if vdev.desc.is_none() {
                error!("[E165] vdev.desc not yet initialized");
                return false;
            }


            // check fields themselves are OK
            if !check_config_fields(&config, &vdev.desc.unwrap()) {
                return false;
            }

            // Check third party compliance constraints
            {
                let mut third_party = self.third_party.write().unwrap();
                if !third_party.check_config_fields(&config, &vdev.desc.unwrap()) {
                    return false;
                }
            }

            if !vdev.configs.contains_key(&index) {

                // Create new ConfigNode
                let conf_node = ConfigNode { desc: config, interfaces: HashMap::new() };

                // Upgrade lock
                let vdev: &mut VirtualDevice = &mut self.vdev.upgrade(vdev).unwrap();

                // Create descriptor node
                vdev.configs.insert(index, conf_node);
                vdev.chosen_conf = Some(index);
            }

        } else {
            // case (2)

            // check total size is consistent with request
            if !(data.len() == h.length as usize) {
                error!("[E064] Invalid size of config desc. Expected {}, got {}",
                       h.length,
                       data.len());
                return false;
            }

            // check total size is consistent with total_length (in descriptor header)
            if data.len() != config.total_length as usize {
                error!("[E065] total_length ({}) is higher than actual size ({})",
                       config.total_length,
                       data.len());
                return false;
            }

            {
                let vdev = self.vdev.read().unwrap();

                if vdev.desc.is_none() {
                    error!("[E167] vdev.desc not yet initialized");
                    return false;
                }

                // check config descriptor itself is correct
                if !check_config_fields(&config, &vdev.desc.unwrap()) {
                    return false;
                }

            }


            let num_interfaces: u8 = config.num_interfaces;

            // add config to our model of the device (if not already there).
            {
                let vdev = self.vdev.read().unwrap();

                if !vdev.configs.contains_key(&index) {

                    // Create new ConfigNode
                    let conf_node = ConfigNode { desc: config, interfaces: HashMap::new() };

                    // Upgrade lock
                    let vdev: &mut VirtualDevice = &mut self.vdev.upgrade(vdev).unwrap();

                    // Create descriptor node
                    vdev.configs.insert(index, conf_node);
                    vdev.chosen_conf = Some(index);
                }
            }

            // Iterate over interface or interface assoc. descriptors and check them.
            // This requires indentifying the type of descriptor first.

            let mut off: usize = usb::CONFIG_DESC_SIZE + usb::HEADER_SIZE;

            while off < data.len() {

                if data.len() < off + usb::HEADER_SIZE {
                    error!("[E066] Data not enough for descriptor header");
                    return false;
                }

                let i_hdr: &usb::DescriptorHeader = unsafe { parse_descriptor!(0, &data[off..]) };
                off += usb::HEADER_SIZE;

                if i_hdr.length as usize == usb::INTERFACE_DESC_SIZE + usb::HEADER_SIZE &&
                   i_hdr.descriptor_type == usb::DT_INTERFACE {

                    // note that this increments off after interface desc and all ep descs
                    // have been processed.
                    if !self.check_interface_desc(index, data, &mut off) {
                        return false;
                    }

                } else if i_hdr.length as usize == usb::INTERFACE_ASSOC_DESC_SIZE &&
                   i_hdr.descriptor_type == usb::DT_INTERFACE_ASSOCIATION {

                    // note that this increments off after interface desc and all ep descs
                    // have been processed.
                    if !self.check_interface_assoc_desc(data, &mut off) {
                        return false;
                    }

                } else {

                    // unknown type
                    error!("[E067] Unknown descriptor type following config desc: 0x{:x}",
                           i_hdr.descriptor_type);
                    return false;
                }
            }

            let vdev = self.vdev.read().unwrap();

            if num_interfaces != vdev.chosen_interfaces.len() as u8 {

                error!("[E068] Number of interfaces is invalid. Promised {}, processed {}",
                       num_interfaces,
                       vdev.chosen_interfaces.len());
                return false;
            }


            // TODO: Check that all interfaces and endpoints have unique and consecutive ids
            // (except for alternatives)
        }

        true
    }

    fn check_interface_desc(&self, index: u8, data: &[u8], off: &mut usize) -> bool {

        if data.len() < *off + usb::INTERFACE_DESC_SIZE {
            error!("[E069] Invalid data length than interface descriptor size");
            return false;
        }

        let iface = parse_descriptor!(usb::DT_INTERFACE, &data[*off..]);
        *off += usb::INTERFACE_DESC_SIZE;

        let vdev = self.vdev.read().unwrap();

        if vdev.desc.is_none() {
            error!("[E079] vdev.desc not yet initialized");
            return false;
        }

        if !check_interface_fields(&iface, &vdev.desc.unwrap()) {
            return false;
        }

        {
            let mut third_party = self.third_party.write().unwrap();
            if !third_party.check_iface_fields(&iface, &vdev.desc.unwrap()) {
                return false;
            }
        }

        let i_num: u8 = iface.interface_number;
        let alt_setting: u8 = iface.alternate_setting;

        // case (1) no entries for this interface number (no alternates either)
        let vdev = if !has_interface!(vdev, index, i_num) {

            // Upgrade lock (note that we need the actual object because we plan
            // to downgrade it).
            let mut vdev = self.vdev.upgrade(vdev).unwrap();

            // Insert entries
            {
                let config: &mut ConfigNode = vdev.configs.get_mut(&index).unwrap();
                config.interfaces.insert(i_num, HashMap::new());

                let alts = &mut config.interfaces.get_mut(&i_num).unwrap();
                alts.insert(alt_setting, InterfaceNode { desc: iface, endpoints: HashMap::new() });
            }

            vdev.chosen_interfaces.insert(i_num, alt_setting);

            self.vdev.downgrade(vdev).unwrap()

            // case (2) interface num is in map, but not this alternate setting
        } else if !has_alternate!(vdev, index, i_num, alt_setting) {

            // Upgrade lock
            let mut vdev = self.vdev.upgrade(vdev).unwrap();

            // Get appropritate hash map and insert entry
            {
                let alts = &mut vdev.configs
                    .get_mut(&index)
                    .unwrap()
                    .interfaces
                    .get_mut(&i_num)
                    .unwrap();

                alts.insert(alt_setting, InterfaceNode { desc: iface, endpoints: HashMap::new() });
            }

            self.vdev.downgrade(vdev).unwrap()

            // (common case) case (3) no need to do anything
        } else {
            vdev
        };


        // check endpoints: (1) ensure we have enough data for all endpoints.
        //                  (2) process and check each endpoint


        // Many classes have different types of hierarchies below interfaces. We need a special
        // check for each class that has antyhing beyond normal endpoint (e.g., HID).
        // The default applies to most classes though.

        match iface.interface_class {

            x if x == usb::CLASS_AUDIO || x == usb::CLASS_AUDIO_VIDEO => {

                for i in 0..iface.num_endpoints {

                    if data.len() < *off + usb::HEADER_SIZE {
                        error!("[E070] Not enough payload for endpoint {} header", i);
                        return false;
                    }

                    let header = unsafe { parse_descriptor!(0, &data[*off..]) };
                    *off += usb::HEADER_SIZE;

                    if header.length as usize != usb::AUDIO_EP_DESC_SIZE + usb::HEADER_SIZE ||
                       header.descriptor_type != usb::DT_ENDPOINT {

                        error!("[E071] Invalid audio endpoint {} length {} or type 0x{:x}",
                               i,
                               header.length,
                               header.descriptor_type);
                        return false;
                    }

                    // TODO: implement check for audio
                    unimplemented!();

                }

                return true;
            }

            usb::CLASS_HID => {

                // HID class has an additional descriptor before the standard endpoint descriptor

                if data.len() < *off + usb::HEADER_SIZE {
                    error!("[E072] Not enough payload for endpoint header");
                    return false;
                }

                let header = unsafe { parse_descriptor!(0, &data[*off..]) };
                *off += usb::HEADER_SIZE;

                if (header.length as usize) < usb::hid::DESC_MIN_SIZE + usb::HEADER_SIZE ||
                   header.descriptor_type != usb::hid::DT_HID {

                    error!("[E073] Invalid hid descriptor type 0x{:x} or length {}",
                           header.descriptor_type,
                           header.length);
                    return false;
                }

                // Get hid checks
                let mut hid_check = self.hid_checks.write().unwrap();

                // this call updates off
                if !hid_check.check_hid_desc(header, data, off, i_num, alt_setting) {
                    return false;
                }
            }

            _ => {}
        }


        if (usb_v!(vdev.desc.unwrap().bcd_usb, 1) || usb_v!(vdev.desc.unwrap().bcd_usb, 2)) &&
           (iface.num_endpoints as usize) * (usb::ENDPOINT_DESC_SIZE + usb::HEADER_SIZE) + *off > data.len() {

            error!("[E074] Invalid number of endpoints or payload {} is not sufficient {}",
                   (iface.num_endpoints as usize) * (usb::ENDPOINT_DESC_SIZE + usb::HEADER_SIZE) + *off,
                   data.len());
            return false;

        } else if usb_v!(vdev.desc.unwrap().bcd_usb, 3) &&
           (iface.num_endpoints as usize) *
           (usb::ENDPOINT_DESC_SIZE + usb::SS_EP_DESC_SIZE + 2 * usb::HEADER_SIZE) + *off > data.len() {
            error!("[E075] Invalid number of SS endpoints. Payload is not sufficient");
            return false;
        }

        // drop so it can be used in endpoint desc
        drop(vdev);

        // process each endpoint
        for i in 0..iface.num_endpoints {

            if data.len() < *off + usb::HEADER_SIZE {
                error!("[E076] Not enough payload for endpoint {} header", i);
                return false;
            }

            let header = unsafe { parse_descriptor!(0, &data[*off..]) };
            *off += usb::HEADER_SIZE;

            if header.length as usize != usb::ENDPOINT_DESC_SIZE + usb::HEADER_SIZE ||
               header.descriptor_type != usb::DT_ENDPOINT {
                error!("[E077] Invalid endpoint length {} or type {:x}",
                       header.length,
                       header.descriptor_type);
            }

            // this call updates the value of "off"
            if !self.check_endpoint_desc(data, &iface, index, off) {
                return false;
            }
        }

        true
    }

    fn check_endpoint_desc(&self,
                           data: &[u8],
                           iface: &usb::InterfaceDescriptor,
                           index: u8,
                           off: &mut usize)
                           -> bool {

        // (1) check endpoint fields
        // (2) if USB3 process SS endpoint companion
        // (3) add endpoint to our model (if needed)

        if data.len() < *off + usb::ENDPOINT_DESC_SIZE {
            error!("[E080] Insufficient payload for endpoint ({} - {})", data.len(), *off);
            return false;
        }

        let ep = parse_descriptor!(usb::DT_ENDPOINT, &data[*off..]);
        *off += usb::ENDPOINT_DESC_SIZE;

        let vdev = self.vdev.read().unwrap();

        if vdev.desc.is_none() {
            error!("[E098] device descriptor has not been initialized yet");
            return false;
        }

        if !check_endpoint_fields(&ep, &vdev.desc.unwrap()) {
            return false;
        }

        {
            let mut third_party = self.third_party.write().unwrap();
            if !third_party.check_endpoint_fields(&ep, &vdev.desc.unwrap()) {
                return false;
            }
        }

        let mut ss_ep: Option<usb::SsEpCompDescriptor> = None;
        let mut pipe: Option<usb::PipeUsageDescriptor> = None;

        // If USB3 process SuperSpeed endpoint companion
        if usb_v!(vdev.desc.unwrap().bcd_usb, 3) {

            if data.len() < *off + usb::SS_EP_DESC_SIZE + usb::HEADER_SIZE {
                error!("[E081] Insufficient payload for SS endpoint companion ({} - {})",
                       data.len(),
                       *off);
                return false;
            }

            let header = unsafe { parse_descriptor!(0, &data[*off..]) };
            *off += usb::HEADER_SIZE;

            if header.length as usize != usb::SS_EP_DESC_SIZE + usb::HEADER_SIZE ||
               header.descriptor_type != usb::DT_SS_ENDPOINT_COMP {
                error!("[E082] Not a USB3 SS ep companion ({:x}) or wrong size ({})",
                       header.descriptor_type,
                       header.length);
                return false;
            }

            ss_ep = Some(parse_descriptor!(usb::DT_SS_ENDPOINT_COMP, &data[*off..]));
            *off += usb::SS_EP_DESC_SIZE;

            // Check SS ep fields
            if !check_ss_ep_comp_fields(&ss_ep.unwrap(), &ep) {
                return false;
            }

            // If this is a USB 3 UAS (usb attached SCSI) interface process the additional pipe
            // usage descriptor
            if iface.interface_class == usb::CLASS_MASS_STORAGE && iface.interface_subclass == 0x06 &&
               iface.interface_protocol == 0x62 {

                if data.len() < *off + usb::PIPE_DESC_SIZE + usb::HEADER_SIZE {
                    error!("[E096] Not enough payload for pipe usage descriptor");
                    return false;
                }

                let pipe_h = unsafe { parse_descriptor!(0, &data[*off..]) };
                *off += usb::HEADER_SIZE;

                if pipe_h.length as usize != usb::PIPE_DESC_SIZE + usb::HEADER_SIZE ||
                   pipe_h.descriptor_type != usb::DT_PIPE_USAGE {
                    error!("[E097] Not a USB3 UAS pipe usage descritor ({:x}) or wrong size ({})",
                           pipe_h.descriptor_type,
                           pipe_h.length);
                    return false;
                }

                pipe = Some(parse_descriptor!(usb::DT_PIPE_USAGE, &data[*off..]));
                *off += usb::PIPE_DESC_SIZE;

                if !check_pipe_usage_fields(&pipe.unwrap()) {
                    return false;
                }
            }
        }

        let vdev: &mut VirtualDevice = &mut self.vdev.upgrade(vdev).unwrap();

        let conf_node = vdev.configs.get_mut(&index).unwrap();
        let iface_node_map = conf_node.interfaces.get_mut(&iface.interface_number).unwrap();
        let iface_node = iface_node_map.get_mut(&iface.alternate_setting).unwrap();

        iface_node.endpoints
            .entry(ep.endpoint_address & 0x8f)
            .or_insert(EndpointNode { desc: ep, ss_desc: ss_ep, pipe_desc: pipe });

        true
    }

    fn check_interface_assoc_desc(&self, data: &[u8], off: &mut usize) -> bool {

        if data.len() < *off + usb::INTERFACE_ASSOC_DESC_SIZE + usb::HEADER_SIZE {
            error!("[E100] Not enough payload for interface assoc descriptor");
            return false;
        }

        let header = unsafe { parse_descriptor!(0, &data[*off..]) };
        *off += usb::HEADER_SIZE;

        if header.length as usize != usb::INTERFACE_ASSOC_DESC_SIZE + usb::HEADER_SIZE ||
           header.descriptor_type != usb::DT_INTERFACE_ASSOCIATION {
            error!("[E101] Incorrect length ({}) or type for iface assoc desc ({:x})",
                   header.length,
                   header.descriptor_type);
            return false;
        }

        let iface = unsafe { parse_descriptor!(usb::DT_INTERFACE_ASSOCIATION, &data[*off..]) };
        *off += usb::INTERFACE_ASSOC_DESC_SIZE;

        if !check_interface_assoc_fields(iface) {
            return false;
        }

        true
    }


    fn check_string_descriptor(&self, h: &usbr::ControlPacketHeader, data: &[u8]) -> bool {

        let index: u8 = h.value as u8;

        if data.len() < usb::HEADER_SIZE {
            error!("[E102] Not enough payload for string descriptor");
            return false;
        }

        let header = unsafe { parse_descriptor!(0, data) };

        if header.descriptor_type != usb::DT_STRING {
            error!("[E103] Invalid descriptor type ({:x}) for string",
                   header.descriptor_type);
            return false;
        }

        // header.length already accounts for usb::HEADER_SIZE
        if data.len() != header.length as usize {

            if data.len() != h.length as usize {
                error!("[E104] Not enough payload ({}) for string descriptor ({}). requested ({})",
                       data.len(),
                       header.length,
                       h.length);
                return false;

            } else if data.len() != usb::STRING_DESC_SIZE_MIN {

                error!("[E164] payload matches host request but is not min size... weird");
                return false;

            } else {

                // This is just a probe for length
                return true;
            }
        }

        let str_node = StringNode { desc: data[2..].to_vec() };

        if !check_string_fields(&str_node.desc, index) {
            return false;
        }

        let vdev = self.vdev.read().unwrap();

        if !vdev.strings.contains_key(&index) {

            let vdev: &mut VirtualDevice = &mut self.vdev.upgrade(vdev).unwrap();
            vdev.strings.insert(index, str_node);

        }

        true
    }

    fn check_device_qual_descriptor(&self, data: &[u8]) -> bool {

        // Reserved in USB3
        let vdev = self.vdev.read().unwrap();

        if vdev.desc.is_none() {
            error!("[E108] Device qualifier descriptor received before device descriptor");
            return false;
        }

        if usb_v!(vdev.desc.unwrap().bcd_usb, 3) {
            error!("[E109] Device qualifier descriptor is reserved in USB3");
            return false;
        }

        if data.len() < usb::DEVICE_QUALIFIER_SIZE + usb::HEADER_SIZE {
            error!("[E110] Not enough payload for device qualifier descriptor");
            return false;
        }

        let header = unsafe { parse_descriptor!(0, data) };

        if header.length as usize != usb::DEVICE_QUALIFIER_SIZE + usb::HEADER_SIZE ||
           header.descriptor_type != usb::DT_DEVICE_QUALIFIER {
            error!("[E111] Incorrect length ({}) or descriptor type ({})",
                   header.length,
                   header.descriptor_type);
            return false;
        }

        let desc = parse_descriptor!(usb::DT_DEVICE_QUALIFIER, &data[2..]);

        if !check_device_qualifier_fields(&desc) {
            return false;
        }

        true
    }

    fn check_other_speed_descriptor(&self, data: &[u8]) -> bool {

        // Reserved in USB3
        let vdev = self.vdev.read().unwrap();

        if vdev.desc.is_none() {
            error!("[E112] Other speed descriptor received before device descriptor");
            return false;
        }

        if usb_v!(vdev.desc.unwrap().bcd_usb, 3) {
            error!("[E113] Other speed descriptor is reserved in USB3");
            return false;
        }

        if data.len() < usb::OTHER_SPEED_SIZE + usb::HEADER_SIZE {
            error!("[E114] Not enough payload for other speed descriptor");
            return false;
        }


        let header = unsafe { parse_descriptor!(0, data) };

        if header.length as usize != usb::OTHER_SPEED_SIZE + usb::HEADER_SIZE ||
           header.descriptor_type != usb::DT_OTHER_SPEED_CONFIG {
            error!("[E115] Incorrect length ({}) or descriptor type ({})",
                   header.length,
                   header.descriptor_type);
            return false;
        }

        // This is OK because OtherSpeedDescriptor is a type alias of ConfigDescriptor
        let desc = parse_descriptor!(usb::DT_CONFIG, &data[2..]);

        if !check_other_speed_fields(&desc) {
            return false;
        }

        true
    }

    fn check_interface_power_desc(&self, data: &[u8]) -> bool {

        if data.len() < usb::HEADER_SIZE {
            error!("[E125] Not enough payload for header");
            return false;
        }

        let header = unsafe { parse_descriptor!(0, data) };

        if header.length as usize != usb::INTERFACE_POWER_DESC_SIZE + usb::HEADER_SIZE ||
           header.descriptor_type != usb::DT_INTERFACE_POWER {
            error!("[E126] Incorrect length ({}) or descriptor type ({})",
                   header.length,
                   header.descriptor_type);
            return false;
        }

        true
    }


    fn check_otg_desc(&self, data: &[u8]) -> bool {
        // From spec/otg.pdf

        if data.len() < usb::HEADER_SIZE {
            error!("[E127] Not enough payload for header");
            return false;
        }

        let header = unsafe { parse_descriptor!(0, data) };

        if header.length as usize != usb::OTG_DESC_SIZE + usb::HEADER_SIZE ||
           header.descriptor_type != usb::DT_OTG {
            error!("[E128] Incorrect length ({}) or descriptor type ({})",
                   header.length,
                   header.descriptor_type);
            return false;
        }

        let otg = unsafe { parse_descriptor!(usb::DT_OTG, data[2..]) };

        if !check_otg_fields(otg) {
            return false;
        }

        true
    }

    fn check_bos_desc(&self, data: &[u8]) -> bool {

        if data.len() < usb::HEADER_SIZE {
            error!("[E131] Not enough payload for header");
            return false;
        }

        let header = unsafe { parse_descriptor!(0, data) };

        if header.length as usize != usb::BOS_DESC_SIZE + usb::HEADER_SIZE ||
           header.descriptor_type != usb::DT_BOS {
            error!("[E132] Incorrect length ({}) or descriptor type ({})",
                   header.length,
                   header.descriptor_type);
            return false;
        }


        // It only includes the BOS header to communicate total length to the OS
        // The OS will likely re-issue a request for the BOS with a larger buffer at a later time.
        if data.len() == usb::BOS_DESC_SIZE + usb::HEADER_SIZE {
            return true;
        }

        let bos = parse_descriptor!(usb::DT_BOS, &data[2..]);

        if data.len() < bos.total_length as usize {
            error!("[E133] Not enough payload for BOS descriptor children");
            return false;
        }

        let mut off: usize = usb::BOS_DESC_SIZE + usb::HEADER_SIZE;

        for _ in 0..bos.num_device_caps {

            if data.len() < off + usb::HEADER_SIZE {
                error!("[E134] Not enough paylaod for the th header");
                return false;
            }

            let t_header = unsafe { parse_descriptor!(0, &data[off..]) };
            off += usb::HEADER_SIZE;

            if t_header.descriptor_type != usb::DT_DEVICE_CAPABILITY ||
               data.len() < off + usb::DEV_CAP_HEADER_SIZE {
                error!("[E135] Incorrect descriptor type for BOS child or insufficient payload");
                return false;
            }

            let c_header = unsafe { parse_descriptor!(usb::DT_DEVICE_CAPABILITY, &data[off..]) };
            off += usb::DEV_CAP_HEADER_SIZE;

            match c_header.dev_capability_type {

                usb::CAP_TYPE_WIRELESS => {

                    if t_header.length as usize !=
                       usb::HEADER_SIZE + usb::DEV_CAP_HEADER_SIZE + usb::CAP_TYPE_WIRELESS_SIZE {
                        error!("[E137] Invalid length for wireless cap descriptor");
                        return false;
                    }

                    if !self.check_cap_wireless_desc(data, &mut off) {
                        return false;
                    }
                }

                usb::CAP_TYPE_EXT => {

                    if t_header.length as usize !=
                       usb::HEADER_SIZE + usb::DEV_CAP_HEADER_SIZE + usb::CAP_TYPE_EXT_SIZE {

                        error!("[E141] Invalid length for wireless cap descriptor");
                        return false;
                    }

                    if !self.check_cap_ext_desc(data, &mut off) {
                        return false;
                    }
                }

                usb::CAP_TYPE_SS => {

                    if t_header.length as usize !=
                       usb::HEADER_SIZE + usb::DEV_CAP_HEADER_SIZE + usb::CAP_TYPE_SS_SIZE {

                        error!("[E142] Invalid length for wireless cap descriptor");
                        return false;
                    }

                    if !self.check_cap_ss_desc(data, &mut off) {
                        return false;
                    }

                }

                usb::CAP_TYPE_CONTAINER_ID => {

                    if t_header.length as usize !=
                       usb::HEADER_SIZE + usb::DEV_CAP_HEADER_SIZE + usb::CAP_TYPE_CONTAINER_SIZE {

                        error!("[E143] Invalid length for wireless cap descriptor");
                        return false;
                    }

                    if !self.check_cap_container_desc(data, &mut off) {
                        return false;
                    }
                }

                _ => {
                    error!("[E136] Invalid device capability type 0x{:x}",
                           c_header.dev_capability_type);
                    return false;
                }
            }
        }

        true

    }


    fn check_cap_wireless_desc(&self, data: &[u8], off: &mut usize) -> bool {

        if data.len() < *off + usb::CAP_TYPE_WIRELESS_SIZE {
            error!("[E138] Not enough payload for wireless cap type descriptor");
            return false;
        }

        // TODO let wireless = unsafe { parse_descriptor!(usb::CAP_TYPE_WIRELESS, data[*off..]) };
        *off += usb::CAP_TYPE_WIRELESS_SIZE;

        // TODO if !check_wireless_cap_fields(&wireless) {
        //  return false;
        // }

        panic!("Not implemented check for wireless fields");
    }


    fn check_cap_ext_desc(&self, data: &[u8], off: &mut usize) -> bool {

        if data.len() < *off + usb::CAP_TYPE_EXT_SIZE {
            error!("[E139] Not enough payload for ext cap descriptor");
            return false;
        }

        let ext = parse_descriptor!(usb::CAP_TYPE_EXT, &data[*off..]);
        *off += usb::CAP_TYPE_EXT_SIZE;

        let vdev = self.vdev.read().unwrap();

        if vdev.desc.is_none() {
            error!("[E145] requesting cap ext descriptor without initializing dev");
            return false;
        }

        if !check_cap_ext_fields(&ext, vdev.desc.unwrap().bcd_usb) {
            return false;
        }

        true
    }


    fn check_cap_ss_desc(&self, data: &[u8], off: &mut usize) -> bool {

        if data.len() < *off + usb::CAP_TYPE_SS_SIZE {
            error!("[E140] Not enough payload for ss cap type descriptor");
            return false;
        }

        let ss = parse_descriptor!(usb::CAP_TYPE_SS, &data[*off..]);
        *off += usb::CAP_TYPE_SS_SIZE;

        if !check_cap_ss_fields(&ss) {
            return false;
        }

        true
    }

    fn check_cap_container_desc(&self, data: &[u8], off: &mut usize) -> bool {

        if data.len() < *off + usb::CAP_TYPE_CONTAINER_SIZE {
            error!("[E144] Not enough payload for container id cap type descriptor");
            return false;
        }

        let con = parse_descriptor!(usb::CAP_TYPE_CONTAINER_ID, &data[*off..]);
        *off += usb::CAP_TYPE_CONTAINER_SIZE;

        if !check_cap_container_fields(&con) {
            return false;
        }

        true
    }

    fn check_dbg_desc(&self, data: &[u8]) -> bool {

        // From spec/debug.pdf

        if data.len() < usb::HEADER_SIZE {
            error!("[E158] Not enough payload for header descriptor");
            return false;
        }

        let header = unsafe { parse_descriptor!(0, &data[..2]) };

        if header.length as usize != usb::DEBUG_DESC_SIZE + usb::HEADER_SIZE ||
           header.descriptor_type != usb::DT_DEBUG {

            error!("[E159] Incorrect length {} or descriptor type 0x{:x} for debug desc",
                   header.length,
                   header.descriptor_type);
            return false;
        }

        if data.len() != usb::HEADER_SIZE + usb::DEBUG_DESC_SIZE {
            error!("[E160] Incorrect payload for debug descriptor");
            return false;
        }

        let desc = unsafe { parse_descriptor!(usb::DT_DEBUG, &data[2..]) };

        let vdev = self.vdev.read().unwrap();

        if vdev.chosen_conf.is_none() || !vdev.configs.contains_key(&vdev.chosen_conf.unwrap()) {
            error!("[E161] Device is nhot set up yet to issue a debug request");
            return false;
        }

        let mut found_ep_in: bool = false;
        let mut found_ep_out: bool = false;


        for ifaces in vdev.configs.get(&vdev.chosen_conf.unwrap()).unwrap().interfaces.values() {
            for iface in ifaces.values() {
                for ep_key in iface.endpoints.keys() {

                    if !found_ep_in && (ep_key & 0x0f) == desc.debug_in_endpoint &&
                       (ep_key & 0x80) == usb::DIR_IN {
                        found_ep_in = true;
                    } else if !found_ep_out && (ep_key & 0x0f) == desc.debug_out_endpoint &&
                       (ep_key & 0x80) == usb::DIR_OUT {
                        found_ep_out = true;
                    }
                }
            }
        }

        if !found_ep_in || !found_ep_out {
            error!("[E162] Debug descriptor asks for eps that do not exist. In {}, Out {}",
                   desc.debug_in_endpoint,
                   desc.debug_out_endpoint);
            return false;
        }

        true
    }

    fn check_get_descriptor(&self, h: &usbr::ControlPacketHeader, data: &[u8]) -> bool {

        // Dealing with standard get descriptor requests
        if (h.requesttype & usb::RECIP_MASK) == usb::RECIP_DEVICE {

            if h.status != (usbr::Result::Success as u8) {
                if data.len() != 0 {
                    error!("[E163] Status is 0x{:x} but data.len() {}", h.status, data.len());
                    return false;
                }

                return true;
            }

            match (h.value >> 8) as u8 {

                usb::DT_DEVICE => {
                    if self.check_device_descriptor(h, data) {
                        return true;
                    }
                }

                usb::DT_CONFIG => {
                    if self.check_config_descriptor(h, data) {
                        return true;
                    }
                }

                usb::DT_STRING => {
                    if self.check_string_descriptor(h, data) {
                        return true;
                    }
                }

                usb::DT_INTERFACE => {
                    error!("[E118] Requesting interface directly");
                    return false;
                }

                usb::DT_ENDPOINT => {
                    error!("[E119] Requesting endpoint directly");
                    return false;
                }

                usb::DT_DEVICE_QUALIFIER => {
                    if self.check_device_qual_descriptor(data) {
                        return true;
                    }
                }

                usb::DT_OTHER_SPEED_CONFIG => {
                    if self.check_other_speed_descriptor(data) {
                        return true;
                    }
                }

                usb::DT_INTERFACE_POWER => {
                    if self.check_interface_power_desc(data) {
                        return true;
                    }
                }

                usb::DT_OTG => {
                    if self.check_otg_desc(data) {
                        return true;
                    }
                }

                usb::DT_DEBUG => {
                    if self.check_dbg_desc(data) {
                        return true;
                    }
                }

                usb::DT_INTERFACE_ASSOCIATION => {
                    error!("[E130] Invalid attempt to directly access interface assoc descriptor");
                    return false;
                }

                usb::DT_BOS => {
                    if self.check_bos_desc(data) {
                        return true;
                    }
                }

                usb::DT_DEVICE_CAPABILITY => {
                    error!("[E120] Requesting dev capability directly");
                    return false;
                }

                usb::DT_SS_ENDPOINT_COMP => {
                    error!("[E121] Requesting ss endpoint companion directly");
                    return false;
                }

                _ => {
                    error!("[E122] Unknown or invalid descriptor type: 0x{:x}", h.value >> 8);
                    return false;
                }
            }

        } else if (h.requesttype & usb::RECIP_MASK) == usb::RECIP_INTERFACE {

            let vdev = self.vdev.read().unwrap();
            let interface = get_current_interface!(vdev, h.index as u8);

            if interface.is_none() {
                return false;
            }

            let desc = &interface.unwrap().desc;

            match desc.interface_class {

                usb::CLASS_HID => {

                    let mut hid = self.hid_checks.write().unwrap();
                    if !hid.check_hid_get_desc(h,
                                               data,
                                               interface.unwrap().desc.interface_number,
                                               interface.unwrap().desc.alternate_setting) {
                        return false;
                    }
                }

                _ => {}
            }

            // TODO deal with class-specific descriptors
            return true;
        }

        error!("[E123] Get descriptor 0x{:x} for recipient 0x{:x} unknown",
               h.value >> 8,
               h.requesttype & usb::RECIP_MASK);
        false
    }
}


impl HasHandlers for ControlCheck {
    fn handle_control_packet(&self, source: Source, req: Request) -> (u8, Vec<Request>) {

        let h_ptr = req.type_header.as_ptr() as *const usbr::ControlPacketHeader;
        let h: &usbr::ControlPacketHeader = unsafe { &*h_ptr };

        let transfer_in: bool = (h.requesttype & usb::DIR_IN) == usb::DIR_IN;
        let req_type: u8 = h.requesttype & usb::TYPE_MASK;

        if req_type == usb::TYPE_STANDARD {

            // Standard requests

            match h.request {

                usb::REQ_GET_STATUS => {
                    if transfer_in && source == Source::Red && !check_get_status(h, &req.data) {

                        control_match!(req, "get_status");
                    }
                }

                usb::REQ_CLEAR_FEATURE => {
                    if !transfer_in && !req.data.is_empty() {
                        control_match!(req, "clear feature");
                    }
                }

                usb::REQ_SET_FEATURE => {
                    if !transfer_in && !req.data.is_empty() {
                        control_match!(req, "set feature");
                    }
                }

                usb::REQ_GET_DESCRIPTOR => {
                    if transfer_in && source == Source::Red && !self.check_get_descriptor(h, &req.data) {

                        control_match!(req, "get descriptor");
                    }
                }

                usb::REQ_SET_DESCRIPTOR => {
                    if !transfer_in && source == Source::Red && !req.data.is_empty() {

                        control_match!(req, "set descriptor");
                    }
                }

                usb::REQ_GET_CONFIGURATION => {
                    if transfer_in && source == Source::Red && !self.check_get_config(&req.data) {

                        control_match!(req, "get config");
                    }
                }

                usb::REQ_SET_CONFIGURATION => {

                    if !transfer_in && source == Source::Red && !req.data.is_empty() {
                        control_match!(req, "set config");
                    }

                    if source == Source::Blue {
                        self.update_config(h);
                    }

                }

                usb::REQ_GET_INTERFACE => {
                    if !transfer_in && source == Source::Red && !self.check_get_interface(h, &req.data) {
                        control_match!(req, "get interface");
                    }
                }

                usb::REQ_SET_INTERFACE => {

                    if !transfer_in && source == Source::Red && !req.data.is_empty() {
                        control_match!(req, "set interface");
                    }

                    if source == Source::Blue {
                        self.update_interface(h);
                    }

                }

                usb::REQ_SYNCH_FRAME => {
                    if transfer_in && source == Source::Red && req.data.len() != 2 {

                        control_match!(req, "synch frame");
                    }
                }

                usb::REQ_SET_ADDRESS => {
                    if !transfer_in && source == Source::Red && !req.data.is_empty() {

                        control_match!(req, "set address");
                    }
                }

                _ => {

                    control_match!(req, "usb request: {}", h.request);

                }
            }

        } else if req_type == usb::TYPE_CLASS {

            let vdev = self.vdev.read().unwrap();
            let interface = get_current_interface!(vdev, h.index as u8);

            if interface.is_none() {

                control_match!(req, "request interface");

            }

            let desc = &interface.unwrap().desc;

            match desc.interface_class {

                usb::CLASS_HID => {
                    if !hid::check_hid_request(h, &req.data, source) {
                        control_match!(req, "hid request checks");
                    }
                }

                usb::CLASS_MASS_STORAGE => {
                    if desc.interface_protocol == usb::bbb::PR_BBB {

                        let mut bbb = self.bbb_checks.write().unwrap();

                        if !bbb.check_bbb_request(h, &req.data, source) {
                            control_match!(req, "bbb request checks");
                        }
                    }
                }

                usb::CLASS_PRINTER => {
                    if !printer::check_printer_request(h, &req.data, source) {
                        control_match!(req, "printer request checks");
                    }
                }

                _ => {}
            }

            // TODO check other classes


        } else {
            control_match!(req, "type of control request");
        }

        (NO_MATCH, vec![req])
    }
}





// tests for private functions.
// tests for public functions are in the tests/control_checks.rs file.

#[cfg(test)]
mod tests {

    use byteorder::{ByteOrder, LittleEndian};

    macro_rules! w_u16 {
        ($buf:expr, $val:expr) => {
            LittleEndian::write_u16(&mut $buf, $val);
        }
    }


    fn util_generate_device_desc(data: &mut [u8]) {
        w_u16!(data[..2], 0x0200); // bcd_usb
        data[2] = 0; // device class
        data[3] = 0; // device subclass
        data[4] = 0; // device protocol
        data[5] = 64; // max packet size
        w_u16!(data[6..8], 0x3340); // id vendor
        w_u16!(data[8..10], 0x3457); // id product
        w_u16!(data[10..12], 0x0100); // bcd device
        data[12] = 1; // manufacturer
        data[13] = 2; // product
        data[14] = 3; // serial
        data[15] = 1; // num configs
    }



    #[test]
    fn valid_bcd() {
        assert_eq!(super::check_bcd(0xa000), false);
        assert_eq!(super::check_bcd(0x000f), false);
        assert_eq!(super::check_bcd(0x0b00), false);
        assert_eq!(super::check_bcd(0x00d0), false);
        assert_eq!(super::check_bcd(0xe000), false);
        assert_eq!(super::check_bcd(0x0000), true);
        assert_eq!(super::check_bcd(0x0123), true);
    }


    #[test]
    fn check_str_fields() {

        let index = 1;

        // malformed utf16 1
        let mut data: [u8; 4] = [0; 4];
        w_u16!(data, 0xdc01);
        w_u16!(data[2..], 0xd801);
        assert_eq!(super::check_string_fields(&data, index), false);

        // malformed utf16 2
        let mut data: [u8; 4] = [0; 4];
        w_u16!(data, 0xd801);
        w_u16!(data[2..], 0x0300);
        assert_eq!(super::check_string_fields(&data, index), false);

        // out of range utf16 1
        let mut data: [u8; 2] = [0; 2];
        w_u16!(data, 0xd801);
        assert_eq!(super::check_string_fields(&data, index), false);

        // out of range utf16 2
        let mut data: [u8; 2] = [0; 2];
        w_u16!(data, 0xdfff);
        assert_eq!(super::check_string_fields(&data, index), false);

        // valid utf16 1
        let mut data: [u8; 4] = [0; 4];
        w_u16!(data, 0xfffe);
        w_u16!(data[2..], 0xfeff);
        assert_eq!(super::check_string_fields(&data, index), true);

        // valid utf16 2
        let mut data: [u8; 4] = [0; 4];
        w_u16!(data, 0x4fff);
        w_u16!(&mut data[2..], 0xffff);
        assert_eq!(super::check_string_fields(&data, index), true);
    }

    #[test]
    fn check_device_fields() {

        let mut data: [u8; 16] = [0; 16];
        assert_eq!(super::check_device_fields(&data), false);

        util_generate_device_desc(&mut data);
        assert_eq!(super::check_device_fields(&data), true);

        // invalid conf num
        data[15] = 0;
        assert_eq!(super::check_device_fields(&data), false);

        util_generate_device_desc(&mut data);

        // invalid csp
        data[2] = 1;
        data[3] = 6;
        data[4] = 0;
        assert_eq!(super::check_device_fields(&data), false);

        let mut data: [u8; 19] = [0; 19];
        util_generate_device_desc(&mut data[..16]);

        // invalid length
        assert_eq!(super::check_device_fields(&data), false);
    }

}
