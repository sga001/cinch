use std::collections::HashMap;

use usb;
use parser::usbr;
use parser::Source;
use byteorder::{ByteOrder, LittleEndian};

macro_rules! parse_hid_descriptor {

    (usb::hid::DT_HID, $data:expr, $off:ident) => {
        {
            let mut desc = usb::hid::HidDescriptor {
                               bcd_hid: LittleEndian::read_u16(&$data[*$off..*$off + 2]),
                               country_code: $data[*$off + 2],
                               num_descriptors: $data[*$off + 3],
                               desc: vec![],
            };

            *$off += 4;

            if $data.len() < *$off + (usb::hid::CLASS_DESC_SIZE * desc.num_descriptors as usize) {
                error!("[E023-HID] Not enough paylaod for all descriptors");
                return false;
            }

            for _ in 0..desc.num_descriptors {
                desc.desc.push(
                    usb::hid::HidClassDescriptor {
                        descriptor_type: $data[*$off],
                        descriptor_length: LittleEndian::read_u16(&$data[*$off+1..*$off+3]),
                    });

                *$off += 3;
            }

            desc
        }
    };


    (usb::hid::DT_HID_REPORT, $data:expr) => {
            usb::hid::HidReportItem {
                attributes: $data[0],
                data: vec![],
            }
    };
}


pub struct HidControlCheck {
    descs: HashMap<(u8, u8), usb::hid::HidDescriptor>, // maps interface to hid descriptor
}



fn check_hid_desc_fields(h: &usb::DescriptorHeader, desc: &usb::hid::HidDescriptor) -> bool {

    // From Section 7.1, Pages 48-49 in spec/usb-hid.pdf

    if !super::check_bcd(desc.bcd_hid) {
        error!("E024-HID] Invalid bcd for hid descriptor");
        return false;
    }

    if desc.country_code > 35 {
        error!("[E025-HID] Invalid hid country code {}", desc.country_code);
        return false;
    }

    if desc.num_descriptors < 1 {
        error!("[E026-HID] Invalid number of hid descriptors {}",
               desc.num_descriptors);
        return false;
    }

    if h.length as usize !=
       usb::HEADER_SIZE +
       (usb::hid::DESC_MIN_SIZE + ((desc.num_descriptors as usize - 1) * usb::hid::CLASS_DESC_SIZE)) {

        error!("[E027-HID] Invalid hid descriptor length. Expected {} and got {}",
               usb::hid::DESC_MIN_SIZE + ((desc.num_descriptors as usize - 1) * usb::hid::CLASS_DESC_SIZE),
               h.length);
        return false;
    }

    if desc.desc[0].descriptor_type != usb::hid::DT_HID_REPORT {
        error!("[E028-HID] Invalid type 0x{:x} of first class descriptor",
               desc.desc[0].descriptor_type);
        return false;
    }

    for d in &desc.desc {

        match d.descriptor_type {

            usb::hid::DT_HID_REPORT => {
                if d.descriptor_length < 1 || d.descriptor_length > 258 {
                    error!("[E029-HID] Invalid length in HID descriptor for HID report");
                    return false;
                }
            }

            usb::hid::DT_HID_PHYSICAL => {
                error!("[E030-HID] Unsupported physical descriptors");
                return false;
            }

            _ => {
                error!("[E031-HID] Invalid class descriptor type 0x{:x} ", d.descriptor_type);
                return false;
            }

        };

    }

    true
}


