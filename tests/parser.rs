extern crate cinch;
extern crate rand;
extern crate byteorder;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::collections::HashMap;
use std::mem;
use std::io::{BufReader, Cursor};
use std::sync::mpsc;


use byteorder::{LittleEndian, WriteBytesExt};

use cinch::parser;
use cinch::parser::usbr;

#[allow(dead_code)]
struct FakeHandler {
    id: u8,
}

impl parser::HasHandlers for FakeHandler {}

macro_rules! wrap_reader {
    ($source:expr) => {{
        let c = Cursor::new($source);
        let buf = BufReader::new(c);
        buf
    }}
}


macro_rules! redir_header {
    ($h_type:ident, $header:ident, $id:expr) => {
        util_redir_header(usbr::HeaderType::$h_type as u32,
                          mem::size_of::<usbr::$header>() as u32,
                          $id)
    };

    ($h_type:ident, $header:ident, $extra:expr, $id:expr) => {
        util_redir_header(usbr::HeaderType::$h_type as u32,
                          mem::size_of::<usbr::$header>() as u32 + $extra as u32,
                          $id);
    };
}

fn util_init_caps() -> [u32; 1] {

    let mut caps: [u32; 1] = [0];

    parser::set_cap(&mut caps, usbr::Caps::BulkStreams as usize);
    parser::set_cap(&mut caps, usbr::Caps::ConnectDeviceVersion as usize);
    parser::set_cap(&mut caps, usbr::Caps::EpInfoMaxPacketSize as usize);
    parser::set_cap(&mut caps, usbr::Caps::Cap64BitsIds as usize);
    parser::set_cap(&mut caps, usbr::Caps::Cap32BitsBulkLength as usize);
    parser::set_cap(&mut caps, usbr::Caps::BulkReceiving as usize);

    return caps;
}

fn util_host_parser() -> parser::Parser {

    let mut x = parser::Parser::new(parser::Source::Red);
    let caps = util_init_caps();

    x.init("1", &caps);

    return x;
}

fn util_guest_parser() -> parser::Parser {

    let mut x = parser::Parser::new(parser::Source::Blue);
    let caps = util_init_caps();

    x.init("1", &caps);

    return x;
}


fn util_header_type(host: bool, send: bool) -> HashMap<u32, usize> {

    let for_red = (host && !send) || (!host && send);

    let mut res = HashMap::new();


    // Below sizes are based on the protocol spec (see usb-redir-proto.txt)

    res.insert(usbr::HeaderType::Hello as u32, 64 as usize);
    res.insert(usbr::HeaderType::ControlPacket as u32, 10);
    res.insert(usbr::HeaderType::BulkPacket as u32, 10);
    res.insert(usbr::HeaderType::IsoPacket as u32, 4);
    res.insert(usbr::HeaderType::IntPacket as u32, 4);


    if for_red {
        // received by host (red machine)

        res.insert(usbr::HeaderType::Reset as u32, 0);
        res.insert(usbr::HeaderType::SetConf as u32, 1);
        res.insert(usbr::HeaderType::GetConf as u32, 0);
        res.insert(usbr::HeaderType::SetAltSetting as u32, 2);
        res.insert(usbr::HeaderType::GetAltSetting as u32, 1);
        res.insert(usbr::HeaderType::StartIsoStream as u32, 3);
        res.insert(usbr::HeaderType::StopIsoStream as u32, 1);
        res.insert(usbr::HeaderType::StartIntReceiving as u32, 1);
        res.insert(usbr::HeaderType::StopIntReceiving as u32, 1);
        res.insert(usbr::HeaderType::AllocBulkStreams as u32, 8);
        res.insert(usbr::HeaderType::FreeBulkStreams as u32, 4);
        res.insert(usbr::HeaderType::CancelDataPacket as u32, 0);
        res.insert(usbr::HeaderType::DeviceDisconnectAck as u32, 0);
        res.insert(usbr::HeaderType::StartBulkReceiving as u32, 10);
        res.insert(usbr::HeaderType::StopBulkReceiving as u32, 5);

    } else {
        // received by blue

        res.insert(usbr::HeaderType::DeviceConnect as u32, 10);
        res.insert(usbr::HeaderType::DeviceDisconnect as u32, 0);
        res.insert(usbr::HeaderType::InterfaceInfo as u32, 4 + (32 * 4));
        res.insert(usbr::HeaderType::EpInfo as u32, (32 * 3) + (32 * 2) + (32 * 4));
        res.insert(usbr::HeaderType::ConfStatus as u32, 2);
        res.insert(usbr::HeaderType::AltSettingStatus as u32, 3);
        res.insert(usbr::HeaderType::IsoStreamStatus as u32, 2);
        res.insert(usbr::HeaderType::IntReceivingStatus as u32, 2);
        res.insert(usbr::HeaderType::BulkStreamsStatus as u32, 9);
        res.insert(usbr::HeaderType::BulkReceivingStatus as u32, 6);
        res.insert(usbr::HeaderType::BufferedBulkPacket as u32, 10);
    }

    return res;
}

