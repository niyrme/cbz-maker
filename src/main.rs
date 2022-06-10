#![allow(non_snake_case)]

use std::{
	fs::{self, File},
	io::{Read, Write},
};

use walkdir::WalkDir;
use zip::ZipWriter;

const SRC_PATH: &str = "./cbzMaker/src";
const OUT_PATH: &str = "./cbzMaker/out";

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
					println!("[ERROR] {:?}", &e);
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

		let entryPathStr = entryPath.to_str().unwrap();
		let entryName = entryPathStr.get(SRC_PATH.len() + 1..).unwrap();

		println!("Creating: {}", entryName);

		for chap in iterPath(entryPathStr) {
			let chapPath = chap.path();
			if !chapPath.is_dir() {
				continue;
			}

			let chapPathStr = chapPath.to_str().unwrap();
			let chapName = chapPathStr.get(entryPathStr.len() + 1..).unwrap();

			println!("   Creating Chapter: {}", chapName);

			let outPath = format!("{}/{}", OUT_PATH, entryName);
			let outCBZ = format!("{}/{} {}.cbz", outPath, entryName, chapName);

			fs::create_dir_all(outPath)?;
			let f = File::create(outCBZ)?;

			let mut zip = ZipWriter::new(f);

			let pages: Vec<_> = iterPath(chapPathStr).collect();
			let pLen = pages.len();

			let mut i = 0;

			let mut buf = Vec::new();

			for page in pages.iter() {
				let pagePath = page.path();
				if !pagePath.is_file() {
					continue;
				}

				i += 1;

				let pageExt = pagePath.extension().unwrap().to_str().unwrap();

				File::open(pagePath)?.read_to_end(&mut buf)?;

				zip.start_file(
					format!("{:0width$}.{ext}", i, width = pLen, ext = pageExt),
					Default::default(),
				)?;
				zip.write_all(&*buf)?;

				buf.clear();
			}

			File::create(format!("{}/{}/.nomedia", OUT_PATH, entryName))?;

			zip.finish()?;
		}

		println!("");
	}

	Ok(())
}
