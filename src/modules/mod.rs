pub mod reset;
pub mod null;
pub mod control_checks;
pub mod logger;
pub mod patcher;

use std::sync::Arc;
use std::collections::HashMap;

use parser;
use parser::{Request, Source};

pub struct Modules {
    nonterminals: Vec<HashMap<u8, Arc<parser::HasHandlers>>>,
    terminal: HashMap<u8, Arc<parser::HasHandlers>>,
}

macro_rules! traverse_modules {
    ($self_:ident, $source:expr, $req:ident, $h_type:ident) => {{

        let mut params: (u8, Vec<Request>) = (0, vec![]); // (port, out) tuple

        for module in &$self_.nonterminals {

            params = match module.get(&params.0) {
                Some(v) => v.$h_type($source, $req),
                None => panic!("Invalid port for nonterminal module"),
            };

            // By requirement, intermediary modules cannot produce more than 1 response
            assert_eq!(params.1.len(), 1);
            $req = match params.1.pop() {
                Some(v) => v,
                None => panic!("nonterminal module returned nothing"),
            };
        }

        assert!((params.0 as usize) < $self_.terminal.len());
        match $self_.terminal.get(&params.0) {
            Some(v) => v.$h_type($source, $req),
            None => panic!("terminal module did not return anything"),
        }
    }}
}

impl Modules {
    pub fn new() -> Modules {
        Modules { nonterminals: vec![], terminal: HashMap::new() }
    }

    pub fn add_terminal(&mut self, port: u8, term: Arc<parser::HasHandlers>) {
        self.terminal.insert(port, term);
    }

    pub fn add_nonterminal(&mut self, idx: usize, port: u8, nonterm: Arc<parser::HasHandlers>) {

        assert!(idx <= self.nonterminals.len());

        if idx == self.nonterminals.len() {
            self.nonterminals.push(HashMap::new());
        }

        self.nonterminals[idx].insert(port, nonterm);
    }
}

unsafe impl Send for Modules {}


#[allow(unused_variables)]
impl parser::HasHandlers for Modules {
    fn handle_hello(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_hello)
    }

    fn handle_connect(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_connect)
    }

    fn handle_disconnect(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_disconnect)
    }

    fn handle_disconnect_ack(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_disconnect_ack)
    }

    fn handle_reset(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_reset)
    }

    fn handle_cancel_data_packet(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_cancel_data_packet)
    }

    fn handle_interface_info(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_interface_info)
    }

    fn handle_ep_info(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_ep_info)
    }

    fn handle_get_conf(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_get_conf)
    }

    fn handle_set_conf(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_set_conf)
    }

    fn handle_conf_status(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_conf_status)
    }

    fn handle_get_alt_setting(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_get_alt_setting)
    }

    fn handle_set_alt_setting(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_set_alt_setting)
    }

    fn handle_alt_setting_status(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_alt_setting_status)
    }

    fn handle_start_iso_stream(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_start_iso_stream)
    }

    fn handle_stop_iso_stream(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_stop_iso_stream)
    }

    fn handle_iso_stream_status(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_iso_stream_status)
    }

    fn handle_start_int_receiving(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_start_int_receiving)
    }

    fn handle_stop_int_receiving(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_stop_int_receiving)
    }

    fn handle_int_receiving_status(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_int_receiving_status)
    }

    fn handle_alloc_bulk_streams(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_alloc_bulk_streams)
    }

    fn handle_free_bulk_streams(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_free_bulk_streams)
    }

    fn handle_bulk_streams_status(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_bulk_streams_status)
    }

    fn handle_start_bulk_receiving(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_start_bulk_receiving)
    }

    fn handle_stop_bulk_receiving(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_stop_bulk_receiving)
    }

    fn handle_bulk_receiving_status(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_bulk_receiving_status)
    }


    // Data packets

    fn handle_control_packet(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_control_packet)
    }

    fn handle_bulk_packet(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_bulk_packet)
    }

    fn handle_int_packet(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_int_packet)
    }

    fn handle_iso_packet(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_iso_packet)
    }

    fn handle_buffered_bulk_packet(&self, source: Source, mut req: Request) -> (u8, Vec<Request>) {
        traverse_modules!(self, source, req, handle_buffered_bulk_packet)
    }
}