fn util_header_type_all() -> HashMap<u32, usize> {

    let mut res = HashMap::new();

    res.insert(usbr::HeaderType::Hello as u32, 64 as usize);
    res.insert(usbr::HeaderType::DeviceConnect as u32, 10);
    res.insert(usbr::HeaderType::DeviceDisconnect as u32, 0);
    res.insert(usbr::HeaderType::Reset as u32, 0);
    res.insert(usbr::HeaderType::InterfaceInfo as u32, 4 + (32 * 4));
    res.insert(usbr::HeaderType::EpInfo as u32, (32 * 3) + (32 * 2) + (32 * 4));
    res.insert(usbr::HeaderType::SetConf as u32, 1);
    res.insert(usbr::HeaderType::GetConf as u32, 0);
    res.insert(usbr::HeaderType::ConfStatus as u32, 2);
    res.insert(usbr::HeaderType::SetAltSetting as u32, 2);
    res.insert(usbr::HeaderType::GetAltSetting as u32, 1);
    res.insert(usbr::HeaderType::AltSettingStatus as u32, 3);
    res.insert(usbr::HeaderType::StartIsoStream as u32, 3);
    res.insert(usbr::HeaderType::StopIsoStream as u32, 1);
    res.insert(usbr::HeaderType::IsoStreamStatus as u32, 2);
    res.insert(usbr::HeaderType::StartIntReceiving as u32, 1);
    res.insert(usbr::HeaderType::StopIntReceiving as u32, 1);
    res.insert(usbr::HeaderType::IntReceivingStatus as u32, 2);
    res.insert(usbr::HeaderType::AllocBulkStreams as u32, 8);
    res.insert(usbr::HeaderType::FreeBulkStreams as u32, 4);
    res.insert(usbr::HeaderType::BulkStreamsStatus as u32, 9);
    res.insert(usbr::HeaderType::CancelDataPacket as u32, 0);
    res.insert(usbr::HeaderType::DeviceDisconnectAck as u32, 0);
    res.insert(usbr::HeaderType::StartBulkReceiving as u32, 10);
    res.insert(usbr::HeaderType::StopBulkReceiving as u32, 5);
    res.insert(usbr::HeaderType::BulkReceivingStatus as u32, 6);
    res.insert(usbr::HeaderType::ControlPacket as u32, 10);
    res.insert(usbr::HeaderType::BulkPacket as u32, 10);
    res.insert(usbr::HeaderType::IsoPacket as u32, 4);
    res.insert(usbr::HeaderType::IntPacket as u32, 4);
    res.insert(usbr::HeaderType::BufferedBulkPacket as u32, 10);

    return res;
}

fn util_host_parser_with_hello() -> parser::Parser {
    let mut x = util_host_parser();

    let caps: [u32; 1] = util_init_caps();
    let version: &[u8; 7] = b"1.4.2.5";

    let h = redir_header!(Hello, HelloHeader, caps.len() * 4, 1);
    let (h_type, data) = util_hello_header(&caps, version);

    let mut req: parser::Request = parser::Request {
        header: [0; 16],
        type_header: h_type,
        data: data,
    };
    req.header[0..h.len()].clone_from_slice(&h);

    x.handle_hello(&req);
    return x;
}


fn util_guest_parser_with_hello() -> parser::Parser {
    let mut x = util_guest_parser();

    let caps: [u32; 1] = util_init_caps();
    let version: &[u8; 7] = b"1.4.2.5";

    let h = redir_header!(Hello, HelloHeader, caps.len() * 4, 1);
    let (h_type, data) = util_hello_header(&caps, version);

    let mut req: parser::Request = parser::Request {
        header: [0; 16],
        type_header: h_type,
        data: data,
    };
    req.header[0..h.len()].clone_from_slice(&h);


    x.handle_hello(&req);
    return x;
}


