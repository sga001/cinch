// Bulk-only storage protocol
pub const PR_BBB: u8 = 0x50;

pub const RESET: u8 = 0xff;
pub const MAX_LUN: u8 = 0xfe;
pub const CBW_SIGN: u32 = 0x43425355;
pub const CSW_SIGN: u32 = 0x53425355;


// Command block status
pub const STAT_OK: u8 = 0x00;
pub const STAT_FAIL: u8 = 0x01;
pub const STAT_PHASE: u8 = 0x02;

#[derive(Copy, Clone)]
pub struct CommandHeader {
    pub signature: u32, // signature that helps identify this data packet as cbw
    pub tag: u32, // this is basically and id to match request with response
}

pub const HEADER_SIZE: usize = 8;


#[derive(Copy, Clone)]
pub struct CommandBlockWrapper {
    pub transfer_length: u32,
    pub flags: u8,
    pub cb_lun: u8,
    pub cb_length: u8, // length of meaningful bytes in bcbw
    pub bcbw: [u8; 16], // command block to be executed by device
}

pub const CBW_SIZE: usize = 23;


#[derive(Copy, Clone)]
pub struct CommandStatusWrapper {
    pub data_residue: u32, // this is transfer_length (from CBW) - actual length
    pub status: u8, // status of command
}

pub const CSW_SIZE: usize = 5;
