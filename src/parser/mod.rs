pub mod usbr;

use std::mem;
use std::slice;
use std::io::{BufReader, BufWriter, Read, Write};
use std::sync::mpsc;

macro_rules! header_type {
    ($x:expr, $header:ident) => {
       $x == usbr::HeaderType::$header as u32
    }
}

macro_rules! get_sender {
    ($source:expr) => {
        match $source {
            Source::Red => Source::Blue,
            Source::Blue => Source::Red,
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    HeaderLength,
    HeaderType,
    Source, // message sent by the wrong endpoint (red or blue machine)
    Version, // message sent by wrong version of usbr
}


// Current state of the parser. It gets updated after processing certain requests.
#[derive(Debug, PartialOrd, PartialEq)]
pub enum ParserState {
    New,
    Init,
    HelloR, // hello recv but not yet sent via push_outcome (needed for 32-bit id quirk)
    Hello,
    IfaceReceived,
    EpReceived,
    Informed, // guest has been informed
    Connected,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Source {
    Red,
    Blue,
}

pub const MAX_BULK_TRANSFER_SIZE: u32 = (128 * 1024 * 1024);
pub const BUFFER_SIZE: usize = 65536;

pub struct Parser {
    pub state: ParserState,
    pub our_caps: [u32; usbr::CAPS_SIZE],
    pub peer_caps: [u32; usbr::CAPS_SIZE],
    pub source: Source,
}

pub struct Request {
    pub header: [u8; usbr::REDIR_HEADER_SIZE], // main usbr header
    pub type_header: Vec<u8>, // header of request (e.g., device connect, control packet).
    pub data: Vec<u8>, // optional data associated with type header
}

impl Request {
    pub fn get_type(&self) -> u32 {

        let h_ptr = self.header.as_ptr() as *const usbr::RedirHeader;
        let h = unsafe { &*h_ptr }; // gives a &T from a *const T

        h.h_type
    }

    pub fn get_total_len(&self) -> usize {

        let h_ptr = self.header.as_ptr() as *const usbr::RedirHeader;
        let h = unsafe { &*h_ptr }; // gives a &T from a *const T

        h.len as usize

    }

    pub fn get_id(&self) -> u64 {

        let h_ptr = self.header.as_ptr() as *const usbr::RedirHeader;
        let h = unsafe { &*h_ptr }; // gives a &T from a *const T

        h.id
    }
}


#[allow(unused_variables)]
pub trait HasHandlers {
    // Handlers take as input the source and the request and output a port (typically 0)
    // and a set of requests. The port is used by the module manager for modules that
    // have conditional outputs (e.g., rule matchers).

    // Header-only packets

    fn handle_hello(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_connect(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_disconnect(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_disconnect_ack(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_reset(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_cancel_data_packet(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_interface_info(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_ep_info(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_get_conf(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_set_conf(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_conf_status(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_get_alt_setting(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_set_alt_setting(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_alt_setting_status(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_start_iso_stream(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_stop_iso_stream(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_iso_stream_status(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_start_int_receiving(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_stop_int_receiving(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_int_receiving_status(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_alloc_bulk_streams(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_free_bulk_streams(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_bulk_streams_status(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_start_bulk_receiving(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_stop_bulk_receiving(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_bulk_receiving_status(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_filter_reject(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }


    // Data packets

    fn handle_control_packet(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_bulk_packet(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_int_packet(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_iso_packet(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }

    fn handle_buffered_bulk_packet(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        (0, vec![req])
    }
}




impl Parser {
    // Constructor
    pub fn new(src: Source) -> Parser {
        Parser {
            state: ParserState::New,
            our_caps: [0; usbr::CAPS_SIZE],
            peer_caps: [0; usbr::CAPS_SIZE],
            source: src,
        }
    }


    pub fn init(&mut self, version: &str, caps: &[u32]) {

        assert!(self.state == ParserState::New);

        debug!("Hello.version size: {}, version: {}",
               mem::size_of::<usbr::HelloHeader>(),
               version);

        // below copies min(caps.len(), our_caps.len()) elements from caps to our_caps

        assert!(caps.len() <= self.our_caps.len());
        self.our_caps[0..caps.len()].clone_from_slice(caps);

        // Only guest (blue machine) has the "ACK" capability
        if self.source == Source::Blue {
            set_cap(&mut self.our_caps, usbr::Caps::DeviceDisconnectAck as usize);
        }

        if !verify_caps(&self.our_caps, "our") {
            panic!("[E000-Parser] Cannot initialize parser");
        }

        self.state = ParserState::Init;
    }

    pub fn handle_hello(&mut self, req: &Request) {

        if self.state >= ParserState::Hello {
            error!("[E001-Parser] Received a second hello message, ignoring");
            return;
        }

        assert!(self.state == ParserState::Init);
        assert!(req.data.len() % 4 == 0);

        let h_ptr = req.type_header.as_ptr() as *const usbr::HelloHeader;
        let hello: &usbr::HelloHeader = unsafe { &*h_ptr }; // gives a &T from a *const T

        let d_ptr = req.data.as_ptr() as *const u32;
        let caps: &[u32] = unsafe { slice::from_raw_parts(d_ptr, req.data.len() / 4) };

        if !verify_caps(caps, "peer") {
            panic!("[E002-Parser] Capacities are invalid");
        }

        assert!(caps.len() <= self.peer_caps.len());

        self.peer_caps[0..caps.len()].clone_from_slice(caps);

        if !is_ascii(&hello.version) {
            panic!("[E003-Parser] version string is not ascii");
        }

        debug!("Peer version len: {}", hello.version.len());
        self.state = ParserState::HelloR;
    }




    pub fn get_type_header_len(&self, h_type: u32, send: bool) -> Result<usize, ParseError> {

        // command for host (i.e., red machine)
        let mut for_red = self.source == Source::Red;

        if send {
            for_red = !for_red
        }

        // Total of 31 message types (not including the 2 filter ones)
        match h_type {

            // Valid: message for either red or blue machine
            x if header_type!(x, Hello) => Ok(usbr::HELLO_MIN_SIZE),

            x if header_type!(x, ControlPacket) => Ok(mem::size_of::<usbr::ControlPacketHeader>()),

            x if header_type!(x, BulkPacket) => Ok(mem::size_of::<usbr::BulkPacketHeader>()),

            x if header_type!(x, IsoPacket) => Ok(mem::size_of::<usbr::IsoPacketHeader>()),

            x if header_type!(x, IntPacket) => Ok(mem::size_of::<usbr::IntPacketHeader>()),

            // Valid: message for red
            x if header_type!(x, Reset) => if for_red { Ok(0) } else { Err(ParseError::Source) },

            x if header_type!(x, DeviceDisconnectAck) => if for_red { Ok(0) } else { Err(ParseError::Source) },

            x if header_type!(x, SetConf) => {
                if for_red { Ok(mem::size_of::<usbr::SetConfHeader>()) } else { Err(ParseError::Source) }
            }

            x if header_type!(x, GetConf) => if for_red { Ok(0) } else { Err(ParseError::Source) },

            x if header_type!(x, SetAltSetting) => {
                if for_red { Ok(mem::size_of::<usbr::SetAltSettingHeader>()) } else { Err(ParseError::Source) }
            }

            x if header_type!(x, GetAltSetting) => {
                if for_red { Ok(mem::size_of::<usbr::GetAltSettingHeader>()) } else { Err(ParseError::Source) }
            }

            x if header_type!(x, StartIsoStream) => {
                if for_red { Ok(mem::size_of::<usbr::StartIsoStreamHeader>()) } else { Err(ParseError::Source) }
            }

            x if header_type!(x, StopIsoStream) => {
                if for_red { Ok(mem::size_of::<usbr::StopIsoStreamHeader>()) } else { Err(ParseError::Source) }
            }

            x if header_type!(x, StartIntReceiving) => {
                if for_red { Ok(mem::size_of::<usbr::StartIntReceivingHeader>()) } else { Err(ParseError::Source) }
            }

            x if header_type!(x, StopIntReceiving) => {
                if for_red { Ok(mem::size_of::<usbr::StopIntReceivingHeader>()) } else { Err(ParseError::Source) }
            }

            x if header_type!(x, AllocBulkStreams) => {
                if for_red { Ok(mem::size_of::<usbr::AllocBulkStreamsHeader>()) } else { Err(ParseError::Source) }
            }

            x if header_type!(x, FreeBulkStreams) => {
                if for_red { Ok(mem::size_of::<usbr::FreeBulkStreamsHeader>()) } else { Err(ParseError::Source) }
            }

            x if header_type!(x, CancelDataPacket) => if for_red { Ok(0) } else { Err(ParseError::Source) },

            x if header_type!(x, StartBulkReceiving) => {
                if for_red {
                    Ok(mem::size_of::<usbr::StartBulkReceivingHeader>())
                } else {
                    Err(ParseError::Source)
                }
            }

            x if header_type!(x, StopBulkReceiving) => {
                if for_red { Ok(mem::size_of::<usbr::StopBulkReceivingHeader>()) } else { Err(ParseError::Source) }
            }

            // Valid: message for blue
            x if header_type!(x, DeviceDisconnect) => if !for_red { Ok(0) } else { Err(ParseError::Source) },

            x if header_type!(x, ConfStatus) => {
                if !for_red { Ok(mem::size_of::<usbr::ConfStatusHeader>()) } else { Err(ParseError::Source) }
            }

            x if header_type!(x, AltSettingStatus) => {
                if !for_red { Ok(mem::size_of::<usbr::AltSettingStatusHeader>()) } else { Err(ParseError::Source) }
            }

            x if header_type!(x, IsoStreamStatus) => {
                if !for_red { Ok(mem::size_of::<usbr::IsoStreamStatusHeader>()) } else { Err(ParseError::Source) }
            }

            x if header_type!(x, IntReceivingStatus) => {
                if !for_red {
                    Ok(mem::size_of::<usbr::IntReceivingStatusHeader>())
                } else {
                    Err(ParseError::Source)
                }
            }

            x if header_type!(x, BulkStreamsStatus) => {
                if !for_red {
                    Ok(mem::size_of::<usbr::BulkStreamsStatusHeader>())
                } else {
                    Err(ParseError::Source)
                }
            }

            x if header_type!(x, BulkReceivingStatus) => {
                if !for_red {
                    Ok(mem::size_of::<usbr::BulkReceivingStatusHeader>())
                } else {
                    Err(ParseError::Source)
                }
            }

            x if header_type!(x, BufferedBulkPacket) => {
                if !for_red {
                    Ok(mem::size_of::<usbr::BufferedBulkPacketHeader>())
                } else {
                    Err(ParseError::Source)
                }
            }

            x if header_type!(x, InterfaceInfo) => {
                if !for_red { Ok(mem::size_of::<usbr::InterfaceInfoHeader>()) } else { Err(ParseError::Source) }
            }

            x if header_type!(x, EpInfo) => {
                if !for_red { Ok(mem::size_of::<usbr::EpInfoHeader>()) } else { Err(ParseError::Source) }
            }

            x if header_type!(x, DeviceConnect) => {
                if !for_red { Ok(mem::size_of::<usbr::ConnectHeader>()) } else { Err(ParseError::Source) }
            }

            x if header_type!(x, FilterReject) => if for_red { Ok(0) } else { Err(ParseError::Source) },
            // All others are either not supported (e.g., filter stuff) or invalid
            _ => Err(ParseError::HeaderType),
        }
    }


    pub fn expect_extra_data(&self, h_type: u32) -> bool {

        match h_type {
            x if header_type!(x, Hello) => true,
            x if header_type!(x, ControlPacket) => true,
            x if header_type!(x, BulkPacket) => true,
            x if header_type!(x, IsoPacket) => true,
            x if header_type!(x, IntPacket) => true,
            x if header_type!(x, BufferedBulkPacket) => true,
            _ => false,
        }
    }


    pub fn verify_type_header(&self, h_type: u32, header: &[u8], data: &[u8], send: bool) -> bool {

        let mut for_red = false;  // request is for red machine
        let mut data_len: usize = 0;
        let mut target_ep: i32 = -1; // USB endpoint receiving the request
        let mut connected_state: bool = true; // if true, self.state must be connected

        if self.source == Source::Red {
            for_red = true;
        }

        if send {
            for_red = !for_red;
        }

        if self.state < ParserState::Hello && !header_type!(h_type, Hello) {
            error!("[E004-Parser] Received a request before processing hello message");
            return false;
        }

        // These checks are in addition to the get_type_header_len checks (which already check
        // direction). So there is no need to repeat them here.

        match h_type {

            // Device commands
            x if header_type!(x, Hello) => {
                connected_state = false;
            }
            x if header_type!(x, DeviceConnect) => {

                if self.state < ParserState::Informed {
                    error!("[E005-Parser] Received a connect before guest is informed: {:?}",
                           self.state);
                    return false;
                }

                connected_state = false;

            }
            x if header_type!(x, DeviceDisconnect) => {}

            x if header_type!(x, DeviceDisconnectAck) => {

                if (send && !has_cap(&self.our_caps, usbr::Caps::DeviceDisconnectAck as usize)) ||
                   (!send && !has_cap(&self.peer_caps, usbr::Caps::DeviceDisconnectAck as usize)) {

                    error!("[E006-Parser] Device disconnect ack without capability for it");
                    return false;
                }

                let len = self.get_type_header_len(h_type, send);

                match len {

                    Ok(v) => {
                        if v != header.len() {
                            error!("[E007-Parser] Header type {} and length {} do not match", h_type, v);
                            return false;
                        }
                    }

                    Err(e) => {
                        error!("[E008-Parser] {:?}", e);
                        return false;
                    }
                }

            }

            x if header_type!(x, Reset) => {
                connected_state = false;
            }

            x if header_type!(x, CancelDataPacket) => {
                connected_state = false;
            }

            x if header_type!(x, FilterReject) => {
                connected_state = false;
            }

            // Device information requests
            x if header_type!(x, InterfaceInfo) => {

                let len = self.get_type_header_len(x, send);

                match len {

                    Ok(v) => {
                        if v == header.len() {

                            let h_ptr = header.as_ptr() as *const usbr::InterfaceInfoHeader;
                            let iface = unsafe { &*h_ptr }; // gives a &T from a *const T

                            if iface.count > 32 {
                                error!("[E009-Parser] Interface counter ({}) is > 32", iface.count);
                                return false;
                            }

                        } else {
                            error!("[E010-Parser] Header type {} and length {} do not match", h_type, v);
                            return false;
                        }
                    }

                    Err(e) => {
                        error!("[E011-Parser] {:?}", e);
                        return false;
                    }
                }

                connected_state = false;
            }

            x if header_type!(x, EpInfo) => {

                connected_state = false;
            }

            x if header_type!(x, SetConf) => {
                connected_state = false;
            }
            x if header_type!(x, GetConf) => {
                connected_state = false;
            }
            x if header_type!(x, ConfStatus) => {

                if self.state < ParserState::Informed {
                    error!("[E012-Parser] Conf status received before guest is informed {:?}",
                           self.state);
                    return false;
                }

                connected_state = false;
            }

            x if header_type!(x, SetAltSetting) => {
                connected_state = false;
            }

            x if header_type!(x, GetAltSetting) => {
                connected_state = false;
            }

            x if header_type!(x, AltSettingStatus) => {

                if self.state < ParserState::EpReceived {
                    error!("[E013-Parser] AltSettingStatus received before receiving EpInfo and InterfaceInfo");
                    return false;
                }

                connected_state = false;
            }

            // Iso requests
            x if header_type!(x, StartIsoStream) => {}
            x if header_type!(x, StopIsoStream) => {}
            x if header_type!(x, IsoStreamStatus) => {}
            x if header_type!(x, IsoPacket) => {

                let len = self.get_type_header_len(h_type, send);

                match len {

                    Ok(v) => {
                        if v == header.len() {

                            let h_ptr = header.as_ptr() as *const usbr::IsoPacketHeader;
                            let iso = unsafe { &*h_ptr };

                            data_len = iso.length as usize;
                            target_ep = iso.ep as i32;

                        } else {
                            error!("[E014-Parser] Header type {} and length {} do not match", h_type, v);
                            return false;
                        }
                    }

                    Err(e) => {
                        error!("[E015-Parser] {:?}", e);
                        return false;
                    }
                }

            }

            // Interrupt requests
            x if header_type!(x, StartIntReceiving) => {

                let len = self.get_type_header_len(h_type, send);

                match len {

                    Ok(v) => {
                        if v == header.len() {

                            let h_ptr = header.as_ptr() as *const usbr::StartIntReceivingHeader;
                            let s_int = unsafe { &*h_ptr };

                            if s_int.ep & 0x80 == 0 {
                                error!("[E016-Parser] Start int receiving on non input ep {}", s_int.ep);
                                return false;
                            }
                        } else {
                            error!("[E017-Parser] Header type {} and length {} do not match", h_type, v);
                            return false;
                        }
                    }

                    Err(e) => {
                        error!("[E018-Parser] {:?}", e);
                        return false;
                    }
                }
            }

            x if header_type!(x, StopIntReceiving) => {

                let len = self.get_type_header_len(h_type, send);

                match len {

                    Ok(v) => {
                        if v == header.len() {


                            let h_ptr = header.as_ptr() as *const usbr::StopIntReceivingHeader;
                            let s_int = unsafe { &*h_ptr };

                            if s_int.ep & 0x80 == 0 {
                                error!("[E019-Parser] Stop int receiving on non input ep {}", s_int.ep);
                                return false;
                            }
                        } else {
                            error!("[E020-Parser] Header type {} and length {} do not match", h_type, v);
                            return false;
                        }
                    }

                    Err(e) => {
                        error!("[E021-Parser] {:?}", e);
                        return false;
                    }
                }
            }

            x if header_type!(x, IntReceivingStatus) => {

                let len = self.get_type_header_len(h_type, send);

                match len {

                    Ok(v) => {
                        if v == header.len() {

                            let h_ptr = header.as_ptr() as *const usbr::IntReceivingStatusHeader;
                            let s_int = unsafe { &*h_ptr };

                            if s_int.ep & 0x80 == 0 {
                                error!("[E022-Parser] Int receiving status for non input ep {}", s_int.ep);
                                return false;
                            }
                        } else {
                            error!("[E023-Parser] Header type {} and length {} do not match", h_type, v);
                            return false;
                        }
                    }

                    Err(e) => {
                        error!("[E024-Parser] {:?}", e);
                        return false;
                    }
                }
            }

            x if header_type!(x, IntPacket) => {

                let len = self.get_type_header_len(h_type, send);

                match len {

                    Ok(v) => {
                        if v == header.len() {

                            let h_ptr = header.as_ptr() as *const usbr::IntPacketHeader;
                            let int_p = unsafe { &*h_ptr };

                            data_len = int_p.length as usize;
                            target_ep = int_p.ep as i32;

                        } else {
                            error!("[E025-Parser] Header type {} and length {} do not match", h_type, v);
                            return false;
                        }
                    }

                    Err(e) => {
                        error!("[E026-Parser] {:?}", e);
                        return false;
                    }
                }

            }

            // Bulk requests
            x if header_type!(x, StartBulkReceiving) => {

                let len = self.get_type_header_len(h_type, send);

                match len {

                    Ok(v) => {
                        if v == header.len() {

                            let h_ptr = header.as_ptr() as *const usbr::StartBulkReceivingHeader;
                            let bulk = unsafe { &*h_ptr };


                            if bulk.bytes_per_transfer > MAX_BULK_TRANSFER_SIZE {
                                error!("[E027-Parser] Start bulk receiving length exceeds limits {} > {}",
                                       bulk.bytes_per_transfer,
                                       MAX_BULK_TRANSFER_SIZE);
                                return false;
                            }

                            if bulk.ep & 0x80 == 0 {
                                error!("[E028-Parser] Start bulk receiving for non input ep {}", bulk.ep);
                                return false;
                            }


                        } else {
                            error!("[E029-Parser] Header type {} and length {} do not match", h_type, v);
                            return false;
                        }
                    }

                    Err(e) => {
                        error!("[E030-Parser] {:?}", e);
                        return false;
                    }
                }
            }

            x if header_type!(x, StopBulkReceiving) => {

                let len = self.get_type_header_len(h_type, send);

                match len {

                    Ok(v) => {
                        if v == header.len() {

                            let h_ptr = header.as_ptr() as *const usbr::StopBulkReceivingHeader;
                            let bulk = unsafe { &*h_ptr };

                            if bulk.ep & 0x80 == 0 {
                                error!("[E031-Parser] Stop bulk receiving for non input ep {}", bulk.ep);
                                return false;
                            }


                        } else {
                            error!("[E032-Parser] Header type {} and length {} do not match", h_type, v);
                            return false;
                        }
                    }

                    Err(e) => {
                        error!("[E033-Parser] {:?}", e);
                        return false;
                    }
                }
            }

            x if header_type!(x, BulkReceivingStatus) => {

                let len = self.get_type_header_len(h_type, send);

                match len {

                    Ok(v) => {
                        if v == header.len() {


                            let h_ptr = header.as_ptr() as *const usbr::BulkReceivingStatusHeader;
                            let bulk = unsafe { &*h_ptr };

                            if bulk.ep & 0x80 == 0 {
                                error!("[E034-Parser] Bulk receiving status for non input ep {}", bulk.ep);
                                return false;
                            }


                        } else {
                            error!("[E035-Parser] Header type {} and length {} do not match", h_type, v);
                            return false;
                        }
                    }

                    Err(e) => {
                        error!("[E036-Parser] {:?}", e);
                        return false;
                    }
                }
            }

            x if header_type!(x, AllocBulkStreams) => {}
            x if header_type!(x, FreeBulkStreams) => {}
            x if header_type!(x, BulkStreamsStatus) => {}
            x if header_type!(x, BulkPacket) => {

                let len = self.get_type_header_len(h_type, send);

                match len {

                    Ok(v) => {
                        if v == header.len() {


                            let h_ptr = header.as_ptr() as *const usbr::BulkPacketHeader;
                            let bulk = unsafe { &*h_ptr };

                            data_len = ((bulk.length_high as usize) << 16) | bulk.length as usize;

                            if data_len > MAX_BULK_TRANSFER_SIZE as usize {
                                error!("[E037-Parser] bulk transfer length exceeds limits {} > {}",
                                       data_len,
                                       MAX_BULK_TRANSFER_SIZE);
                                return false;
                            }

                            target_ep = bulk.ep as i32;

                        } else {
                            error!("[E038-Parser] Header type {} and length {} do not match", h_type, v);
                            return false;
                        }
                    }

                    Err(e) => {
                        error!("[E039-Parser] {:?}", e);
                        return false;
                    }
                }


            }

            x if header_type!(x, BufferedBulkPacket) => {

                let len = self.get_type_header_len(h_type, send);

                match len {

                    Ok(v) => {
                        if v == header.len() {


                            let h_ptr = header.as_ptr() as *const usbr::BufferedBulkPacketHeader;
                            let bulk = unsafe { &*h_ptr };

                            data_len = bulk.length as usize;

                            if data_len > MAX_BULK_TRANSFER_SIZE as usize {
                                error!("[E040-Parser] buffered bulk transfer length exceeds limits {} > {}",
                                       data_len,
                                       MAX_BULK_TRANSFER_SIZE);
                                return false;
                            }

                            target_ep = bulk.ep as i32;

                        } else {
                            error!("[E041-Parser] Header type {} and length {} do not match", h_type, v);
                            return false;
                        }
                    }

                    Err(e) => {
                        error!("[E042-Parser] {:?}", e);
                        return false;
                    }
                }
            }

            // Control requests
            x if header_type!(x, ControlPacket) => {

                let len = self.get_type_header_len(h_type, send);

                match len {

                    Ok(v) => {
                        if v == header.len() {


                            let h_ptr = header.as_ptr() as *const usbr::ControlPacketHeader;
                            let ctrl = unsafe { &*h_ptr };

                            target_ep = ctrl.ep as i32;
                            data_len = ctrl.length as usize;

                        } else {
                            error!("[E043-Parser] Header type {} and length {} do not match", h_type, v);
                            return false;
                        }
                    }

                    Err(e) => {
                        error!("[E044-Parser] {:?}", e);
                        return false;
                    }
                }

            }

            // The rest (filter and invalid types)
            _ => return false,
        }

        if connected_state && self.state < ParserState::Connected {
            error!("[E045-Parser] received a packet that requires the device to be connected but \
                    the state is {:?}",
                   self.state);
            return false;
        }


        if target_ep != -1 {

            let expect_extra = ((target_ep & 0x80 != 0) && !for_red) || ((target_ep & 0x80 == 0) && for_red);

            if expect_extra {

                if data_len != data.len() {
                    error!("[E046-Parser] data.len() {} != data_len {} ep {}",
                           data.len(),
                           data_len,
                           target_ep);
                    return false;
                }

            } else {

                if data.len() > 0 {
                    error!("[E047-Parser] Unexpected extra data of size {} at ep {}",
                           data.len(),
                           target_ep);
                    return false;
                }

                match h_type {

                    x if header_type!(x, IsoPacket) => {
                        error!("[E048-Parser] Iso packet sent in wrong direction");
                        return false;
                    }

                    x if header_type!(x, IntPacket) => {
                        error!("[E049-Parser] Int packet sent in wrong direction");
                        return false;
                    }

                    x if header_type!(x, BufferedBulkPacket) => {
                        error!("[E050-Parser] Buffered bulk packet sent in wrong direction");
                        return false;
                    }

                    _ => {}
                }
            }
        }

        true
    }

    // This is usually called when we encounter a packet that is corrupted and we need to
    // get rid of the rest of the data associated with that packet so we can move on.
    // This call either returns (which means all is good), or it panics (cannot recover)
    pub fn discard_current_request<T: Read>(&self, input: &mut BufReader<T>, len: usize) {

        // Discard packet and try again
        let mut discard: Vec<u8> = Vec::with_capacity(len);

        match input.read_exact(&mut discard[..len]) {

            Ok(_) => {
                debug!("Discarding corrupted packet");
            }

            Err(e) => {
                panic!("[E051-Parser] Unable to read discard buffer from TcpStream. {:?}", e);
            }
        }
    }


    pub fn pull_next_request<T: Read>(&self, mut input: &mut BufReader<T>) -> Request {


        loop {
            // until we succeed or we fail in a way that we can't recover

            let mut request = Request {
                header: [0; usbr::REDIR_HEADER_SIZE],
                type_header: Vec::new(),
                data: Vec::new(),
            };

            // get header (this blocks)
            let ret = if self.state < ParserState::HelloR {
                // Quirk: The first message that usbr sends has a 32-bit id, and it is
                // then upgraded to 64-bits. We skip the last 4 bytes to account for
                // this behavior
                input.read_exact(&mut request.header[..usbr::REDIR_HEADER_SIZE - 4])
            } else {
                input.read_exact(&mut request.header)
            };

            match ret {
                Ok(_) => {}
                Err(e) => {
                    panic!("[E052-Parser] Unable to read full header from TcpStream. {:?}", e);
                }
            }

            let total_len: usize = request.get_total_len();
            let h_type: u32 = request.get_type();

            // get size of type header (assuming above is valid)
            let type_len: usize = match self.get_type_header_len(h_type, false) {
                Ok(v) => v,
                Err(e) => {

                    error!("[E053-Parser] Could not get type header ({}), length: {:?}",
                           h_type,
                           e);
                    self.discard_current_request(&mut input, total_len); // returns (ok) or panics
                    continue;

                }
            };

            // Check that the total length makes sense given the type header
            if total_len < type_len || (total_len > type_len) && !self.expect_extra_data(h_type) {

                error!("[E054-Parser] Total length does not make sense given the type of header");

                self.discard_current_request(&mut input, total_len); // returns (ok) or panics
                continue;
            }

            // reserve space for type header and parse type header
            request.type_header.extend_from_slice(&vec![0; type_len]);

            match input.read_exact(&mut request.type_header) { // this blocks
                Ok(_) => {}
                Err(e) => {
                    panic!("[E055-Parser] Unable to read full type header from TcpStream. {:?}",
                           e);
                }
            }


            // reserve space for data and parse it (if any)
            let mut data_len: usize = 0;

            if total_len > type_len {

                data_len = total_len - type_len;
                request.data.extend_from_slice(&vec![0; data_len]);

                match input.read_exact(&mut request.data) {
                    Ok(_) => {}
                    Err(e) => {
                        panic!("[E056-Parser] Unable to read full data from TcpStream. {:?}", e);
                    }
                }
            }


            // We are almost out of the woods! Just need to check that everything is consistent.
            if self.verify_type_header(h_type,
                                       &request.type_header[..type_len],
                                       &request.data[..data_len],
                                       false) {
                // We are good! return request.
                return request;

            } else {

                // Bad news bears...
                error!("[E057-Parser] Type header verification failed. Type header and data are inconsistent");
                debug!("Discarding corrupted packet"); //implicit since we've already read the data

            }

        }
    }

    pub fn push_outputs<W: Write>(&mut self, mut output: &mut BufWriter<W>, requests: Vec<Request>) {

        // TODO: this might be inefficient. Revisit if there are performance issues.

        for req in requests {

            if self.state == ParserState::HelloR {
                // Quirk: USBR typically starts off with 32-bit ids, and then upgrades to 64-bit
                // ids.
                // We skip the first 4 bytes to account for this initial message.
                output.write(&req.header[..usbr::REDIR_HEADER_SIZE - 4]).unwrap();
                self.process_state_change(ParserState::Hello);
            } else {
                output.write(&req.header).unwrap();
            }

            output.write(&req.type_header).unwrap();
            output.write(&req.data).unwrap();
        }

    }


    pub fn process_request<T: HasHandlers>(&mut self,
                                           handlers: &T,
                                           req: Request,
                                           tx: &mpsc::Sender<ParserState>)
                                           -> Vec<Request> {

        // (1) Figure out if operation leads to a parser state change. If so, update state and
        //     propagate info to the other parser (via channel tx).
        //
        // (2) Call the apropriate handler, which returns a Vec<Request> as a response
        //     The default handlers simply return the same request.

        let (_, out) = match req.get_type() {

            // Requests that could lead to a parser state change
            x if header_type!(x, Hello) => {
                self.handle_hello(&req);
                handlers.handle_hello(get_sender!(self.source), req)
            }

            x if header_type!(x, DeviceConnect) => {

                if self.process_state_change(ParserState::Connected) {
                    tx.send(ParserState::Connected).unwrap();
                }

                handlers.handle_connect(get_sender!(self.source), req)
            }

            x if header_type!(x, InterfaceInfo) => {

                if self.process_state_change(ParserState::IfaceReceived) {
                    tx.send(ParserState::IfaceReceived).unwrap();
                }

                handlers.handle_interface_info(get_sender!(self.source), req)

            }

            x if header_type!(x, EpInfo) => {

                if self.process_state_change(ParserState::EpReceived) {
                    tx.send(ParserState::EpReceived).unwrap();
                }

                handlers.handle_ep_info(get_sender!(self.source), req)

            }

            // Requests that do not lead to a parser state change
            x if header_type!(x, SetConf) => handlers.handle_set_conf(get_sender!(self.source), req),
            x if header_type!(x, SetAltSetting) => handlers.handle_set_alt_setting(get_sender!(self.source), req),
            x if header_type!(x, DeviceDisconnect) => handlers.handle_disconnect(get_sender!(self.source), req),
            x if header_type!(x, Reset) => handlers.handle_reset(get_sender!(self.source), req),
            x if header_type!(x, ConfStatus) => handlers.handle_conf_status(get_sender!(self.source), req),
            x if header_type!(x, GetAltSetting) => handlers.handle_get_alt_setting(get_sender!(self.source), req),
            x if header_type!(x, AltSettingStatus) => {
                handlers.handle_alt_setting_status(get_sender!(self.source), req)
            }
            x if header_type!(x, StartIsoStream) => {
                handlers.handle_start_iso_stream(get_sender!(self.source), req)
            }
            x if header_type!(x, StopIsoStream) => handlers.handle_stop_iso_stream(get_sender!(self.source), req),
            x if header_type!(x, IsoStreamStatus) => {
                handlers.handle_iso_stream_status(get_sender!(self.source), req)
            }
            x if header_type!(x, StartIntReceiving) => {
                handlers.handle_start_int_receiving(get_sender!(self.source), req)
            }
            x if header_type!(x, IntReceivingStatus) => {
                handlers.handle_int_receiving_status(get_sender!(self.source), req)
            }
            x if header_type!(x, AllocBulkStreams) => {
                handlers.handle_alloc_bulk_streams(get_sender!(self.source), req)
            }
            x if header_type!(x, FreeBulkStreams) => {
                handlers.handle_free_bulk_streams(get_sender!(self.source), req)
            }
            x if header_type!(x, BulkStreamsStatus) => {
                handlers.handle_bulk_streams_status(get_sender!(self.source), req)
            }
            x if header_type!(x, CancelDataPacket) => {
                handlers.handle_cancel_data_packet(get_sender!(self.source), req)
            }
            x if header_type!(x, DeviceDisconnectAck) => {
                handlers.handle_disconnect_ack(get_sender!(self.source), req)
            }
            x if header_type!(x, StartBulkReceiving) => {
                handlers.handle_start_bulk_receiving(get_sender!(self.source), req)
            }
            x if header_type!(x, StopBulkReceiving) => {
                handlers.handle_stop_bulk_receiving(get_sender!(self.source), req)
            }
            x if header_type!(x, BulkReceivingStatus) => {
                handlers.handle_bulk_receiving_status(get_sender!(self.source), req)
            }

            x if header_type!(x, FilterReject) => handlers.handle_filter_reject(get_sender!(self.source), req),
            x if header_type!(x, ControlPacket) => handlers.handle_control_packet(get_sender!(self.source), req),
            x if header_type!(x, BulkPacket) => handlers.handle_bulk_packet(get_sender!(self.source), req),
            x if header_type!(x, IsoPacket) => handlers.handle_iso_packet(get_sender!(self.source), req),
            x if header_type!(x, IntPacket) => handlers.handle_int_packet(get_sender!(self.source), req),
            x if header_type!(x, BufferedBulkPacket) => {
                handlers.handle_buffered_bulk_packet(get_sender!(self.source), req)
            }

            // Other requests
            _ => (0, vec![]),
        };

        out
    }


    pub fn process_state_change(&mut self, state: ParserState) -> bool {

        // This is straightforward except for 1 cases:
        //
        // if self.state U state == IfaceInfo U EpInfo => self.state = Informed

        if (self.state == ParserState::IfaceReceived && state == ParserState::EpReceived) ||
           (self.state == ParserState::EpReceived && state == ParserState::IfaceReceived) {

            self.state = ParserState::Informed;
            return true;

        } else if self.state < state {

            self.state = state;
            return true;

        }

        false
    }
}


pub fn has_cap(caps: &[u32], cap: usize) -> bool {

    assert!(cap / 32 < usbr::CAPS_SIZE,
            "[E058-Parser] request for out of bounds cap {}",
            cap);

    if caps[cap / 32] & (1 << (cap % 32)) != 0 {
        return true;
    }

    false
}

pub fn set_cap(caps: &mut [u32], cap: usize) {

    assert!(cap / 32 < usbr::CAPS_SIZE,
            "[E059-Parser] request for out of bounds cap {}",
            cap);
    caps[cap / 32] |= 1 << (cap % 32);
}

pub fn del_cap(caps: &mut [u32], cap: usize) {

    if has_cap(caps, cap) {
        caps[cap / 32] ^= 1 << (cap % 32);
    }
}


pub fn verify_caps(caps: &[u32], desc: &str) -> bool {

    // We are not backwards compatible. All endpoints must have all capabilities.
    // The exception are filter, and disconnect ack (which only the host has)

    if !has_cap(caps, usbr::Caps::BulkStreams as usize) ||
       !has_cap(caps, usbr::Caps::ConnectDeviceVersion as usize) ||
       !has_cap(caps, usbr::Caps::EpInfoMaxPacketSize as usize) ||
       !has_cap(caps, usbr::Caps::Cap64BitsIds as usize) ||
       !has_cap(caps, usbr::Caps::Cap32BitsBulkLength as usize) ||
       !has_cap(caps, usbr::Caps::BulkReceiving as usize) {

        error!("[E060-Parser] {} caps does not contain all necessary capabilities",
               desc);
        return false;
    }

    true
}

// Returns true if all characters as ascii (not counting extended ascii)
pub fn is_ascii(x: &[u8]) -> bool {

    for i in x {
        if *i == 0 {
            // this terminates the string.
            return true;
        }

        if *i > 0x7F {
            return false;
        }
    }

    true
}