fn util_redir_header(h_type: u32, len: u32, id: u64) -> Vec<u8> {

    let mut buf: Vec<u8> = vec![];

    buf.write_u32::<LittleEndian>(h_type).unwrap();
    buf.write_u32::<LittleEndian>(len).unwrap();
    buf.write_u64::<LittleEndian>(id).unwrap();

    return buf;
}


fn util_hello_header(caps: &[u32], version: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut h_type: Vec<u8> = vec![0; 64];
    assert!(version.len() <= h_type.len());

    h_type[0..version.len()].clone_from_slice(version);

    let mut data: Vec<u8> = vec![];

    for i in caps {
        data.write_u32::<LittleEndian>(*i).unwrap();
    }

    return (h_type, data);
}

// same structure as control header
fn util_connect_header() -> Vec<u8> {

    let mut buf: Vec<u8> = vec![];

    buf.push(1);
    buf.push(2);
    buf.push(3);
    buf.push(4);
    buf.write_u16::<LittleEndian>(5).unwrap();
    buf.write_u16::<LittleEndian>(6).unwrap();
    buf.write_u16::<LittleEndian>(7).unwrap();

    return buf;
}


fn util_interface_header(count: u32) -> Vec<u8> {

    let mut buf: Vec<u8> = vec![];

    buf.write_u32::<LittleEndian>(count).unwrap();
    buf.extend(&[1u8; 32]); // interface
    buf.extend(&[2u8; 32]); // class
    buf.extend(&[3u8; 32]); // subclass
    buf.extend(&[4u8; 32]); // proto

    return buf;
}

fn util_ep_header() -> Vec<u8> {

    let mut buf: Vec<u8> = vec![];

    buf.extend(&[1u8; 32]); // ep_type
    buf.extend(&[2u8; 32]); // interval
    buf.extend(&[3u8; 32]); // interface

    for i in 0..32 {
        buf.write_u16::<LittleEndian>(i).unwrap(); // max packet
    }

    for i in 0..32 {
        buf.write_u32::<LittleEndian>(i).unwrap(); // max streams
    }

    return buf;
}




#[test]
fn set_cap() {
    let mut caps: [u32; 1] = [0];
    let mut total: u32 = 0;

    for _ in 0..100 {

        let x = rand::random::<usize>() % 32;
        parser::set_cap(&mut caps, x);

        total = total | (1 << x);
        assert_eq!(caps[0], total);
    }
}

#[test]
#[should_panic]
fn set_cap_should_panic() {

    let mut caps: [u32; 2] = [0, 0];
    parser::set_cap(&mut caps, 32);
}


#[test]
fn has_cap() {

    let caps: [u32; 1] = [2147483659];

    assert!(parser::has_cap(&caps, 0));
    assert!(parser::has_cap(&caps, 1));
    assert!(parser::has_cap(&caps, 3));
    assert!(parser::has_cap(&caps, 31));

    assert!(!parser::has_cap(&caps, 22));
    assert!(!parser::has_cap(&caps, 2));
    assert!(!parser::has_cap(&caps, 16));
    assert!(!parser::has_cap(&caps, 30));
}

#[test]
#[should_panic]
fn has_cap_should_panic() {
    let caps: [u32; 2] = [2147483659, 0];
    parser::has_cap(&caps, 33);
}


#[test]
fn init() {

    let mut x = parser::Parser::new(parser::Source::Red);
    let caps = util_init_caps();

    // below calls init as a host (red machine) the "host 1" string is arbitrary
    x.init("host 1", &caps);

    assert_eq!(x.source, parser::Source::Red);
    assert!(!parser::has_cap(&x.our_caps, parser::usbr::Caps::DeviceDisconnectAck as usize));

    // below calls init as a guest (blue machine) instead of host
    let mut x = parser::Parser::new(parser::Source::Blue);
    x.init("guest 1", &caps);

    assert_eq!(x.source, parser::Source::Blue);
    assert!(parser::has_cap(&x.our_caps, parser::usbr::Caps::DeviceDisconnectAck as usize));
}

#[test]
#[should_panic]
fn init_should_panic() {

    let mut x = parser::Parser::new(parser::Source::Red);
    let caps: [u32; 1] = [423]; // reason for panic is that it is missing the default required caps

    x.init("1.2.3", &caps);
}


