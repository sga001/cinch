use std::collections::HashMap;
use std::io::prelude::*;
use std::fs;
use std::rc::Rc;
use rustc_serialize::json;

use usb;


#[derive(RustcDecodable)]
struct FieldCheck {
    field: String,
    operation: String,
    value: u16,
}

#[derive(RustcDecodable)]
struct Constraint {
    id: u16,
    desc_type: String,
    field_checks: Vec<FieldCheck>,
    count: Option<u16>,
}

#[derive(RustcDecodable)]
struct PatchId {
    vendor_id: u16,
    product_id: u16,
}

#[derive(RustcDecodable)]
struct CompliancePatch {
    ids: Vec<PatchId>,
    constraints: Vec<Constraint>,
}

pub struct Patcher {
    patches: HashMap<(u16, u16), Rc<CompliancePatch>>,
    satisfied: HashMap<(u16, u16, u16), u16>, // constraint -> satisfied?

    num_ifs: u8,
    num_eps: u8,
}

// TODO: Change from string matches to enum matches... Problem is json doesn't support enum :(
impl Patcher {
    pub fn new(dir_path: &str) -> Patcher {

        let mut patches = HashMap::new();

        if let Ok(dir) = fs::read_dir(dir_path) {
            for entry in dir {
                let mut file = match fs::File::open(&entry.unwrap().path()) {
                    Ok(file) => file,
                    Err(e) => panic!("[E000-TP] Could not open file {}", e),
                };

                // read file
                let mut json_line = String::new();
                file.read_to_string(&mut json_line).unwrap();

                // decode file and add
                let patch: Rc<CompliancePatch> = Rc::new(json::decode(&json_line).unwrap());

                for id in &patch.ids {
                    patches.insert((id.vendor_id, id.product_id), patch.clone());
                }
            }
        }

        Patcher { patches: patches, satisfied: HashMap::new(), num_ifs: 0, num_eps: 0 }
    }


    pub fn check_config_fields(&mut self, config: &usb::ConfigDescriptor, dev: &usb::DeviceDescriptor) -> bool {

        if let Some(ref patch) = self.patches.get(&(dev.id_vendor, dev.id_product)) {
            for constraint in &patch.constraints {
                if constraint.desc_type == "configuration" {

                    for fc in &constraint.field_checks {

                        let value: u16 = match fc.field.as_ref() {
                            "total_length" => config.total_length,
                            "num_interfaces" => config.num_interfaces as u16,
                            "configuration_value" => config.configuration_value as u16,
                            "configuration" => config.configuration as u16,
                            "attributes" => config.attributes as u16,
                            "max_power" => config.max_power as u16,
                            _ => panic!("[E001-TP] Invalid field index for config descriptor"),
                        };

                        let res: bool = match fc.operation.as_ref() {
                            "leq" => value <= fc.value,
                            "eq" => value == fc.value,
                            "geq" => value >= fc.value,
                            "and" => value & fc.value == fc.value,
                            "or" => value | fc.value == fc.value,
                            "bit_is_set" => value & (1 << fc.value) != 0,
                            "bit_not_set" => value & (1 << fc.value) == 0,
                            _ => panic!("[E002-TP] Invalid operation type {}", fc.operation),
                        };

                        if !res {
                            error!("[E003-TP] Constraint config check failed ({}): value {} {} value {}",
                                   fc.field,
                                   value,
                                   fc.operation,
                                   fc.value);
                            return res;
                        }
                    }
                }
            }

            self.num_ifs = config.num_interfaces;
        }

        true
    }

