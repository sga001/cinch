#![allow(dead_code)]

pub mod hid;
pub mod bbb;
pub mod printer;

// This file holds USB constants and structures that are needed for
// USB device APIs.  These are used by the USB device model, which is
// defined in chapter 9 of the USB 2.0 specification and in the
// Wireless USB 1.0 (spread around).  Linux has several APIs in C that
// need these:
//
// - the master/host side Linux-USB kernel driver API;
// - the "usbfs" user space API; and
// - the Linux "gadget" slave/device/peripheral side driver API.
//
// USB 2.0 adds an additional "On The Go" (OTG) mode, which lets systems
// act either as a USB master/host or as a USB slave/device.  That means
// the master and slave side APIs benefit from working well together.
//
// There's also "Wireless USB", using low power short range radios for
// peripheral interconnection but otherwise building on the USB framework.
//
// Note all descriptors are declared '__attribute__((packed))' so that:
//
// [a] they never get padded, either internally (USB spec writers
//     probably handled that) or externally;
//
// [b] so that accessing bigger-than-a-bytes fields will never
//     generate bus errors on any platform, even when the location of
//     its descriptor inside a bundle isn't "naturally aligned", and
//
// [c] for consistency, removing all doubt even when it appears to
//     someone that the two other points are non-issues for that
//     particular descriptor type.
//


// -------------------------------------------------------------------------


// USB Version

pub const V1: u16 = 0x0100;
pub const V2: u16 = 0x0200;
pub const V3: u16 = 0x0300;


// CONTROL REQUEST SUPPORT
//
// USB directions
//
// This bit flag is used in endpoint descriptors' bEndpointAddress field.
// It's also one of three fields in control requests bmRequestType.
//

pub const DIR_OUT: u8 = 0;
pub const DIR_IN: u8 = 0x80;
pub const DIR_REMOVE: u8 = 0x7f; // removes direction bit

// USB types, the second of three bmRequestType fields
//
pub const TYPE_MASK: u8 = (0x03 << 5);
pub const TYPE_STANDARD: u8 = 0;
pub const TYPE_CLASS: u8 = (0x01 << 5);
pub const TYPE_VENDOR: u8 = (0x02 << 5);
pub const TYPE_RESERVED: u8 = (0x03 << 5);


// USB recipients, the third of three bmRequestType fields
//

pub const RECIP_MASK: u8 = 0x1f;
pub const RECIP_DEVICE: u8 = 0x00;
pub const RECIP_INTERFACE: u8 = 0x01;
pub const RECIP_ENDPOINT: u8 = 0x02;
pub const RECIP_OTHER: u8 = 0x03;
// From Wireless USB 1.0
pub const RECIP_PORT: u8 = 0x04;
pub const RECIP_RPIPE: u8 = 0x05;

// Standard requests, for the bRequest field of a SETUP packet.
//
// These are qualified by the bRequestType field, so that for example
// TYPE_CLASS or TYPE_VENDOR specific feature flags could be retrieved
// by a GET_STATUS request.
//
pub const REQ_GET_STATUS: u8 = 0x00;
pub const REQ_CLEAR_FEATURE: u8 = 0x01;
pub const REQ_SET_FEATURE: u8 = 0x03;
pub const REQ_SET_ADDRESS: u8 = 0x05;
pub const REQ_GET_DESCRIPTOR: u8 = 0x06;
pub const REQ_SET_DESCRIPTOR: u8 = 0x07;
pub const REQ_GET_CONFIGURATION: u8 = 0x08;
pub const REQ_SET_CONFIGURATION: u8 = 0x09;
pub const REQ_GET_INTERFACE: u8 = 0x0A;
pub const REQ_SET_INTERFACE: u8 = 0x0B;
pub const REQ_SYNCH_FRAME: u8 = 0x0C;
pub const REQ_SET_SEL: u8 = 0x30;
pub const REQ_SET_ISOCH_DELAY: u8 = 0x31;