#[test]
fn init_utils() {

    // Below is a test for util functions

    let x = util_host_parser();
    assert_eq!(x.source, parser::Source::Red);
    assert!(!parser::has_cap(&x.our_caps, parser::usbr::Caps::DeviceDisconnectAck as usize));


    let x = util_guest_parser();
    assert_eq!(x.source, parser::Source::Blue);
    assert!(parser::has_cap(&x.our_caps, parser::usbr::Caps::DeviceDisconnectAck as usize));
}


#[test]
fn handle_hello() {

    // parser is for host and has correct version and size 1 capabilities
    let mut x = util_host_parser();
    let caps: [u32; 1] = util_init_caps();
    let version: &[u8; 7] = b"1.4.2.5";

    let h = redir_header!(Hello, HelloHeader, caps.len() * 4, 1);
    let (h_type, data) = util_hello_header(&caps, version);

    let mut req: parser::Request = parser::Request {
        header: [0; 16],
        type_header: h_type,
        data: data,
    };

    assert!(h.len() <= req.header.len());
    req.header[0..h.len()].clone_from_slice(&h);

    x.handle_hello(&req);

    assert_eq!(caps, x.peer_caps);
    assert!(x.state >= parser::ParserState::HelloR);

}


#[test]
#[should_panic]
fn handle_hello_should_panic() {

    // parser is for host and has correct version and size 1 capabilities
    let mut x = util_host_parser();
    let caps: [u32; 1] = util_init_caps();
    let version: [u8; 4] = [127, 128, 129, 130];

    let h = redir_header!(Hello, HelloHeader, caps.len() * 4, 1);
    let (h_type, data) = util_hello_header(&caps, &version);


    let mut req: parser::Request = parser::Request {
        header: [0; 16],
        type_header: h_type,
        data: data,
    };
    req.header[0..h.len()].clone_from_slice(&h);

    x.handle_hello(&req);
}


#[test]
#[should_panic]
fn handle_hello_should_panic2() {

    // parser is for host, has correct version and capabilities > CAP_SIZE
    // parser should bitch at.
    let mut x = util_host_parser();
    let caps: [u32; 1] = util_init_caps();
    let version: &[u8; 7] = b"1.4.2.5";

    let h = redir_header!(Hello, HelloHeader, caps.len() * 4, 1);

    let mut bad_caps: Vec<u32> = vec![];
    bad_caps.push(caps[0]);
    for _ in 0..100 {
        let r = rand::random::<u32>();
        bad_caps.push(r);
    }

    let (h_type, data) = util_hello_header(&bad_caps, version);

    let mut req: parser::Request = parser::Request {
        header: [0; 16],
        type_header: h_type,
        data: data,
    };

    assert!(h.len() <= req.header.len());

    req.header[0..h.len()].clone_from_slice(&h);

    x.handle_hello(&req);

    assert_eq!(caps, x.peer_caps);
    assert!(x.state == parser::ParserState::HelloR);
}


#[test]
#[should_panic]
fn handle_hello_should_panic3() {

    // Should be no different for guest parser


    let mut x = util_guest_parser();
    let caps: [u32; 1] = util_init_caps();
    let version: &[u8; 7] = b"1.4.2.5";

    let h = redir_header!(Hello, HelloHeader, caps.len() * 4, 1);

    let mut bad_caps: Vec<u32> = vec![];
    bad_caps.push(caps[0]);
    for _ in 0..100 {
        let r = rand::random::<u32>();
        bad_caps.push(r);
    }

    let (h_type, data) = util_hello_header(&bad_caps, version);

    let mut req: parser::Request = parser::Request {
        header: [0; 16],
        type_header: h_type,
        data: data,
    };

    assert!(h.len() <= req.header.len());
    req.header[0..h.len()].clone_from_slice(&h);

    x.handle_hello(&req);

    assert_eq!(caps, x.peer_caps);
    assert!(x.state == parser::ParserState::HelloR);
}





