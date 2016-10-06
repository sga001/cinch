extern crate env_logger;
#[macro_use]
extern crate log;
extern crate time;
extern crate rustc_serialize;
extern crate getopts;
extern crate cinch;


// Core
use std::io::prelude::*;
use std::fs::File;
use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, BufWriter};
use std::thread;
use std::sync::mpsc; // for channel to communicate between threads
use std::sync::Arc;

// To parse configuration
use rustc_serialize::json;

// To parse runtime arguments
use getopts::Options;
use std::env;

// Usbr parsing library
use cinch::parser;

// cinch modules
use cinch::modules;
use cinch::util;

const DEFAULT_CONFIG_LINE: &'static str = "{ \"red_addr\": \"192.168.1.100:8000\",\
                                             \"cinch_addr\": \"192.168.1.7:5555\",
                                             \"log_prefix\": \"logs/trace\",
                                             \"log\": true,
                                             \"checks_active\": true,
                                             \"patch_active\": false,
                                             \"patches\": \"\",
                                             \"third_party_folder\": \"third-party-checks\"}";


pub struct CinchEndpoint<T: parser::HasHandlers, R: Read, W: Write> {
    parser: parser::Parser,

    reader: BufReader<R>, // read endpoint (e.g., from blue machine)
    writer: BufWriter<W>, // write endpoint (e.g., to red machine)

    handlers: T, // handlers for processing requests

    tx: mpsc::Sender<parser::ParserState>,
    rx: mpsc::Receiver<parser::ParserState>,
}

impl<T, R, W> CinchEndpoint<T, R, W>
    where T: parser::HasHandlers,
          R: Read,
          W: Write {
    fn new(parser: parser::Parser,
           reader: BufReader<R>,
           writer: BufWriter<W>,
           handlers: T,
           tx: mpsc::Sender<parser::ParserState>,
           rx: mpsc::Receiver<parser::ParserState>)
           -> CinchEndpoint<T, R, W> {

        CinchEndpoint {
            parser: parser,
            reader: reader,
            writer: writer,
            handlers: handlers,
            tx: tx,
            rx: rx,
        }
    }

    fn process_requests(&mut self) {

        loop {

            if self.parser.state < parser::ParserState::Connected {

                // Update parser state during the initial device configuration phase
                loop {
                    match self.rx.try_recv() {
                        Ok(v) => {
                            self.parser.process_state_change(v);
                        }
                        Err(e) if e == mpsc::TryRecvError::Empty => {
                            break;
                        }
                        Err(e) => {
                            panic!("Unknown error processing state change {:?}", e);
                        }
                    }
                }
            }

            // Pull request is blocking
            let request = self.parser.pull_next_request(&mut self.reader);
            let outputs = self.parser.process_request(&self.handlers, request, &self.tx);
            self.parser.push_outputs(&mut self.writer, outputs);
            self.writer.flush().unwrap();

        }
    }
}

fn gen_caps() -> [u32; 1] {

    let mut caps: [u32; 1] = [0];

    parser::set_cap(&mut caps, parser::usbr::Caps::BulkStreams as usize);
    parser::set_cap(&mut caps, parser::usbr::Caps::ConnectDeviceVersion as usize);
    parser::set_cap(&mut caps, parser::usbr::Caps::EpInfoMaxPacketSize as usize);
    parser::set_cap(&mut caps, parser::usbr::Caps::Cap64BitsIds as usize);
    parser::set_cap(&mut caps, parser::usbr::Caps::Cap32BitsBulkLength as usize);
    parser::set_cap(&mut caps, parser::usbr::Caps::BulkReceiving as usize);

    caps
}