pub const REQ_SET_ENCRYPTION: u8 = 0x0D;	/* Wireless USB */
pub const REQ_GET_ENCRYPTION: u8 = 0x0E;
pub const REQ_RPIPE_ABORT: u8 = 0x0E;
pub const REQ_SET_HANDSHAKE: u8 = 0x0F;
pub const REQ_RPIPE_RESET: u8 = 0x0F;
pub const REQ_GET_HANDSHAKE: u8 = 0x10;
pub const REQ_SET_CONNECTION: u8 = 0x11;
pub const REQ_SET_SECURITY_DATA: u8 = 0x12;
pub const REQ_GET_SECURITY_DATA: u8 = 0x13;
pub const REQ_SET_WUSB_DATA: u8 = 0x14;
pub const REQ_LOOPBACK_DATA_WRITE: u8 = 0x15;
pub const REQ_LOOPBACK_DATA_READ: u8 = 0x16;
pub const REQ_SET_INTERFACE_DS: u8 = 0x17;

// pub struct usb_ctrlrequest - SETUP data for a USB device control request
// @bRequestType: matches the USB bmRequestType field
// @bRequest: matches the USB bRequest field
// @wValue: matches the USB wValue field (le16 byte order)
// @wIndex: matches the USB wIndex field (le16 byte order)
// @wLength: matches the USB wLength field (le16 byte order)
//
// This structure is used to send control requests to a USB device.  It matches
// the different fields of the USB 2.0 Spec section 9.3, table 9-2.  See the
// USB spec for a fuller description of the different fields, and what they are
// used for.
//
// Note that the driver for any interface can issue control requests.
// For most devices, interfaces don't coordinate with each other, so
// such requests may be made at any time.
//


#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct ControlRequest {
    pub request_type: u8,
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

// -------------------------------------------------------------------------

// STANDARD DESCRIPTORS ... as returned by GET_DESCRIPTOR, or
// (rarely) accepted by SET_DESCRIPTOR.
//
// Note that all multi-byte values here are encoded in little endian
// byte order "on the wire".  Within the kernel and when exposed
// through the Linux-USB APIs, they are not converted to cpu byte
// order; it is the responsibility of the client code to do this.
// The single exception is when device and configuration descriptors (but
// not other descriptors) are read from usbfs (i.e. /proc/bus/usb/BBB/DDD);
// in this case the fields are converted to host endianness by the kernel.
//

// Descriptor types ... USB 2.0 spec table 9.5
//
pub const DT_DEVICE: u8 = 0x01;
pub const DT_CONFIG: u8 = 0x02;
pub const DT_STRING: u8 = 0x03;
pub const DT_INTERFACE: u8 = 0x04;
pub const DT_ENDPOINT: u8 = 0x05;
pub const DT_DEVICE_QUALIFIER: u8 = 0x06;
pub const DT_OTHER_SPEED_CONFIG: u8 = 0x07;
pub const DT_INTERFACE_POWER: u8 = 0x08;
// these are from a minor usb 2.0 revision (ECN)
pub const DT_OTG: u8 = 0x09;
pub const DT_DEBUG: u8 = 0x0a;
pub const DT_INTERFACE_ASSOCIATION: u8 = 0x0b;
// these are from the Wireless USB spec
pub const DT_SECURITY: u8 = 0x0c;
pub const DT_KEY: u8 = 0x0d;
pub const DT_ENCRYPTION_TYPE: u8 = 0x0e;
pub const DT_BOS: u8 = 0x0f;
pub const DT_DEVICE_CAPABILITY: u8 = 0x10;
pub const DT_WIRELESS_ENDPOINT_COMP: u8 = 0x11;
pub const DT_WIRE_ADAPTER: u8 = 0x21;
pub const DT_RPIPE: u8 = 0x22;
pub const DT_CS_RADIO_CONTROL: u8 = 0x23;
// From the T10 UAS specification
pub const DT_PIPE_USAGE: u8 = 0x24;
// From the USB 3.0 spec
pub const DT_SS_ENDPOINT_COMP: u8 = 0x30;

// Conventional codes for class-specific descriptors.  The convention is
// defined in the USB "Common Class" Spec (3.11).  Individual class specs
// are authoritative for their usage, not the "common class" writeup.
//