#[test]
fn get_type_header_len() {

    let host: bool = true; // true = red, false = blue
    let send: bool = true; // true = sending, false = receiving

    let all: HashMap<u32, usize> = util_header_type_all();
    let host_receiving: HashMap<u32, usize> = util_header_type(host, !send);
    let host_sending: HashMap<u32, usize> = util_header_type(host, send);



    let x = util_host_parser();
    let y = util_guest_parser();

    for (h_type, len) in all.iter() {

        // Case 1.1 host is receiving
        if host_receiving.contains_key(h_type) {
            assert_eq!(x.get_type_header_len(*h_type, !send).unwrap(), *len);
        } else {
            assert!(x.get_type_header_len(*h_type, !send).is_err());
        }

        // Case 1.2 host is sending
        if host_sending.contains_key(h_type) {
            assert_eq!(x.get_type_header_len(*h_type, send).unwrap(), *len);
        } else {
            assert!(x.get_type_header_len(*h_type, send).is_err());
        }

        // Case 2.1 guest is receiving (i.e., host is sending)
        if host_sending.contains_key(h_type) {
            assert_eq!(y.get_type_header_len(*h_type, !send).unwrap(), *len);
        } else {
            assert!(y.get_type_header_len(*h_type, !send).is_err());
        }

        // Case 2.2 guest is sending (i.e., host is receiving)
        if host_receiving.contains_key(h_type) {
            assert_eq!(y.get_type_header_len(*h_type, send).unwrap(), *len);
        } else {
            assert!(y.get_type_header_len(*h_type, send).is_err());
        }
    }
}


#[test]
fn pull_next_request() {

    let _ = env_logger::init();
    let mut x = util_guest_parser_with_hello();

    // pretend guest has already received EpInfo and IfaceInfo
    x.state = parser::ParserState::Informed;

    // generate device connect request
    let mut h = redir_header!(DeviceConnect, ConnectHeader, 1);
    h.extend(&util_connect_header());

    // generate interface info request
    h.extend(&redir_header!(InterfaceInfo, InterfaceInfoHeader, 2));
    h.extend(&util_interface_header(1));

    // generate another interface info request
    h.extend(&redir_header!(InterfaceInfo, InterfaceInfoHeader, 3));
    h.extend(&util_interface_header(32));

    let mut buf = wrap_reader!(h);

    let req: parser::Request = x.pull_next_request(&mut buf);

    assert_eq!(req.type_header.len(), 10);
    assert_eq!(req.get_total_len(), 10);
    assert_eq!(req.get_id(), 1);

    let req: parser::Request = x.pull_next_request(&mut buf);

    assert_eq!(req.type_header.len(), 132);
    assert_eq!(req.get_total_len(), 132);
    assert_eq!(req.get_id(), 2);

    let req: parser::Request = x.pull_next_request(&mut buf);

    assert_eq!(req.type_header.len(), 132);
    assert_eq!(req.get_total_len(), 132);
    assert_eq!(req.get_id(), 3);
}



#[test]
fn pull_next_request_2() {

    let _ = env_logger::init();
    let mut x = util_guest_parser_with_hello();

    // pretend guest has already received EpInfo and IfaceInfo
    x.state = parser::ParserState::Informed;

    // generate device connect request
    let mut h = redir_header!(DeviceConnect, ConnectHeader, 1);
    h.extend(&util_connect_header());

    // generate an invalid interface info request. This should trigger a (recoverable) error
    // in verify_type_header()
    h.extend(&redir_header!(InterfaceInfo, InterfaceInfoHeader, 2));
    h.extend(&util_interface_header(34));


    // generate an invalid control packet. This should trigger a (recoverable) error
    // in verify_type_header()
    h.extend(&redir_header!(ControlPacket, ControlPacketHeader, 3));
    h.extend(&util_connect_header());



    // generate another interface info request
    h.extend(&redir_header!(InterfaceInfo, InterfaceInfoHeader, 4));
    h.extend(&util_interface_header(32));

    let mut buf = wrap_reader!(h);

    let req: parser::Request = x.pull_next_request(&mut buf);

    assert_eq!(req.type_header.len(), 10);
    assert_eq!(req.get_total_len(), 10);
    assert_eq!(req.get_id(), 1);

    let req: parser::Request = x.pull_next_request(&mut buf);

    assert_eq!(req.type_header.len(), 132);
    assert_eq!(req.get_total_len(), 132);
    assert_eq!(req.get_id(), 4);
}


#[test]
#[should_panic]
fn pull_next_request_should_panic() {

    let _ = env_logger::init();
    let mut x = util_host_parser_with_hello();

    // pretend guest has already received EpInfo and IfaceInfo
    x.state = parser::ParserState::Informed;

    // This is an invalid type header because of the source (host + connect).
    let mut h = redir_header!(DeviceConnect, ConnectHeader, 1);
    h.extend(&util_connect_header());


    let mut buf = wrap_reader!(h);

    // This should trigger an error in get_type_header_len(). Since we don't have
    // any other packets, there is no next request, so it will panic.
    let _ = x.pull_next_request(&mut buf);
}