fn handle_blue_machine(blue_stream: TcpStream, config: util::config::CinchConfig) {

    let red_stream = TcpStream::connect(&config.red_addr[..]).unwrap();

    println!("Connected to red machine");

    // Disable tcp_nodelay
    blue_stream.set_nodelay(true).unwrap();
    red_stream.set_nodelay(true).unwrap();


    // Clone streams to get read/write components
    let red_stream_write = red_stream.try_clone().unwrap();
    let blue_stream_write = blue_stream.try_clone().unwrap();


    // Create parsers
    let mut red_parser = parser::Parser::new(parser::Source::Blue);
    let mut blue_parser = parser::Parser::new(parser::Source::Red);

    // Init parsers
    let caps: [u32; 1] = gen_caps();

    red_parser.init("parser for red", &caps); // pretends to be guest (blue machine)
    blue_parser.init("parser for blue", &caps); //pretends to be host (red machine)

    // Create channels between parsers
    let (red_tx, red_rx) = mpsc::channel();
    let (blue_tx, blue_rx) = mpsc::channel();


    // Initialize handlers with module terminals and non-terminals

    let mut blue_handler = modules::Modules::new();
    let mut red_handler = modules::Modules::new();

    // Add the default module (null)
    let null_module = Arc::new(modules::null::Null::new());
    let null_module_clone = null_module.clone();

    blue_handler.add_terminal(0, null_module);
    red_handler.add_terminal(0, null_module_clone);


    // Add non-default modules

    let mut index: usize = 0; // index of module in the non-terminal chain

    if config.log {

        let log_name = format!("{}-{}.{}",
                               config.log_prefix,
                               time::strftime("%d-%b-%Y-%H-%M-%S", &time::now()).unwrap(),
                               "log");

        // Module for logging requests
        let logger_module = Arc::new(modules::logger::Logger::new(&log_name));
        let logger_module_clone = logger_module.clone();

        // The flow is: logger -> * -> null
        blue_handler.add_nonterminal(index, 0, logger_module);
        red_handler.add_nonterminal(index, 0, logger_module_clone);
        index += 1;
    }

    if config.checks_active {

        // Module for checking correctness of control packets
        let checks_module = Arc::new(modules::control_checks::ControlCheck::new(&config.third_party_folder));
        let checks_module_clone = checks_module.clone();

        // The flow is: * -> checks -> reset or *
        blue_handler.add_nonterminal(index, 0, checks_module);
        red_handler.add_nonterminal(index, 0, checks_module_clone);

        index += 1;
    }

    if config.patch_active {

        // Module for applying patches
        let patch_module = Arc::new(modules::patcher::Patcher::new(&config.patches[..]));
        // let patch_module_clone = patch_module.clone();

        // The flow for red endpoint is: * -> patcher -> reset or null
        // blue_handler.add_nonterminal(index, 0, patch_module);
        red_handler.add_nonterminal(index, 0, patch_module);

        if config.checks_active {

            // The flowfor red endpoint is: * -> patcher or reset -> reset or null

            // Module that resets communication
            let reset_module = Arc::new(modules::reset::Reset::new());
            // let reset_module_clone = reset_module.clone();

            // blue_handler.add_nonterminal(index, 1, reset_module);
            red_handler.add_nonterminal(index, 1, reset_module);
        }
    }

    if config.checks_active || config.patch_active {

        // Module that resets communication
        let reset_module = Arc::new(modules::reset::Reset::new());
        let reset_module_clone = reset_module.clone();


        blue_handler.add_terminal(1, reset_module);
        red_handler.add_terminal(1, reset_module_clone);
    }


    // Create endpoints

    let mut blue_end = CinchEndpoint::new(blue_parser,
                                          BufReader::new(blue_stream),
                                          BufWriter::new(red_stream_write),
                                          blue_handler,
                                          blue_tx,
                                          red_rx);


    // launch red endpoint on its own thread
    thread::spawn(move || {
        let mut red_end = CinchEndpoint::new(red_parser,
                                             BufReader::new(red_stream),
                                             BufWriter::new(blue_stream_write),
                                             red_handler,
                                             red_tx,
                                             blue_rx);
        red_end.process_requests();
    });

    blue_end.process_requests();
}


fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("c", "config", "set configuration file", "PATH");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => panic!(e.to_string()),
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let conf_line = match matches.opt_str("c") {

        None => {
            println!("Using default configuration");
            DEFAULT_CONFIG_LINE.to_string()
        }

        Some(c) => {

            println!("Using configuration file {}", c);

            let mut conf_file = match File::open(c) {
                Ok(file) => file,
                Err(e) => panic!(e.to_string()),
            };

            let mut line = String::new();
            conf_file.read_to_string(&mut line).unwrap();

            line
        }
    };

    // Parse configuration file
    let config: util::config::CinchConfig = json::decode(&conf_line).unwrap();

    // Setup logging
    env_logger::init().unwrap();

    let listener = TcpListener::bind(&config.cinch_addr[..]).unwrap();

    for stream in listener.incoming() {

        println!("Blue machine has connected");

        let config_clone = config.clone();

        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_blue_machine(stream, config_clone);
                });
            }

            Err(e) => {
                error!("Connection failed {:?}", e);
            }
        }
    }

    drop(listener);
}