pub const DT_CS_DEVICE: u8 = (TYPE_CLASS | DT_DEVICE);
pub const DT_CS_CONFIG: u8 = (TYPE_CLASS | DT_CONFIG);
pub const DT_CS_STRING: u8 = (TYPE_CLASS | DT_STRING);
pub const DT_CS_INTERFACE: u8 = (TYPE_CLASS | DT_INTERFACE);
pub const DT_CS_ENDPOINT: u8 = (TYPE_CLASS | DT_ENDPOINT);

// All standard descriptors have these 2 fields at the beginning

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct DescriptorHeader {
    pub length: u8,
    pub descriptor_type: u8,
}

pub const HEADER_SIZE: usize = 2;

// -------------------------------------------------------------------------

// USB_DT_DEVICE: Device descriptor
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct DeviceDescriptor {
    pub bcd_usb: u16,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub max_packet_size0: u8,
    pub id_vendor: u16,
    pub id_product: u16,
    pub bcd_device: u16,
    pub manufacturer: u8,
    pub product: u8,
    pub serial_number: u8,
    pub num_configurations: u8,
}

pub const DEVICE_DESC_SIZE: usize = 16;

// Device and/or Interface Class codes
// as found in bDeviceClass or bInterfaceClass
// and defined by www.usb.org documents
//
pub const CLASS_PER_INTERFACE: u8 = 0; /* for DeviceClass */
pub const CLASS_AUDIO: u8 = 1;
pub const CLASS_COMM: u8 = 2;
pub const CLASS_HID: u8 = 3;
pub const CLASS_PHYSICAL: u8 = 5;
pub const CLASS_STILL_IMAGE: u8 = 6;
pub const CLASS_PRINTER: u8 = 7;
pub const CLASS_MASS_STORAGE: u8 = 8;
pub const CLASS_HUB: u8 = 9;
pub const CLASS_CDC_DATA: u8 = 0x0a;
pub const CLASS_CSCID: u8 = 0x0b;       /* chip+ smart card */
pub const CLASS_CONTENT_SEC: u8 = 0x0d; /* content security */
pub const CLASS_VIDEO: u8 = 0x0e;
pub const CLASS_HEALTH: u8 = 0x0f;
pub const CLASS_AUDIO_VIDEO: u8 = 0x10;
pub const CLASS_BILLBOARD: u8 = 0x11;
pub const CLASS_DIAGNOSTIC: u8 = 0xdc;
pub const CLASS_WIRELESS_CONTROLLER: u8 = 0xe0;
pub const CLASS_MISC: u8 = 0xef;
pub const CLASS_APP_SPEC: u8 = 0xfe;
pub const CLASS_VENDOR_SPEC: u8 = 0xff;

pub const SUBCLASS_VENDOR_SPEC: u8 = 0xff;

// -------------------------------------------------------------------------

// USB_DT_DEVICE_QUALIFIER: Describes information about a high-speed
// capable device that would change if the device were operating at
// the other speed. For example, if the device is currently operating
// full speed, the device qualifier returns information about how it
// would operate at high-speed and vice-versa.
//

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct DeviceQualifierDescriptor {
    pub bcd_usb: u16,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub max_packet_size0: u8,
    pub num_configurations: u8,
    pub reserved: u8,
}

pub const DEVICE_QUALIFIER_SIZE: usize = 8;
// -------------------------------------------------------------------------

// USB_DT_CONFIG: Configuration descriptor information.
//
// USB_DT_OTHER_SPEED_CONFIG is the same descriptor, except that the
// descriptor type is different.  Highspeed-capable devices can look
// different depending on what speed they're currently running.  Only
// devices with a USB_DT_DEVICE_QUALIFIER have any OTHER_SPEED_CONFIG
// descriptors.
//

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct ConfigDescriptor {
    pub total_length: u16,
    pub num_interfaces: u8,
    pub configuration_value: u8,
    pub configuration: u8,
    pub attributes: u8,
    pub max_power: u8,
}

pub const CONFIG_DESC_SIZE: usize = 7;

pub type OtherSpeedDescriptor = ConfigDescriptor;
pub const OTHER_SPEED_SIZE: usize = CONFIG_DESC_SIZE;

