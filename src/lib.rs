#![allow(non_snake_case)]

pub mod details;
pub use details::Details;

#[macro_export]
macro_rules! time {
	() => {
		chrono::Utc::now().format("%Y/%m/%d %H:%M:%S")
	};
}

#[macro_export]
macro_rules! log {
	($msg:expr) => {
		println!("[INFO] {} | {}", cbzmaker::time!(), $msg)
	};
	($entry:expr, $chapter:expr) => {
		println!("[INFO] {} | {} | Creating {}", cbzmaker::time!(), $entry, $chapter)
	};
}

#[macro_export]
macro_rules! error {
	($msg:expr) => {
		eprint!("[ERROR] {} | {}", cbzmaker::time!(), $msg)
	};
}

#[macro_export]
macro_rules! errorln {
	($msg:expr) => {
		cbzmaker::error!(format!("{}\n", $msg))
	};
}
