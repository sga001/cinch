use usb;
use parser::usbr;
use parser::Source;
use byteorder::{ByteOrder, LittleEndian};


pub struct BBBControlCheck {
    header: Option<usb::bbb::CommandHeader>,
    cbw: Option<usb::bbb::CommandBlockWrapper>,
}

macro_rules! parse_b_descriptor {

    (0, $data:expr) => {
        usb::bbb::CommandHeader {
            signature: LittleEndian::read_u32(&$data[..4]),
            tag: LittleEndian::read_u32(&$data[4..]),
        }
    };

    (usb::bbb::CBW_SIGN, $data:expr) => {
        {
            let mut cbw = usb::bbb::CommandBlockWrapper {
                transfer_length: LittleEndian::read_u32(&$data[..4]),
                flags: $data[4],
                cb_lun: $data[5],
                cb_length: $data[6],
                bcbw: [0; 16],
            };

            cbw.bcbw.clone_from_slice(&$data[7..]);
            cbw
        }
    };

    (usb::bbb::CSW_SIGN, $data:expr) => {
        usb::bbb::CommandStatusWrapper {
            data_residue: LittleEndian::read_u32(&$data[..4]),
            status: $data[4],
        }
    };
}

impl BBBControlCheck {
    pub fn new() -> BBBControlCheck {
        BBBControlCheck { header: None, cbw: None }
    }

    pub fn check_control_req(&self, h: &usbr::ControlPacketHeader, data: &[u8], source: Source) -> bool {

        match h.request {

            usb::bbb::RESET => {

                if h.requesttype != usb::TYPE_CLASS | usb::RECIP_INTERFACE | usb::DIR_OUT {
                    error!("[E002-BBB] Invalid request type 0x{:x}", h.requesttype);
                    return false;
                }

                if h.value != 0 || h.length != 0 {
                    error!("[E003-BBB] Invalid request value 0x{:x} or length {}",
                           h.value,
                           h.length);
                    return false;
                }

                if source == Source::Red && data.len() != 0 {
                    error!("[E004-BB] Invalid data length {}", data.len());
                    return false;
                }

                true
            }

            usb::bbb::MAX_LUN => {

                if h.requesttype != usb::TYPE_CLASS | usb::RECIP_INTERFACE | usb::DIR_IN {
                    error!("[E005-BBB] Invalid request type 0x{:x}", h.requesttype);
                    return false;
                }

                if h.value != 0 || h.length != 1 {
                    error!("[E006-BBB] Invalid value 0x{:x} or length {}", h.value, h.length);
                    return false;
                }

                if source == Source::Red {

                    if data.len() != 1 {
                        error!("[E007-BBB] Invalid data length {}", data.len());
                        return false;
                    }

                    if data[0] > 15 {
                        error!("[E008-BBB] Invalid max LUN of {}", data[0]);
                        return false;
                    }
                }

                true
            }

            _ => {
                error!("[E001-BBB] Unrecognized request type: 0x{:x}", h.request);
                false
            }
        }
    }


    pub fn check_bbb_request(&mut self, h: &usbr::ControlPacketHeader, data: &[u8], source: Source) -> bool {


        if (h.ep & usb::DIR_REMOVE) == 0 {
            // Control requests go to endpoint 0

            if !self.check_control_req(h, data, source) {
                return false;
            }

        } else {

            // All other requests go to other endpoints


            // TODO: data requests might still go throuh this brach.
            // Check that this is not the case, or deal with it otherwise.

            if data.len() < usb::bbb::HEADER_SIZE {
                error!("[E009-BBB] Not enough payload for header");
                return false;
            }

            let header = parse_b_descriptor!(0, &data[..usb::bbb::HEADER_SIZE]);

            if header.signature == usb::bbb::CBW_SIGN {

                if source != Source::Blue {
                    error!("[E010-BBB] Device sending CBW packet");
                    return false;
                }

                if data.len() < usb::bbb::HEADER_SIZE + usb::bbb::CBW_SIZE {
                    error!("[E011-BBB] Not enough payload for CBW");
                    return false;
                }

                let cbw = parse_b_descriptor!(usb::bbb::CBW_SIGN, &data[usb::bbb::HEADER_SIZE..]);

                self.header = Some(header);
                self.cbw = Some(cbw);

            } else if header.signature == usb::bbb::CSW_SIGN {

                if source != Source::Red {
                    error!("[E012-BBB] Host sending CSW packet");
                    return false;
                }

                if self.header.is_none() || self.cbw.is_none() {
                    error!("[E013-BBB] Received a CSW without a CBW");
                    return false;
                }

                if self.header.unwrap().tag != header.tag {
                    error!("[E014-BBB] Received a CSW without a matching tag");
                    return false;
                }

                if data.len() < usb::bbb::HEADER_SIZE + usb::bbb::CSW_SIZE {
                    error!("[E015-BBB] Not enough payload fro CSW");
                    return false;
                }

                let csw = parse_b_descriptor!(usb::bbb::CSW_SIGN, &data[usb::bbb::HEADER_SIZE..]);


                match csw.status {

                    x if x == usb::bbb::STAT_OK || x == usb::bbb::STAT_FAIL => {
                        if csw.data_residue > self.cbw.unwrap().transfer_length {
                            error!("[E016-BBB] Data residue higher than transfer length");
                            return false;
                        }
                    }

                    usb::bbb::STAT_PHASE => {}

                    _ => {
                        error!("[E017-BBB] Invalid csw status 0x{:x}", csw.status);
                        return false;
                    }
                }
            } else {
                error!("[E018-BBB] Invalid signature 0x{:x}", header.signature);
                return false;
            }
        }

        true
    }
}