// from config descriptor attributes
pub const CONFIG_ATT_ONE: u8 = (1 << 7); /* must be set */
pub const CONFIG_ATT_SELFPOWER: u8 = (1 << 6); /* self powered */
pub const CONFIG_ATT_WAKEUP: u8 = (1 << 5); /* can wakeup */
pub const CONFIG_ATT_BATTERY: u8 = (1 << 4); /* battery powered */

// -------------------------------------------------------------------------

// USB_DT_STRING: String descriptor

// the data portion needs to be read into a Vec<u16> once the size is known.
// #[repr(C, packed)]
// pub struct usb_string_descriptor {
// 	uint16_t *wData;		/* UTF-16LE encoded */
// }


pub const STRING_DESC_SIZE_MIN: usize = 2;

// note that "string" zero is special, it holds language codes that
// the device supports, not Unicode characters.
//

// -------------------------------------------------------------------------

// USB_DT_INTERFACE: Interface descriptor

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct InterfaceDescriptor {
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub interface: u8,
}

pub const INTERFACE_DESC_SIZE: usize = 7;

// -------------------------------------------------------------------------

// USB_DT_ENDPOINT: Endpoint descriptor
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct EndpointDescriptor {
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}

pub const ENDPOINT_DESC_SIZE: usize = 5;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct AudioEndpointDescriptor {
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
    pub refresh: u8,
    pub synch_address: u8,
}

pub const AUDIO_EP_DESC_SIZE: usize = 7;

// Endpoints
//
pub const ENDPOINT_NUMBER_MASK: u8 = 0x0f; /* in bEndpointAddress */
pub const ENDPOINT_DIR_MASK: u8 = 0x80;

pub const ENDPOINT_XFERTYPE_MASK: u8 = 0x03; /* in bmAttributes */
pub const ENDPOINT_XFER_CONTROL: u8 = 0;
pub const ENDPOINT_XFER_ISOC: u8 = 1;
pub const ENDPOINT_XFER_BULK: u8 = 2;
pub const ENDPOINT_XFER_INT: u8 = 3;
pub const ENDPOINT_MAX_ADJUSTABLE: u8 = 0x80;

// The USB 3.0 spec redefines bits 5:4 of bmAttributes as interrupt ep type.
pub const ENDPOINT_INTRTYPE: u8 = 0x30;
pub const ENDPOINT_INTR_PERIODIC: u8 = (0 << 4);
pub const ENDPOINT_INTR_NOTIFICATION: u8 = (1 << 4);

pub const ENDPOINT_SYNCTYPE: u8 = 0x0c;
pub const ENDPOINT_SYNC_NONE: u8 = (0 << 2);
pub const ENDPOINT_SYNC_ASYNC: u8 = (1 << 2);
pub const ENDPOINT_SYNC_ADAPTIVE: u8 = (2 << 2);
pub const ENDPOINT_SYNC_SYNC: u8 = (3 << 2);

pub const ENDPOINT_USAGE_MASK: u8 = 0x30;
pub const ENDPOINT_USAGE_DATA: u8 = 0x00;
pub const ENDPOINT_USAGE_FEEDBACK: u8 = 0x10;
pub const ENDPOINT_USAGE_IMPLICIT_FB: u8 = 0x20; /* Implicit feedback Data endpoint */

// -------------------------------------------------------------------------

// USB_DT_SS_ENDPOINT_COMP: SuperSpeed Endpoint Companion descriptor
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct SsEpCompDescriptor {
    pub max_burst: u8,
    pub attributes: u8,
    pub bytes_per_interval: u16,
}

pub const SS_EP_DESC_SIZE: usize = 4;

// USB_DT_PIPE_USAGE: Pipe usage descriptor sent for USB attached SCSI (UAS)

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct PipeUsageDescriptor {
    pub pipe_id: u8,
    pub reserved: u8,
}

pub const PIPE_DESC_SIZE: usize = 2;


// -------------------------------------------------------------------------

// USB_DT_DEVICE_QUALIFIER: Device Qualifier descriptor
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct QualifierDescriptor {
    pub bcd_usb: u16,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub max_packet_size0: u8,
    pub num_configurations: u8,
    pub reserved: u8,
}


// -------------------------------------------------------------------------