fn check_hid_item_fields(item: &usb::hid::HidReportItem) -> bool {

    // From Section 6.2.2.1 - 6.2.2.4, Pages 26-29 in spec/usb-hid.pdf

    match (item.attributes & usb::hid::ITEM_TYPE_MASK) >> 2 {

        usb::hid::ITEM_MAIN => {

            match (item.attributes & usb::hid::ITEM_TAG_MASK) >> 4 {

                usb::hid::TAG_INPUT => {

                    if !item.data.is_empty() && (item.data[0] & 0x80) != 0 {
                        // bit 7 is reserved
                        error!("[E042-HID] hid item input using reserved bit");
                        return false;
                    }

                    // bits 9 - 15, if available, are reserved
                    if item.data.len() > 1 && (item.data[1] & 0xfe) != 0 {
                        error!("[E043-HID] hid item input using reserved bits 9-15");
                        return false;
                    }


                    // bits 16-31, if available, are reserved
                    for i in 2..item.data.len() {
                        if item.data[i] != 0 {
                            error!("[E044-HID] hid item using reserved bits 16-31");
                            return false;
                        }
                    }
                }

                x if x == usb::hid::TAG_OUTPUT || x == usb::hid::TAG_FEATURE => {

                    // bits 9 - 15, if available, are reserved
                    if item.data.len() > 1 && (item.data[1] & 0xfe) != 0 {
                        error!("[E045-HID] hid item input using reserved bits 9-15");
                        return false;
                    }


                    // bits 16-31, if available, are reserved
                    for i in 2..item.data.len() {
                        if item.data[i] != 0 {
                            error!("[E046-HID] hid item using reserved bits 16-31");
                            return false;
                        }
                    }
                }

                usb::hid::TAG_COLLECTION => {

                    if item.data.len() != 1 {
                        error!("[E047-HID] hid collection item with size other than 1");
                        return false;
                    }

                    // reserved between 0x07 and 0x7f
                    if item.data[0] >= 0x07 && item.data[0] <= 0x7f {
                        error!("[E048-HID] hid collection using reserved bits");
                        return false;
                    }
                }

                usb::hid::TAG_END_COLLECTION => {
                    if !item.data.is_empty() {
                        error!("[E049-HID] hid end collection item with non-zero data");
                        return false;
                    }
                }

                _ => {
                    error!("[E050-HID] Invalid item main tag 0x{:x}",
                           (item.attributes & usb::hid::ITEM_TAG_MASK) >> 4);
                    return false;
                }
            }
        }

        usb::hid::ITEM_GLOBAL => {
            match (item.attributes & usb::hid::ITEM_TAG_MASK) >> 4 {

                x if x == usb::hid::TAG_USAGE_PAGE || x == usb::hid::TAG_LOGIC_MIN ||
                     x == usb::hid::TAG_LOGIC_MAX || x == usb::hid::TAG_PHYS_MIN ||
                     x == usb::hid::TAG_PHYS_MAX || x == usb::hid::TAG_UNIT_EXP ||
                     x == usb::hid::TAG_UNIT ||
                     x == usb::hid::TAG_REPORT_SIZE ||
                     x == usb::hid::TAG_REPORT_COUNT || x == usb::hid::TAG_PUSH ||
                     x == usb::hid::TAG_POP => {}

                usb::hid::TAG_REPORT_ID => {

                    if item.data.is_empty() {
                        error!("[E051-HID] hid item with report id without data");
                        return false;
                    }


                    if (item.data.len() == 4 && LittleEndian::read_u32(&item.data[0..4]) == 0) ||
                       (item.data.len() == 2 && LittleEndian::read_u16(&item.data[0..2]) == 0) ||
                       (item.data.len() == 1 && item.data[0] == 0) {

                        error!("[E052-HID] hid item with report id being set to 0");
                        return false;
                    }
                }

                _ => {
                    error!("[E053-HID] Invalid hid global item");
                    return false;
                }
            }
        }

        usb::hid::ITEM_LOCAL => {
            match (item.attributes & usb::hid::ITEM_TAG_MASK) >> 4 {

                x if x == usb::hid::TAG_USAGE || x == usb::hid::TAG_USAGE_MIN || x == usb::hid::TAG_USAGE_MAX ||
                     x == usb::hid::TAG_DESIGN_IDX ||
                     x == usb::hid::TAG_DESIGN_MIN || x == usb::hid::TAG_DESIGN_MAX ||
                     x == usb::hid::TAG_STRING_IDX ||
                     x == usb::hid::TAG_STRING_MAX || x == usb::hid::TAG_DELIM => {}

                _ => {
                    error!("[E054-HID] Invalid hid local item");
                    return false;
                }
            }
        }

        _ => {
            error!("[E055-HID] Invalid hid item type 0x{:x}",
                   (item.attributes & usb::hid::ITEM_TYPE_MASK) >> 2);
            return false;
        }
    }

    true
}


