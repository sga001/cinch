use parser;
use parser::usbr;

struct Reset {
    reset_issued: bool,
}

#[rustfmt_skip]
const RESET_HEADER: [u8; usbr::REDIR_HEADER_SIZE] = [0, 0, 0, usbr::HeaderType::Reset as u8,
                                                     0, 0, 0, 0,
                                                     0, 0, 0, 0, 0, 0, 0, 0];

impl Reset {

    pub fn new() -> Reset {
        Reset { reset_issued: false }
    }

    fn reset(&mut self) -> Vec<parser::Request> {

        self.reset_issued = true;
        vec![parser::Request {
                 header: RESET_HEADER,
                 type_header: vec![],
                 data: vec![],
             }]
    }
}


impl parser::HasHandlers for Reset {

    // I'm sure there is a macro that could generate the stuff below, but whatever.

    fn handle_hello(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_connect(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_disconnect(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_disconnect_ack(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_reset(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_cancel_data_packet(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_interface_info(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_ep_info(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_get_conf(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_set_conf(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_conf_status(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_get_alt_setting(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_set_alt_setting(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_alt_setting_status(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_start_iso_stream(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_stop_iso_stream(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_iso_stream_status(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_start_int_receiving(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_stop_int_receiving(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_int_receiving_status(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_alloc_bulk_streams(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_free_bulk_streams(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_bulk_streams_status(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_start_bulk_receiving(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_stop_bulk_receiving(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_bulk_receiving_status(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }


    // Data packets

    fn handle_control_packet(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_bulk_packet(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_int_packet(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_iso_packet(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

    fn handle_buffered_bulk_packet(&mut self, _: parser::Request) -> Vec<parser::Request> {
        self.reset()
    }

}