// USB_DT_OTG (from OTG 1.0a supplement)
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct OtgDescriptor {
    pub attributes: u8, // support for HNP, SRP, etc
}

pub const OTG_DESC_SIZE: usize = 1;

// from usb_otg_descriptor.bmAttributes
pub const OTG_SRP: u8 = 0x01;
pub const OTG_HNP: u8 = 0x02; /* swap host/device roles */

// -------------------------------------------------------------------------

// USB_DT_DEBUG:  for special highspeed devices, replacing serial console
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct DebugDescriptor {
    // bulk endpoints with 8 byte maxpacket
    pub debug_in_endpoint: u8,
    pub debug_out_endpoint: u8,
}

pub const DEBUG_DESC_SIZE: usize = 2;

// -------------------------------------------------------------------------

// USB_DT_INTERFACE_ASSOCIATION: groups interfaces
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct InterfaceAssocDescriptor {
    pub first_interface: u8,
    pub interface_count: u8,
    pub function_class: u8,
    pub function_subclass: u8,
    pub function_protocol: u8,
    pub function: u8,
}

pub const INTERFACE_ASSOC_DESC_SIZE: usize = 6;

// -------------------------------------------------------------------------

// USB_DT_BOS:  group of device-level capabilities
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct BosDescriptor {
    pub total_length: u16,
    pub num_device_caps: u8,
}

pub const BOS_DESC_SIZE: usize = 3;

// -------------------------------------------------------------------------

// USB_DT_DEVICE_CAPABILITY:  grouped with BOS
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct DevCapHeader {
    pub dev_capability_type: u8,
}

pub const DEV_CAP_HEADER_SIZE: usize = 1;

pub const CAP_TYPE_WIRELESS: u8 = 0x01;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct WirelessCapDescriptor {
    // Ultra Wide Band */
    // This is in addition to DescriptorHeader and DevCapHeader
    pub attributes: u8,
    pub phy_rates: u16, // bit rates, Mbps
    pub tfi_tx_power_info: u8, // TFI power levels
    pub ffi_tx_power_info: u8, // FFI power levels
    pub band_group: u16,
    pub reserved: u8,
}

pub const CAP_TYPE_WIRELESS_SIZE: usize = 8;

pub const WIRELESS_P2P_DRD: u8 = (1 << 1);
pub const WIRELESS_BEACON_MASK: u8 = (3 << 2);
pub const WIRELESS_BEACON_SELF: u8 = (1 << 2);
pub const WIRELESS_BEACON_DIRECTED: u8 = (2 << 2);
pub const WIRELESS_BEACON_NONE: u8 = (3 << 2);

pub const WIRELESS_PHY_53: u8 = 1;	        /* always set */
pub const WIRELESS_PHY_80: u8 = (1 << 1);
pub const WIRELESS_PHY_107: u8 = (1 << 2);	/* always set */
pub const WIRELESS_PHY_160: u8 = (1 << 3);
pub const WIRELESS_PHY_200: u8 = (1 << 4);	/* always set */
pub const WIRELESS_PHY_320: u8 = (1 << 5);
pub const WIRELESS_PHY_400: u8 = (1 << 6);
pub const WIRELESS_PHY_480: u8 = (1 << 7);

// USB 2.0 Extension descriptor
pub const CAP_TYPE_EXT: u8 = 2;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct ExtCapDescriptor {
    // Link Power Management */
    // This is in addition to DescriptorHeader and DevCapHeader
    pub attributes: u32,
}

pub const CAP_TYPE_EXT_SIZE: usize = 4;


pub const LPM_SUPPORT: u32 = (1 << 1);/* supports LPM */
pub const BESL_SUPPORT: u32 = (1 << 2); /* supports BESL */
pub const BESL_BASELINE_VALID: u32 = (1 << 3); /* Baseline BESL valid*/
pub const BESL_DEEP_VALID: u32 = (1 << 4); /* Deep BESL valid */
pub const BESL_BASELINE_MASK: u32 = (0xf << 8); // requires >> 8 after applying
pub const BESL_DEEP: u32 = (0xf << 12); // requires >> 12 after applying


