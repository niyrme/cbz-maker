#![allow(non_snake_case)]

use serde::Serialize;

#[macro_export]
macro_rules! info {
	($msg:expr) => {
		println!("[INFO] {}", $msg)
	};
}

#[macro_export]
macro_rules! error {
	($msg:expr) => {
		eprintln!("[ERROR] {}", $msg)
	};
}

#[macro_export]
macro_rules! getSomeOrErrorContinue {
	($res:expr, $msg:expr) => {
		if let Some(v) = $res {
			v
		} else {
			error!($msg);
			continue;
		}
	};
}

#[macro_export]
macro_rules! getOkOrErrorContinue {
	($res:expr, $errMsg:expr) => {
		match $res {
			Ok(v) => v,
			Err(e) => {
				error!(format!("{}: {}", $errMsg, e));
				continue;
			}
		}
	};
}

#[derive(Debug, Serialize)]
pub struct Details {
	title:        String,
	author:       String,
	artist:       String,
	description:  String,
	genre:        Vec<String>,
	status:       String,
	statusValues: Vec<String>,
}

impl Details {
	pub fn new(
		title: String,
		author: String,
		artist: String,
		description: String,
		genre: Vec<String>,
		status: String,
		statusValues: Vec<String>,
	) -> Self {
		Self {
			title,
			author,
			artist,
			description,
			genre,
			status,
			statusValues,
		}
	}

	pub fn barebone(title: String) -> Self {
		Self {
			title,
			..Default::default()
		}
	}
}

impl Default for Details {
	fn default() -> Self {
		Self {
			title:        String::new(),
			author:       String::new(),
			artist:       String::new(),
			description:  String::new(),
			genre:        Vec::new(),
			status:       String::from("0"),
			statusValues: vec![
				String::from("0 = Unknown"),
				String::from("1 = Ongoing"),
				String::from("2 = Completed"),
				String::from("3 = Licensed"),
			],
		}
	}
}