#[test]
#[should_panic]
fn pull_next_request_should_panic_2() {

    let _ = env_logger::init();
    let x = util_guest_parser_with_hello();

    // This is an invalid type header because of the state (guest has not
    // received Ep Info and Iface Info)
    let mut h = redir_header!(DeviceConnect, ConnectHeader, 1);
    h.extend(&util_connect_header());


    let mut buf = wrap_reader!(h);

    // This should trigger an error in get_type_header_len(). Since we don't have
    // any other packets, there is no next request, so it will panic.
    let _ = x.pull_next_request(&mut buf);
}


#[test]
#[should_panic]
fn pull_next_request_should_panic_3() {

    let _ = env_logger::init();
    let mut x = util_guest_parser_with_hello();


    // pretend guest has already received EpInfo and IfaceInfo
    x.state = parser::ParserState::Informed;

    // generate device connect request
    let mut h = redir_header!(DeviceConnect, ConnectHeader, 1);
    h.extend(&util_connect_header());

    // generate a valid interface info request
    h.extend(&redir_header!(InterfaceInfo, InterfaceInfoHeader, 3));
    h.extend(&util_interface_header(32));

    let mut buf = wrap_reader!(h);

    let req: parser::Request = x.pull_next_request(&mut buf);

    assert_eq!(req.type_header.len(), 10);
    assert_eq!(req.get_total_len(), 10);
    assert_eq!(req.get_id(), 1);


    // roll back state into an illegal state
    x.state = parser::ParserState::HelloR;

    // Below should panic because state of parser should not be correct.
    let _: parser::Request = x.pull_next_request(&mut buf);
}



#[test]
fn process_request() {

    let _ = env_logger::init();
    let mut x = util_guest_parser(); // no hello
    let handler: FakeHandler = FakeHandler { id: 0 };
    let caps = util_init_caps();

    // Let's create a channel
    let (tx, rx) = mpsc::channel();

    assert_eq!(x.state, parser::ParserState::Init);

    // Start with hello header,
    let version: &[u8; 5] = b"1.2.3";
    let mut h = redir_header!(Hello, HelloHeader, caps.len() * 4, 1);
    for _ in 0..4 {
        h.pop();
    }
    let (h_type, data) = util_hello_header(&caps, version);

    h.extend(&h_type);
    h.extend(&data);

    let mut buf = wrap_reader!(h);

    let req: parser::Request = x.pull_next_request(&mut buf);
    let _: Vec<parser::Request> = x.process_request(&handler, req, &tx);

    assert_eq!(rx.try_recv().err(), Some(mpsc::TryRecvError::Empty));
    assert_eq!(x.state, parser::ParserState::HelloR);
}


#[test]
fn state_promotion() {

    let _ = env_logger::init();
    let mut x = util_guest_parser(); // no hello
    let handler: FakeHandler = FakeHandler { id: 0 };
    let caps = util_init_caps();

    // Let's create a channel
    let (tx, rx) = mpsc::channel();

    assert_eq!(x.state, parser::ParserState::Init);

    // Start with hello header,
    let version: &[u8; 5] = b"1.2.3";
    let mut h = redir_header!(Hello, HelloHeader, caps.len() * 4, 1);
    let (h_type, data) = util_hello_header(&caps, version);

    for _ in 0..4 {
        h.pop();
    }

    h.extend(&h_type);
    h.extend(&data);

    h.extend(&redir_header!(EpInfo, EpInfoHeader, 2));
    h.extend(&util_ep_header());

    h.extend(&redir_header!(InterfaceInfo, InterfaceInfoHeader, 3));
    h.extend(&util_interface_header(4));

    let mut buf = wrap_reader!(h);

    let req: parser::Request = x.pull_next_request(&mut buf);
    let _ = x.process_request(&handler, req, &tx);

    assert_eq!(rx.try_recv().err(), Some(mpsc::TryRecvError::Empty));
    assert_eq!(x.state, parser::ParserState::HelloR);

    x.process_state_change(parser::ParserState::Hello);

    let req: parser::Request = x.pull_next_request(&mut buf);
    let _ = x.process_request(&handler, req, &tx);

    assert_eq!(x.state, parser::ParserState::EpReceived);
    assert_eq!(rx.try_recv().unwrap(), parser::ParserState::EpReceived);


    let req: parser::Request = x.pull_next_request(&mut buf);
    let _ = x.process_request(&handler, req, &tx);

    assert_eq!(x.state, parser::ParserState::Informed);
    assert_eq!(rx.try_recv().unwrap(), parser::ParserState::IfaceReceived);
}