// SuperSpeed USB Capability descriptor: Defines the set of SuperSpeed USB
// specific device level capabilities
//
pub const CAP_TYPE_SS: u8 = 3;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct SsCapDescriptor {
    // Link Power Management */
    // This is in addition to DescriptorHeader and DevCapHeader
    pub attributes: u8,
    pub speed_supported: u16,
    pub functionality_support: u8,
    pub u1_dev_exit_lat: u8,
    pub u2_dev_exit_lat: u16,
}

pub const CAP_TYPE_SS_SIZE: usize = 7;

pub const LTM_SUPPORT: u8 = (1 << 1); /* supports LTM */
pub const LOW_SPEED_OPERATION: u8 = 1; /* Low speed operation */
pub const FULL_SPEED_OPERATION: u8 = (1 << 1); /* Full speed operation */
pub const HIGH_SPEED_OPERATION: u8 = (1 << 2); /* High speed operation */
pub const SUPER_SPEED_OPERATION: u8 = (1 << 3); /* Operation at 5Gbps */


// Container ID Capability descriptor: Defines the instance unique ID used to
// identify the instance across all operating modes
//
pub const CAP_TYPE_CONTAINER_ID: u8 = 4;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct ContainerIdDescriptor {
    // This is in addition to DescriptorHeader and DevCapHeader
    pub reserved: u8,
    pub container_id: [u8; 16], // 128-bit number
}

pub const CAP_TYPE_CONTAINER_SIZE: usize = 129;

// -------------------------------------------------------------------------

// USB_DT_WIRELESS_ENDPOINT_COMP:  companion descriptor associated with
// each endpoint descriptor for a wireless device
//

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct WirelessEpCompDescriptor {
    pub max_burst: u8,
    pub max_sequence: u8,
    pub max_stream_delay: u16,
    pub over_the_air_packet_size: u16,
    pub over_the_air_interval: u8,
    pub comp_attributes: u8,
}

pub const ENDPOINT_SWITCH_MASK: u8 = 0x03; /* in bmCompAttributes */
pub const ENDPOINT_SWITCH_NO: u8 = 0;
pub const ENDPOINT_SWITCH_SWITCH: u8 = 1;
pub const ENDPOINT_SWITCH_SCALE: u8 = 2;

// -------------------------------------------------------------------------

// USB_DT_INTERFACE_POWER: Descriptor for providing power management
// to particular interfaces. This spec is no longer available anywhere
// in the web (unless you use wayback machine and go back to 2002!).
// It's now in src/spec/ifpm.pdf
//

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct InterfacePowerDescriptor {
    pub capabilities_flags: u8,
}

pub const INTERFACE_POWER_DESC_SIZE: usize = 1;


// -------------------------------------------------------------------------