    pub fn check_iface_fields(&mut self, iface: &usb::InterfaceDescriptor, dev: &usb::DeviceDescriptor) -> bool {

        if let Some(ref patch) = self.patches.get(&(dev.id_vendor, dev.id_product)) {
            for constraint in &patch.constraints {
                if constraint.desc_type == "interface" {

                    let mut res = true;

                    for fc in &constraint.field_checks {
                        let value: u16 = match fc.field.as_ref() {
                            "interface_number" => iface.interface_number as u16,
                            "alternate_setting" => iface.alternate_setting as u16,
                            "num_endpoints" => iface.num_endpoints as u16,
                            "interface_class" => iface.interface_class as u16,
                            "interface_subclass" => iface.interface_subclass as u16,
                            "interface_protocol" => iface.interface_protocol as u16,
                            "interface" => iface.interface as u16,
                            _ => panic!("[E004-TP] Invalid field index for interface descriptor"),
                        };

                        res = res &&
                              match fc.operation.as_ref() {
                            "leq" => value <= fc.value,
                            "eq" => value == fc.value,
                            "geq" => value >= fc.value,
                            "and" => value & fc.value == fc.value,
                            "or" => value | fc.value == fc.value,
                            "bit_is_set" => value & (1 << fc.value) != 0,
                            "bit_not_set" => value & (1 << fc.value) == 0,
                            _ => panic!("[E005-TP] Invalid operation type {}", fc.operation),
                        };

                        if !res {

                            if constraint.count.is_some() {
                                break;
                            }

                            error!("[E006-TP] Constraint interface check failed ({}): value {} {} value {}",
                                   fc.field,
                                   value,
                                   fc.operation,
                                   fc.value);
                            return res;
                        }
                    }

                    if let Some(count) = constraint.count {
                        let val = self.satisfied
                            .entry((dev.id_vendor, dev.id_product, constraint.id))
                            .or_insert(count);

                        if res && *val > 0 {
                            *val -= 1;
                        }
                    }
                }
            }

            if self.num_ifs > 0 {
                self.num_ifs -= 1;
                self.num_eps = iface.num_endpoints;
            }

            if self.num_ifs == 0 && self.num_eps == 0 {

                // We are done with all. Ensure all constraints were satisfied.
                for (key, count) in &self.satisfied {
                    if *count != 0 {
                        error!("[E011-TP] Not all constraints were satisfied id: {:?}, count: {}",
                               key,
                               count);
                        return false;
                    }
                }

            }
        }

        true
    }

    pub fn check_endpoint_fields(&mut self, ep: &usb::EndpointDescriptor, dev: &usb::DeviceDescriptor) -> bool {

        if let Some(ref patch) = self.patches.get(&(dev.id_vendor, dev.id_product)) {
            for constraint in &patch.constraints {
                if constraint.desc_type == "endpoint" {

                    let mut res = true;

                    for fc in &constraint.field_checks {

                        let value: u16 = match fc.field.as_ref() {
                            "endpoint_address" => ep.endpoint_address as u16,
                            "attributes" => ep.attributes as u16,
                            "max_packet_size" => ep.max_packet_size as u16,
                            "interval" => ep.interval as u16,
                            _ => panic!("[E007-TP] Invalid field index for endpoint descriptor"),
                        };

                        res = res &&
                              match fc.operation.as_ref() {
                            "leq" => value <= fc.value,
                            "eq" => value == fc.value,
                            "geq" => value >= fc.value,
                            "and" => value & fc.value == fc.value,
                            "or" => value | fc.value == fc.value,
                            "bit_is_set" => value & (1 << fc.value) != 0,
                            "bit_not_set" => value & (1 << fc.value) == 0,
                            _ => panic!("[E008-TP] Invalid operation type {}", fc.operation),
                        };

                        if !res {

                            if constraint.count.is_some() {
                                break;
                            }

                            error!("[E009-TP] Constraint endpoint check failed ({}): value {} {} value {}",
                                   fc.field,
                                   value,
                                   fc.operation,
                                   fc.value);
                            return res;
                        }
                    }

                    if let Some(count) = constraint.count {
                        let val = self.satisfied
                            .entry((dev.id_vendor, dev.id_product, constraint.id))
                            .or_insert(count);

                        if res && *val > 0 {
                            *val -= 1;
                        }
                    }
                }
            }

            if self.num_eps > 0 {
                self.num_eps -= 1;
            }

            if self.num_ifs == 0 && self.num_eps == 0 {

                // We are done with all. Ensure all constraints were satisfied.
                for (key, count) in &self.satisfied {
                    if *count != 0 {
                        error!("[E010-TP] Not all constraints were satisfied id: {:?}, count: {}",
                               key,
                               count);
                        return false;
                    }
                }
            }
        }

        true
    }
}
