#![allow(non_snake_case)]

use std::{
	fs::{self, File},
	io::{ErrorKind, Read, Write},
};

use cbzmaker::{error, getOkOrErrorContinue, getSomeOrErrorContinue, info, Details};
use walkdir::{DirEntry, WalkDir};
use zip::ZipWriter;

const SRC_PATH: &str = "./cbzMaker/src";
const OUT_PATH: &str = "./cbzMaker/out";

pub fn isDir(entry: &DirEntry) -> bool { entry.path().is_dir() }

fn main() -> std::io::Result<()> {
	fs::create_dir_all(SRC_PATH)?;
	fs::create_dir_all(OUT_PATH)?;

	let iterPath = |path: &str, filter: fn(&DirEntry) -> bool| {
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
			.filter(filter)
	};

	for entry in iterPath(SRC_PATH, isDir) {
		let entryPath = entry.path();
		let entryPathStr = getSomeOrErrorContinue!(entryPath.to_str(), "failed to convert entry path to string");
		let entryName = getSomeOrErrorContinue!(entryPathStr.get(SRC_PATH.len() + 1..), "failed to get entry name from path");

		info!(format!("Creating: {}", entryName));

		let entryOutPath = format!("{}/{}", OUT_PATH, &entryName);
		fs::create_dir_all(&entryOutPath)?;

		'chapterLoop: for chapter in iterPath(entryPathStr, isDir) {
			let chapterPath = chapter.path();
			let chapterPathStr = getSomeOrErrorContinue!(chapterPath.to_str(), "failed to convert chapter path to string");
			let chapterName = getSomeOrErrorContinue!(
				chapterPathStr.get(entryPathStr.len() + 1..),
				"failed to get name from chapter"
			);

			info!(format!("   Creating Chapter: {}", chapterName));

			let entryOutCBZ = format!("{}/{} {}.cbz", &entryOutPath, &entryName, &chapterName);

			let cbzFile = getOkOrErrorContinue!(File::create(entryOutCBZ), "failed to create cbz file");

			let mut zipWriter = ZipWriter::new(cbzFile);

			let pages: Vec<_> = iterPath(chapterPathStr, |entry| entry.path().is_file()).collect();
			let pageCount = pages.len();

			let mut i = 0;

			let mut buf = Vec::new();

			for page in pages.iter() {
				let pagePath = page.path();

				i += 1;

				let pageExt = match pagePath.extension().and_then(|e| e.to_str()) {
					Some(v) => v,
					None => {
						error!("failed to get extension from file");
						continue 'chapterLoop;
					}
				};

				let mut pageFile = match File::open(pagePath) {
					Ok(f) => f,
					Err(e) => {
						error!(format!("failed to open image file: {}", e));
						continue 'chapterLoop;
					}
				};

				if let Err(e) = pageFile.read_to_end(&mut buf) {
					error!(format!("failed to read image file: {}", e));
					continue 'chapterLoop;
				}

				if buf.is_empty() {
					error!("something went wrong, buffer is empty");
					continue 'chapterLoop;
				}

				let pageName = format!("{:0width$}.{ext}", i, width = pageCount, ext = pageExt);
				if let Err(e) = zipWriter.start_file(pageName, Default::default()) {
					error!(format!("failed to start zip file: {}", e));
					continue 'chapterLoop;
				};

				if let Err(e) = zipWriter.write_all(&*buf) {
					error!(format!("failed to write to zip file: {}", e));
					continue 'chapterLoop;
				}

				buf.clear();
			}

			if let Err(e) = zipWriter.finish() {
				error!(format!("failed to finish archive: {}", e));
			}
		}

		// create .nomedia
		if let Err(e) = File::create(format!("{}/.nomedia", &entryOutPath)) {
			error!(format!("failed to create .nomedia file: {}", e));
		}

		match File::create(format!("{}/details.json", &entryOutPath)) {
			Ok(mut detailsF) => match File::open(format!("{}/details.json", &entryPathStr)) {
				Ok(mut f) => {
					// copy details.json
					let mut buf = Vec::new();
					if let Err(e) = f.read_to_end(&mut buf) {
						error!(format!("failed to read source details.json: {}", e))
					} else if let Err(e) = detailsF.write_all(&mut buf) {
						error!(format!("failed to write to destination details.json: {}", e))
					} else {
						info!("   Copied over details.json")
					}
				}
				Err(e) => {
					if e.kind().eq(&ErrorKind::NotFound) {
						// create details.json
						let details = Details::barebone(entryName.to_string());
						match detailsF.write_all(serde_json::to_string_pretty(&details)?.as_bytes()) {
							Ok(_) => info!("   Created barebone details.json"),
							Err(e) => error!(format!("failed to create details.json {}", e)),
						}
					} else {
						error!(format!("   Failed to read source details.json: {}", e))
					}
				}
			},
			Err(e) => error!(format!("failed to create/copy details.json file: {}", e)),
		};

		println!("");
	}

	info!("Done!");

	Ok(())
}