pub fn check_hid_request(h: &usbr::ControlPacketHeader, data: &[u8], source: Source) -> bool {

    match h.request {

        usb::hid::GET_REPORT => {
            if source != Source::Blue {
                if (h.value >> 8) >= 0x0004 {
                    error!("[E002-HID] Get report request using reserved values");
                    return false;
                }

                if h.length as usize != data.len() {
                    error!("[E003-HID] payload size and purported payload differ");
                    return false;
                }
            }
        }

        usb::hid::GET_IDLE => {
            if source != Source::Blue {
                if h.length != 1 {
                    error!("[E004-HID] Invalid Get idle length");
                    return false;
                }

                if (h.value >> 8) != 0 {
                    error!("[E005-HID] Invalid Get idle value");
                    return false;
                }
            }
        }

        usb::hid::GET_PROT => {
            if source != Source::Blue {

                if h.length != 1 {
                    error!("[E006-HID] Get protocol invalid length");
                    return false;
                }

                if h.value != 0 {
                    error!("[E007-HID] Get protocol invalid value");
                    return false;
                }

                if data.len() > 1 || data.len() == 0 || data[0] > 1 {
                    error!("[E008-HID] Get protocol invalid data length ({}) or response",
                           data.len());
                    return false;
                }
            }
        }

        usb::hid::SET_REPORT => {
            if source != Source::Blue && !data.is_empty() {
                error!("[E009-HID] Set report invalid size {}", data.len());
                return false;
            }
        }

        usb::hid::SET_IDLE => {
            if source != Source::Blue && !data.is_empty() {
                error!("[E010-HID] Set idle invalid size {}", data.len());
                return false;
            }
        }

        usb::hid::SET_PROT => {
            if source != Source::Blue {
                if h.value > 1 {
                    error!("[E011-HID] Set protocol invalid wValue (0x{:x})", h.value);
                    return false;
                }

                if !data.is_empty() {
                    error!("[E012-HID] Set protocol invalid data length ({})", data.len());
                    return false;
                }
            }
        }

        _ => {
            error!("[E013-HID] Unrecognized request type: 0x{:x}", h.request);
            return false;
        }
    }

    true
}


impl HidControlCheck {
    pub fn new() -> HidControlCheck {
        HidControlCheck { descs: HashMap::new() }
    }

    pub fn check_hid_desc(&mut self,
                          header: &usb::DescriptorHeader,
                          data: &[u8],
                          off: &mut usize,
                          inum: u8,
                          alt: u8)
                          -> bool {

        if data.len() < *off + usb::hid::DESC_MIN_SIZE {
            error!("[E022-HID] Not enough payload for descriptor.");
            return false;
        }

        // Below updates off automatically
        let desc = parse_hid_descriptor!(usb::hid::DT_HID, &data, off);

        if !check_hid_desc_fields(header, &desc) {
            return false;
        }

        // Add descriptor to our model
        self.descs.entry((inum, alt)).or_insert(desc);

        true
    }


    pub fn check_hid_report_desc(&self, data: &[u8], inum: u8, alt: u8) -> bool {

        let desc: &usb::hid::HidDescriptor = match self.descs.get(&(inum, alt)) {
            None => {
                error!("[E032-HID] Invalid hid report. Descriptor not found in our model.");
                return false;
            }

            Some(v) => v,
        };

        // TODO: This might be imporatnt. Not explicitly in the spec.
        if desc.num_descriptors > 1 {
            error!("[E033-HID] Unsure how to deal with optional descriptors");
            return false;
        }


        if desc.desc.len() < 1 {
            error!("[E034-HID] Invalid number of hid class descriptors");
            return false;
        }

        // Go through each item in the report

        let mut off: usize = 0;

        while off < desc.desc[0].descriptor_length as usize {
            if !self.check_hid_report_item(data, &mut off) {
                return false;
            }
        }

        if off != desc.desc[0].descriptor_length as usize {
            error!("[E035-HID] Offset value is not equal to descriptor length");
            return false;
        }

        true
    }


