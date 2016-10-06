pub const VERSION: usize = 0x000700;
pub const CAPS_SIZE: usize = 1;

pub enum Result {
    Success,
    Cancelled,
    Inval, // invalid packet type / length / ep
    Ioerror,
    Stall,
    Timeout,
    Babble, // the device is just sending random stuff
}

pub enum TransferType {
    Control,
    Iso,
    Bulk,
    Interrupt,
    Invalid = 255,
}

pub enum Speed {
    Slow,
    Full,
    High,
    Super,
    Unknown = 255,
}

// 33 different types
pub enum HeaderType {
    Hello,
    DeviceConnect,
    DeviceDisconnect,
    Reset,
    InterfaceInfo,
    EpInfo,
    SetConf,
    GetConf,
    ConfStatus,
    SetAltSetting,
    GetAltSetting,
    AltSettingStatus,
    StartIsoStream,
    StopIsoStream,
    IsoStreamStatus,
    StartIntReceiving,
    StopIntReceiving,
    IntReceivingStatus,
    AllocBulkStreams,
    FreeBulkStreams,
    BulkStreamsStatus,
    CancelDataPacket,
    FilterReject,
    FilterFilter,
    DeviceDisconnectAck,
    StartBulkReceiving,
    StopBulkReceiving,
    BulkReceivingStatus,

    // Data packets
    ControlPacket = 100,
    BulkPacket,
    IsoPacket,
    IntPacket,
    BufferedBulkPacket,
}

pub enum Caps {
    // Supports USB 3 bulk streams
    BulkStreams,
    // The DeviceConnect packet has the DeviceVersionBcd field
    ConnectDeviceVersion,
    // Supports UsbRedirFilterReject and UsbRedirFilterFilter pkts
    Filter,
    // Supports the UsbRedirDeviceDisconnectAck packet
    DeviceDisconnectAck,
    // The EpInfo packet has the MaxPacketSize field
    EpInfoMaxPacketSize,
    // Supports 64 bits ids in UsbRedirHeader
    Cap64BitsIds,
    // Supports 32 bits length in UsbRedirBulkPacketHeader
    Cap32BitsBulkLength,
    // Supports bulk receiving / buffered bulk input
    BulkReceiving,
}

#[repr(C, packed)]
pub struct RedirHeader {
    pub h_type: u32,
    pub len: u32,
    pub id: u64,
}

pub const REDIR_HEADER_SIZE: usize = 16;


#[repr(C, packed)]
pub struct HelloHeader {
    pub version: [u8; 64], // pub caps: Vec<u32>,  we will treat this as the data.
}

pub const HELLO_MIN_SIZE: usize = 64;


#[repr(C, packed)]
pub struct ConnectHeader {
    pub speed: u8,
    pub class: u8,
    pub subclass: u8,
    pub proto: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub version_bcd: u16,
}


#[repr(C, packed)]
pub struct InterfaceInfoHeader {
    pub count: u32,
    pub interface: [u8; 32],
    pub class: [u8; 32],
    pub subclass: [u8; 32],
    pub proto: [u8; 32],
}


#[repr(C, packed)]
pub struct EpInfoHeader {
    pub ep_type: [u8; 32],
    pub interval: [u8; 32],
    pub interface: [u8; 32],
    pub max_packet_size: [u16; 32],
    pub max_streams: [u32; 32],
}


#[repr(C, packed)]
pub struct SetConfHeader {
    pub conf: u8,
}

#[repr(C, packed)]
pub struct ConfStatusHeader {
    pub status: u8,
    pub conf: u8,
}


#[repr(C, packed)]
pub struct SetAltSettingHeader {
    pub interface: u8,
    pub alt: u8,
}


#[repr(C, packed)]
pub struct GetAltSettingHeader {
    pub interface: u8,
}


#[repr(C, packed)]
pub struct AltSettingStatusHeader {
    pub status: u8,
    pub interface: u8,
    pub alt: u8,
}


#[repr(C, packed)]
pub struct StartIsoStreamHeader {
    pub ep: u8,
    pub pkts_per_urb: u8,
    pub no_urbs: u8,
}


#[repr(C, packed)]
pub struct StopIsoStreamHeader {
    pub ep: u8,
}


#[repr(C, packed)]
pub struct IsoStreamStatusHeader {
    pub status: u8,
    pub ep: u8,
}


#[repr(C, packed)]
pub struct StartIntReceivingHeader {
    pub ep: u8,
}

#[repr(C, packed)]
pub struct StopIntReceivingHeader {
    pub ep: u8,
}

#[repr(C, packed)]
pub struct IntReceivingStatusHeader {
    pub status: u8,
    pub ep: u8,
}

#[repr(C, packed)]
pub struct AllocBulkStreamsHeader {
    pub ep_bmask: u32, // bitmask indicating on which eps to alloc streams
    pub no_streams: u32,
}

#[repr(C, packed)]
pub struct FreeBulkStreamsHeader {
    pub ep_bmask: u32, // bitmask
}

#[repr(C, packed)]
pub struct BulkStreamsStatusHeader {
    pub ep_bmask: u32, // bitmask
    pub no_streams: u32,
    pub status: u8,
}

#[repr(C, packed)]
pub struct StartBulkReceivingHeader {
    pub stream_id: u32,
    pub bytes_per_transfer: u32,
    pub ep: u8,
    pub no_transfers: u8,
}

#[repr(C, packed)]
pub struct StopBulkReceivingHeader {
    pub stream_id: u32,
    pub ep: u8,
}


#[repr(C, packed)]
pub struct BulkReceivingStatusHeader {
    pub stream_id: u32,
    pub ep: u8,
    pub status: u8,
}

#[repr(C, packed)]
pub struct ControlPacketHeader {
    pub ep: u8,
    pub request: u8,
    pub requesttype: u8,
    pub status: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

#[repr(C, packed)]
pub struct BulkPacketHeader {
    pub ep: u8,
    pub status: u8,
    pub length: u16,
    pub stream_id: u32,
    pub length_high: u16, // high 16 bits of packet length
}


#[repr(C, packed)]
pub struct IsoPacketHeader {
    pub ep: u8,
    pub status: u8,
    pub length: u16,
}

#[repr(C, packed)]
pub struct IntPacketHeader {
    pub ep: u8,
    pub status: u8,
    pub length: u16,
}


#[repr(C, packed)]
pub struct BufferedBulkPacketHeader {
    pub stream_id: u32,
    pub length: u32,
    pub ep: u8,
    pub status: u8,
}