custom_derive! { #[derive(TryFrom(u16))]
pub enum LangIds {
    Afrikaans = 0x436,
    Albanian = 0x041c,
    ArabicSaudi = 0x0401,
    ArabicIraq = 0x0801,
    ArabicEgypt = 0x0c01,
    ArabicLibya = 0x1001,
    ArabicAlgeria = 0x1401,
    ArabicMorocco = 0x1801,
    ArabicTunisia = 0x1c01,
    ArabicOman = 0x2001,
    ArabicYemen = 0x2401,
    ArabicSyria = 0x2801,
    ArabicJordan = 0x2c01,
    ArabicLebanon = 0x3001,
    ArabicKuwait = 0x3401,
    ArabicUAE = 0x3801,
    ArabicBahrain = 0x3c01,
    ArabicQatar = 0x4001,
    Armenian = 0x042b,
    Assamese = 0x044d,
    AzeriLatin = 0x042c,
    AzeriCyrillic = 0x082c,
    Basque = 0x042d,
    Belarussian = 0x0423,
    Bengali = 0x0445,
    Bulgarian = 0x0402,
    Burmese = 0x0455,
    Catalan = 0x0403,
    ChineseTaiwan = 0x0404,
    Chinese = 0x0804,
    ChineseHK = 0x0c04,
    ChineseSingapore = 0x1004,
    ChineseMacau = 0x1404,
    Croatian = 0x041a,
    Czech = 0x0405,
    Danish = 0x0406,
    Dutch = 0x0413,
    DutchBelgium = 0x0813,
    EnglishUSA = 0x0409,
    EnglishUK = 0x0809,
    EnglishAU = 0x0c09,
    EnglishCA = 0x1009,
    EnglishNZ = 0x1409,
    EnglishIR = 0x1809,
    EnglishSA = 0x1c09,
    EnglishJamaica = 0x2009,
    EnglishCaribbean = 0x2409,
    EnglishBelize = 0x2809,
    EnglishTrinidad = 0x2c09,
    EnglishZimbabwe = 0x3009,
    EnglishPhilippines = 0x3409,
    Estonian = 0x0425,
    Faeroese = 0x0438,
    Farsi = 0x0429,
    Finnish = 0x040b,
    French = 0x040c,
    FrenchBelgium = 0x80c,
    FrenchCA = 0x0c0c,
    FrenchSwiss = 0x100c,
    FrenchLux = 0x140c,
    FrenchMonaco = 0x180c,
    Georgian = 0x0437,
    German = 0x0407,
    GermanSwiss = 0x0807,
    GermanAustria = 0x0c07,
    GermanLux = 0x1007,
    GermanLiechtenstein = 0x1407,
    Greek = 0x0408,
    Gujarati = 0x0447,
    Hebrew = 0x040d,
    Hindi = 0x0439,
    Hungarian = 0x040e,
    Icelandic = 0x040f,
    Indonesian = 0x0421,
    Italian = 0x0410,
    ItalianSwiss = 0x0810,
    Japanese = 0x0411,
    Kannada = 0x044b,
    Kashmiri = 0x0860,
    Kazakh = 0x043f,
    Konkani = 0x0457,
    Korean = 0x0412,
    KoreanJohab = 0x0812,
    Latvian = 0x0426,
    Lithuanian = 0x0427,
    LithuanianClassic = 0x0827,
    Macedonian = 0x042f,
    Malay = 0x043e,
    MalayBrunei = 0x83e,
    Malayalam = 0x044c,
    Manipuri = 0x0458,
    Marathi = 0x044e,
    Nepali = 0x0861,
    Norwegian = 0x0414,
    NorwegianNynorsk = 0x0814,
    Oriya = 0x0448,
    Polish = 0x0415,
    PortugueseBZ = 0x0416,
    Portuguese = 0x0816,
    Punjabi = 0x0446,
    Romanian = 0x0418,
    Russian = 0x0419,
    Sanskrit = 0x044f,
    SerbianCyrillic = 0x0c1a,
    SerbianLatin = 0x081a,
    Sindhi = 0x0459,
    Slovak = 0x041b,
    Slovenian = 0x0424,
    SpanishTraditional = 0x040a,
    SpanishMX = 0x080a,
    SpanishModern = 0x0c0a,
    SpanishGuatemala = 0x100a,
    SpanishCR = 0x140a,
    SpanishPA = 0x180a,
    SpanishDR = 0x1c0a,
    SpanishVE = 0x200a,
    SpanishCO = 0x240a,
    SpanishPE = 0x280a,
    SpanishAR = 0x2c0a,
    SpanishEC = 0x300a,
    SpanishCH = 0x340a,
    SpanishUR = 0x380a,
    SpanishPAR = 0x3c0a,
    SpanishBO = 0x400a,
    SpanishSalvador = 0x440a,
    SpanishHO = 0x480a,
    SpanishNI = 0x4c0a,
    SpanishPR = 0x500a,
    Sutu = 0x0430,
    Swahili = 0x0441,
    Swedish = 0x041d,
    SwedishFinland = 0x081d,
    Tamil = 0x0449,
    Tatar = 0x0444,
    Telugu = 0x044a,
    Thai = 0x041e,
    Turkish = 0x041f,
    Ukranian = 0x0422,
    Urdu = 0x0420,
    UrduIndia = 0x0820,
    UzbekLatin = 0x0443,
    UzbekCyrillic = 0x0843,
    Vietnamese = 0x042a,
    // HID-specific
    HidUdd = 0x04ff,
    HidVendor1 = 0xf0ff,
    HidVendor2 = 0xf4ff,
    HidVendor3 = 0xf8ff,
    HidVendor4 = 0xfcff,
}
}
