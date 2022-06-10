#![allow(non_snake_case)]

use std::{
	fs::{self, File},
	io::{Read, Write},
};

use walkdir::WalkDir;
use zip::ZipWriter;

const SRC_PATH: &str = "./cbzMaker/src";
const OUT_PATH: &str = "./cbzMaker/out";

macro_rules! log {
	($level:expr, $msg:expr) => {
		println!("[{}] {}", $level, $msg)
	};
}
macro_rules! info {
	($msg:expr) => {
		log!("INFO", $msg);
	};
}
macro_rules! error {
	($msg:expr) => {
		log!("ERROR", $msg);
	};
}

macro_rules! getOrErrorContinue {
	($res:expr, $msg:expr) => {
		if let Some(v) = $res {
			v
		} else {
			error!($msg);
			continue;
		}
	};
}

fn main() -> std::io::Result<()> {
	fs::create_dir_all(SRC_PATH)?;
	fs::create_dir_all(OUT_PATH)?;

	let iterPath = |path: &str| {
		WalkDir::new(path)
			.max_depth(1)
			.contents_first(false)
			.into_iter()
			.filter_map(|e| {
				if e.is_err() {
					error!(format!("{:?}", &e));
				}
				e.ok()
			})
			.skip(1)
	};

	for entry in iterPath(SRC_PATH) {
		let entryPath = entry.path();
		if !entryPath.is_dir() {
			continue;
		}

		let entryPathStr = getOrErrorContinue!(entryPath.to_str(), "failed to convert entry path to string");
		let entryName = getOrErrorContinue!(entryPathStr.get(SRC_PATH.len() + 1..), "failed to get entry name from path");

		info!(format!("Creating: {}", entryName));

		'chapterLoop: for chap in iterPath(entryPathStr) {
			let chapPath = chap.path();
			if !chapPath.is_dir() {
				continue;
			}

			let chapPathStr = getOrErrorContinue!(chapPath.to_str(), "failed to convert chapter path to string");

			// let chapPathStr = chapPath.to_str().unwrap();
			let chapName = getOrErrorContinue!(chapPathStr.get(entryPathStr.len() + 1..), "failed to get name from chapter");

			info!(format!("   Creating Chapter: {}", chapName));

			let outPath = format!("{}/{}", OUT_PATH, entryName);
			let outCBZ = format!("{}/{} {}.cbz", outPath, entryName, chapName);

			fs::create_dir_all(outPath)?;
			let cbzF = match File::create(outCBZ) {
				Ok(v) => v,
				Err(e) => {
					error!(format!("failed to create cbz file: {}", e));
					continue;
				}
			};

			let mut zip = ZipWriter::new(cbzF);

			let pages: Vec<_> = iterPath(chapPathStr).collect();
			let pLen = pages.len();

			let mut i = 0;

			let mut buf = Vec::new();

			for page in pages.iter() {
				let pagePath = page.path();
				if !pagePath.is_file() {
					continue 'chapterLoop;
				}

				i += 1;

				let pageExt = getOrErrorContinue!(
					pagePath.extension().and_then(|e| e.to_str()),
					"failed to get extension from file"
				);

				let mut pageF = match File::open(pagePath) {
					Ok(f) => f,
					Err(e) => {
						error!(format!("failed to open image file: {}", e));
						continue 'chapterLoop;
					}
				};

				match pageF.read_to_end(&mut buf) {
					Ok(_) => {}
					Err(e) => {
						error!(format!("failed to read image file: {}", e));
						continue 'chapterLoop;
					}
				};

				if buf.is_empty() {
					error!("something went wrong, buffer is empty");
					continue 'chapterLoop;
				}

				match zip.start_file(
					format!("{:0width$}.{ext}", i, width = pLen, ext = pageExt),
					Default::default(),
				) {
					Ok(_) => {}
					Err(e) => {
						error!(format!("failed to start zip file: {}", e));
						continue 'chapterLoop;
					}
				};

				match zip.write_all(&*buf) {
					Ok(_) => {}
					Err(e) => {
						error!(format!("failed to write to zip file: {}", e));
						continue 'chapterLoop;
					}
				}

				buf.clear();
			}

			match File::create(format!("{}/{}/.nomedia", OUT_PATH, entryName)) {
				Ok(_) => {}
				Err(e) => {
					error!(format!("failed to create .nomedia file: {}", e));
				}
			};

			match zip.finish() {
				Ok(_) => {}
				Err(e) => {
					error!(format!("failed to finish archive: {}", e));
				}
			}
		}

		println!("");
	}

	info!("done!");

	Ok(())
}
