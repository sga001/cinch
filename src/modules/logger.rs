use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::sync::RwLock;
use std::io::BufWriter;

use parser;
use parser::{Request, Source};


macro_rules! log_request {
    ($logger:ident, $source:expr, $req:expr, $ty:expr) => {{
        let mut file = $logger.f.write().unwrap();

        file.write(format!("[Start Cinch log. Type: {}, Source: {:?}]", $ty, $source).as_bytes()).unwrap();
        file.write(&$req.header).unwrap();
        file.write(&$req.type_header).unwrap();
        file.write(&$req.data).unwrap();
        file.write("[End Cinch log]".as_bytes()).unwrap();
        file.flush().unwrap();
    }}

}


pub struct Logger {
    f: RwLock<BufWriter<File>>,
}

impl Logger {
    pub fn new(path_name: &str) -> Logger {

        let path = Path::new(path_name);

        let file = match File::create(&path) {
            Ok(file) => RwLock::new(BufWriter::new(file)),
            Err(e) => {
                panic!("[Logger] Could not create {}: {}",
                       path.display(),
                       Error::description(&e))
            }
        };

        Logger { f: file }
    }
}




impl parser::HasHandlers for Logger {
    fn handle_hello(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "hello");
        (0, vec![req])
    }

    fn handle_connect(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "connect");
        (0, vec![req])
    }

    fn handle_disconnect(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "disconnect");
        (0, vec![req])
    }

    fn handle_disconnect_ack(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "disconenct ack");
        (0, vec![req])
    }

    fn handle_reset(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "reset");
        (0, vec![req])
    }

    fn handle_cancel_data_packet(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "cancel data packet");
        (0, vec![req])
    }

    fn handle_interface_info(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "interface info");
        (0, vec![req])
    }

    fn handle_ep_info(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "ep info");
        (0, vec![req])
    }

    fn handle_get_conf(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "get conf");
        (0, vec![req])
    }

    fn handle_set_conf(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "set conf");
        (0, vec![req])
    }

    fn handle_conf_status(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "conf status");
        (0, vec![req])
    }

    fn handle_get_alt_setting(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "get alt setting");
        (0, vec![req])
    }

    fn handle_set_alt_setting(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "set alt setting");
        (0, vec![req])
    }

    fn handle_alt_setting_status(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "alt setting status");
        (0, vec![req])
    }

    fn handle_start_iso_stream(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "start iso stream");
        (0, vec![req])
    }

    fn handle_stop_iso_stream(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "stop iso stream");
        (0, vec![req])
    }

    fn handle_iso_stream_status(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "iso stream status");
        (0, vec![req])
    }

    fn handle_start_int_receiving(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "start int receiving");
        (0, vec![req])
    }

    fn handle_stop_int_receiving(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "stop int receiving");
        (0, vec![req])
    }

    fn handle_int_receiving_status(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "int receiving status");
        (0, vec![req])
    }

    fn handle_alloc_bulk_streams(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "alloc bulk streams");
        (0, vec![req])
    }

    fn handle_free_bulk_streams(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "free bulk streams");
        (0, vec![req])
    }

    fn handle_bulk_streams_status(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "bulk streams status");
        (0, vec![req])
    }

    fn handle_start_bulk_receiving(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "start bulk receiving");
        (0, vec![req])
    }

    fn handle_stop_bulk_receiving(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "stop bulk receving");
        (0, vec![req])
    }

    fn handle_bulk_receiving_status(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "bulk receiving status");
        (0, vec![req])
    }


    // Data packets

    fn handle_control_packet(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "control packet");
        (0, vec![req])
    }

    fn handle_bulk_packet(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "bulk packet");
        (0, vec![req])
    }

    fn handle_int_packet(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "int packet");
        (0, vec![req])
    }

    fn handle_iso_packet(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "iso packet");
        (0, vec![req])
    }

    fn handle_buffered_bulk_packet(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
        log_request!(self, source, req, "buffered bulk packet");
        (0, vec![req])
    }
}
