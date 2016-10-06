use std::io::prelude::*;
use std::fs;
use std::cell::RefCell;
use std::fs::File;
use std::collections::HashMap;
use rustc_serialize::json;
use rustc_serialize::hex::ToHex;

use parser;
use parser::usbr;
use parser::{Request, Source};


#[derive(RustcDecodable)]
struct PatchMetadata {
    p_type: String,
    vendor_id: u16,
    product_id: u16,
    request: u8,
    requesttype: u8,
    patch_id: u32, // id of this patch
    min_matches: u16, // min number of matches before patch_id is considered a match
}

#[derive(RustcDecodable)]
struct Patch {
    meta: PatchMetadata,
    data: String, // hex-encoded
}

pub struct Patcher {
    patches: Vec<Patch>,
    counts: RefCell<HashMap<u32, u16>>,
}

impl PatchMetadata {
    fn matches_descriptor(&self, header: &usbr::ControlPacketHeader) -> bool {
        self.request == header.request && self.requesttype == header.requesttype
    }
}



impl Patcher {
    pub fn new(dir_path: &str) -> Patcher {

        let mut patcher = Patcher { patches: vec![], counts: RefCell::new(HashMap::new()) };

        for entry in fs::read_dir(dir_path).unwrap() {

            let mut file = match File::open(&entry.unwrap().path()) {
                Ok(file) => file,
                Err(e) => panic!("[E000-Patcher] Could not open file {}", e),
            };

            // read file
            let mut json_line = String::new();
            file.read_to_string(&mut json_line).unwrap();

            // decode file
            let patch: Patch = json::decode(&json_line).unwrap();

            // Insert count entry
            if !patcher.counts.borrow().contains_key(&patch.meta.patch_id) {
                patcher.counts.borrow_mut().insert(patch.meta.patch_id, patch.meta.min_matches);
            }

            // Insert patch into our list
            patcher.patches.push(patch);
        }

        patcher
    }

    fn check_control_packet(&self, req: &Request) -> bool {

        let h_ptr = req.type_header.as_ptr() as *const usbr::ControlPacketHeader;
        let h: &usbr::ControlPacketHeader = unsafe { &*h_ptr };

        for patch in &self.patches {

            if patch.meta.p_type != "control" {
                continue;
            }

            if patch.meta.matches_descriptor(h) {
                // The metadata matches, time to compare the data
                let data_hex: String = req.data[..].to_hex();

                if data_hex.len() >= patch.data.len() && data_hex.contains(&patch.data) {

                    let mut count: u16 = *self.counts.borrow().get(&patch.meta.patch_id).unwrap();
                    count -= 1;

                    if count == 0 {
                        error!("[E001-Patcher] matched at least {} signatures",
                               patch.meta.min_matches);
                        return false;
                    }

                    self.counts.borrow_mut().insert(patch.meta.patch_id, count);
                }
            }
        }

        true
    }


    fn check_bulk_packet(&self, req: &Request) -> bool {

        for patch in &self.patches {

            if patch.meta.p_type != "bulk" {
                continue;
            }

            // The metadata matches, time to compare the data
            let data_hex: String = req.data[..].to_hex();

            if data_hex.len() >= patch.data.len() && data_hex.contains(&patch.data) {

                let mut count: u16 = *self.counts.borrow().get(&patch.meta.patch_id).unwrap();
                count -= 1;

                if count == 0 {
                    error!("[E002-Patcher] matched at least {} signatures",
                           patch.meta.min_matches);
                    return false;
                }

                self.counts.borrow_mut().insert(patch.meta.patch_id, count);

            }
        }

        true
    }


    fn check_connect(&self, req: &Request) -> bool {

        let h_ptr = req.type_header.as_ptr() as *const usbr::ConnectHeader;
        let h: &usbr::ConnectHeader = unsafe { &*h_ptr };

        for patch in &self.patches {

            if patch.meta.p_type != "connect" {
                continue;
            }

            if patch.meta.vendor_id == h.vendor_id && patch.meta.product_id == h.product_id {

                error!("[E003-Patcher] malicious device found {:x}:{:x}",
                       h.vendor_id,
                       h.product_id);
                return false;
            }
        }

        true
    }
}




impl parser::HasHandlers for Patcher {
    fn handle_control_packet(&self, _: Source, req: Request) -> (u8, Vec<Request>) {
        if self.check_control_packet(&req) { (0, vec![req]) } else { (1, vec![req]) }
    }

    fn handle_bulk_packet(&self, _: Source, req: Request) -> (u8, Vec<Request>) {
        if self.check_bulk_packet(&req) { (0, vec![req]) } else { (1, vec![req]) }
    }

    fn handle_connect(&self, _: Source, req: Request) -> (u8, Vec<Request>) {
        if self.check_connect(&req) { (0, vec![req]) } else { (1, vec![req]) }
    }

    // TODO: Implement below
    //
    // fn handle_int_packet(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
    // (0, vec![req])
    // }
    //
    // fn handle_iso_packet(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
    // (0, vec![req])
    // }
    //
    // fn handle_buffered_bulk_packet(&self, source: Source, req: Request) -> (u8, Vec<Request>) {
    // (0, vec![req])
    // }
    //
}
