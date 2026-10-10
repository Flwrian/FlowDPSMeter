//! Only the local Windows APIs used by this application.
#[path = "win32/admin.rs"] pub mod admin;
#[path = "win32/pcap.rs"] pub mod pcap;
#[path = "win32/atomic_file.rs"] pub mod atomic_file;
#[path = "win32/window_detector.rs"] pub mod window_detector;