    pub fn check_hid_report_item(&self, data: &[u8], off: &mut usize) -> bool {

        if data.len() < *off + usb::hid::REPORT_MIN_SIZE {
            error!("[E036-HID] Not enough payload for report item.");
            return false;
        }

        let mut item = parse_hid_descriptor!(usb::hid::DT_HID_REPORT, &data[*off..]);
        *off += 1;

        if (item.attributes & usb::hid::ITEM_TAG_MASK) == usb::hid::ITEM_LONG_TAG {
            // we're dealing with long item format. Not well documented...

            let item_size = (item.attributes & usb::hid::ITEM_SIZE_MASK) as usize;

            if item_size != 0x02 || (item.attributes & usb::hid::ITEM_TYPE_MASK) != (0x03 << 2) {

                error!("[E037-HID] Invalid report item size {} or type 0x{:x}",
                       item_size,
                       item.attributes & usb::hid::ITEM_TYPE_MASK);

                return false;
            }

            // Parse the first (required) part of the optional item data.
            if data.len() < *off + item_size {
                error!("[E038-HID] not enough payload for report item");
                return false;
            }

            item.data.extend_from_slice(&data[*off..*off + item_size]);
            *off += item_size;

            let data_size = item.data[0] as usize;

            if data.len() < *off + data_size {
                error!("[E039-HID not enough payload for report item optional data");
                return false;
            }

            item.data.extend_from_slice(&data[*off..*off + data_size]);
            *off += data_size;

            // TODO: Find a spec to know how to process these. Right now we are just checking
            // the size since we couldn't find anything more useful.

        } else {
            // we're dealing with a short item format.

            if (item.attributes & usb::hid::ITEM_TYPE_MASK) == (0x03 << 2) {
                error!("[E040-HID] Invalid type in short format item");
                return false;
            }

            let item_size = if (item.attributes & usb::hid::ITEM_SIZE_MASK) == 3 {
                4 as usize
            } else {
                (item.attributes & usb::hid::ITEM_SIZE_MASK) as usize
            };

            // Parse the optional item data.
            if data.len() < *off + item_size {
                error!("[E041-HID] Not enough payload for report item optional data");
                return false;
            }

            item.data.extend_from_slice(&data[*off..*off + item_size]);
            *off += item_size;

            if !check_hid_item_fields(&item) {
                return false;
            }
        }

        true
    }

    pub fn check_hid_get_desc(&mut self, h: &usbr::ControlPacketHeader, data: &[u8], inum: u8, alt: u8) -> bool {

        if (h.requesttype & usb::RECIP_MASK) != usb::RECIP_INTERFACE {
            error!("[E014-HID] Request get descriptor for a recipient that's not the interface");
            return false;
        }

        match (h.value >> 8) as u8 {

            usb::hid::DT_HID => {
                let mut off: usize = 0;

                if data.len() < usb::HEADER_SIZE {
                    error!("[E021-HID] Not enough payload for descriptor header");
                    return false;
                }

                let header = unsafe { parse_descriptor!(0, &data[off..]) };
                off += usb::HEADER_SIZE;

                if (header.length as usize) < usb::hid::DESC_MIN_SIZE + usb::HEADER_SIZE ||
                   header.descriptor_type != usb::hid::DT_HID {

                    error!("[E019-HID] Invalid hid descriptor type 0x{:x} or length {}",
                           header.descriptor_type,
                           header.length);
                    return false;
                }

                return self.check_hid_desc(header, data, &mut off, inum, alt);
            }

            usb::hid::DT_HID_REPORT => {
                return self.check_hid_report_desc(data, inum, alt);
            }

            usb::hid::DT_HID_PHYSICAL => {
                error!("[E020-HID] Unsupported HID physical descriptor");
            }

            _ => {
                error!("Unrecognized descriptor type 0x{:x}", h.value >> 8);
            }

        }

        false
    }
}
