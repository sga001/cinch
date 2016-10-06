// hid descriptor types

pub const DT_HID: u8 = 0x21;
pub const DT_HID_REPORT: u8 = 0x22;
pub const DT_HID_PHYSICAL: u8 = 0x23;

// class request values
pub const GET_REPORT: u8 = 0x01;
pub const GET_IDLE: u8 = 0x02;
pub const GET_PROT: u8 = 0x03;

pub const SET_REPORT: u8 = 0x09;
pub const SET_IDLE: u8 = 0x0a;
pub const SET_PROT: u8 = 0x0b;

#[derive(Copy, Clone)]
pub struct HidClassDescriptor {
    pub descriptor_type: u8,
    pub descriptor_length: u16,
}

pub const CLASS_DESC_SIZE: usize = 3;

pub struct HidDescriptor {
    pub bcd_hid: u16,
    pub country_code: u8,
    pub num_descriptors: u8,
    pub desc: Vec<HidClassDescriptor>, // at least 1
}

pub const DESC_MIN_SIZE: usize = 7;

// masks

pub const ITEM_SIZE_MASK: u8 = 0x03;
pub const ITEM_TYPE_MASK: u8 = 0x0c;
pub const ITEM_TAG_MASK: u8 = 0xf0;
pub const ITEM_LONG_TAG: u8 = 0xf0; // same as tag

// report item types

pub const ITEM_MAIN: u8 = 0;
pub const ITEM_GLOBAL: u8 = 1;
pub const ITEM_LOCAL: u8 = 2;

// main tag types

pub const TAG_INPUT: u8 = 0x08;
pub const TAG_OUTPUT: u8 = 0x09;
pub const TAG_COLLECTION: u8 = 0x0a;
pub const TAG_FEATURE: u8 = 0x0b;
pub const TAG_END_COLLECTION: u8 = 0x0c;

// global tag types

pub const TAG_USAGE_PAGE: u8 = 0x00;
pub const TAG_LOGIC_MIN: u8 = 0x01;
pub const TAG_LOGIC_MAX: u8 = 0x02;
pub const TAG_PHYS_MIN: u8 = 0x03;
pub const TAG_PHYS_MAX: u8 = 0x04;
pub const TAG_UNIT_EXP: u8 = 0x05;
pub const TAG_UNIT: u8 = 0x06;
pub const TAG_REPORT_SIZE: u8 = 0x07;
pub const TAG_REPORT_ID: u8 = 0x08;
pub const TAG_REPORT_COUNT: u8 = 0x09;
pub const TAG_PUSH: u8 = 0x0a;
pub const TAG_POP: u8 = 0x0b;

// local tag types


pub const TAG_USAGE: u8 = 0x00;
pub const TAG_USAGE_MIN: u8 = 0x01;
pub const TAG_USAGE_MAX: u8 = 0x02;

pub const TAG_DESIGN_IDX: u8 = 0x03;
pub const TAG_DESIGN_MIN: u8 = 0x04;
pub const TAG_DESIGN_MAX: u8 = 0x05;

pub const TAG_STRING_IDX: u8 = 0x07;
pub const TAG_STRING_MIN: u8 = 0x08;
pub const TAG_STRING_MAX: u8 = 0x09;

pub const TAG_DELIM: u8 = 0x0a;


// attributes contains 3 fields:
// size = bits 0 - 1,
// type = bits 2 - 3,
// tag = bits 4-7 (all bits set---0x0f---implies long format)
//
// data following it is optional

pub struct HidReportItem {
    pub attributes: u8,
    pub data: Vec<u8>, // optional
}

pub const REPORT_MIN_SIZE: usize = 1;
