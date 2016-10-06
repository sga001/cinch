#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct CinchConfig {
    pub red_addr: String, // ip:port
    pub cinch_addr: String, // ip:port
    pub log: bool,
    pub log_prefix: String, // (e.g., logs/trace results in logs/trace-ts1.log, logs/trace-ts2.log)
    pub checks_active: bool,
    pub patch_active: bool,
    pub patches: String, // Folder containing patches
    pub third_party_folder: String, // Folder containing third-party checks
}
