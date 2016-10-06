#![allow(unused_variables)]

use parser;
// use parser::usbr;
use parser::{Request, Source};

#[derive(Default)]
pub struct Reset;

// #[cfg_attr(rustfmt, rustfmt_skip)]
// const RESET_HEADER: [u8; usbr::REDIR_HEADER_SIZE] = [0, 0, 0, usbr::HeaderType::DeviceDisconnect as u8,
//                                                    0, 0, 0, 0,
//                                                     0, 0, 0, 0, 0, 0, 0, 0];

impl Reset {
    pub fn new() -> Reset {
        Reset
    }

    fn reset(&self) -> (u8, Vec<Request>) {

        panic!("Ending connection because a packet did not pass all checks");

        //        (0, vec![Request {
        //                 header: RESET_HEADER,
        //                 type_header: vec![],
        //                 data: vec![],
        //                 }])
    }
}


impl parser::HasHandlers for Reset {
    // I'm sure there is a macro that could generate the stuff below, but whatever.

    fn handle_hello(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_connect(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_disconnect(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_disconnect_ack(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_reset(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_cancel_data_packet(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_interface_info(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_ep_info(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_get_conf(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_set_conf(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_conf_status(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_get_alt_setting(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_set_alt_setting(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_alt_setting_status(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_start_iso_stream(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_stop_iso_stream(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_iso_stream_status(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_start_int_receiving(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_stop_int_receiving(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_int_receiving_status(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_alloc_bulk_streams(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_free_bulk_streams(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_bulk_streams_status(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_start_bulk_receiving(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_stop_bulk_receiving(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_bulk_receiving_status(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }


    // Data packets

    fn handle_control_packet(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_bulk_packet(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_int_packet(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_iso_packet(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }

    fn handle_buffered_bulk_packet(&self, source: Source, _: Request) -> (u8, Vec<Request>) {
        self.reset()
    }
}
