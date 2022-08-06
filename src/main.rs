#![allow(non_snake_case)]

use std::{
	fs::{self, File},
	io::{self, ErrorKind, Read, Write},
	thread,
};

use cbzmaker::{errorln, log, Details};
use futures::executor::block_on;
use walkdir::{DirEntry, WalkDir};
use zip::ZipWriter;

const SRC_PATH: &str = "./cbzMaker/src";
const OUT_PATH: &str = "./cbzMaker/out";

pub fn isDir(entry: &DirEntry) -> bool { entry.path().is_dir() }

fn makeCBZ(dirEntry: DirEntry) {
	let iterPath = |path: &str, filter: fn(&DirEntry) -> bool| {
		WalkDir::new(path)
			.max_depth(1)
			.contents_first(false)
			.into_iter()
			.filter_map(|res| {
				if res.is_err() {
					errorln!(format!("{:?}", &res));
				}
				res.ok()
			})
			.skip(1)
			.filter(filter)
	};

	let entryPath = dirEntry.path();
	let entryPathStr = entryPath.to_str().expect("failed to convert entry path to string");
	let entryName = entryPathStr
		.get(SRC_PATH.len() + 1..)
		.expect("failed to get entry name from path");

	log!(format!("Creating: {}", entryName));

	let entryOutPath = format!("{}/{}", OUT_PATH, &entryName);
	fs::create_dir_all(&entryOutPath).expect("failed to create entry output path");

	for chapter in iterPath(entryPathStr, isDir) {
		let chapterPath = chapter.path();
		let chapterPathStr = chapterPath.to_str().expect("failed to convert chapter path to string");
		let chapterName = chapterPathStr
			.get(entryPathStr.len() + 1..)
			.expect("failed to get name from chapter");

		log!(&entryName, &chapterName);

		let entryOutCBZ = format!("{}/{} - {}.cbz", &entryOutPath, &entryName, &chapterName);

		let mut zipWriter = ZipWriter::new(File::create(entryOutCBZ).expect("failed to create output ZBZ file"));

		let pages: Vec<_> = iterPath(chapterPathStr, |entry| entry.path().is_file()).collect();
		let pageCount = pages.len();

		let mut i = 0;

		let mut buf = Vec::new();

		for page in pages.iter() {
			let pagePath = page.path();

			i += 1;

			let pageExt = pagePath
				.extension()
				.and_then(|e| e.to_str())
				.expect("failed to get extension from file");

			File::open(pagePath)
				.expect("failed to open chapter page")
				.read_to_end(&mut buf)
				.expect("failed to read chapter page");

			if buf.is_empty() {
				errorln!("something went wrong, buffer is empty");
				return;
			}

			let pageName = format!("{:0width$}.{ext}", i, width = pageCount, ext = pageExt);
			zipWriter
				.start_file(pageName, Default::default())
				.expect("failed to start new zip file");
			zipWriter.write_all(&*buf).expect("failed to write to zip file");

			buf.clear();
		}

		zipWriter.finish().expect("failed to finish zip file");
	}

	// create .nomedia
	File::create(format!("{}/.nomedia", &entryOutPath)).expect("failed to create .nomedia");

	match (
		File::create(format!("{}/details.json", &entryOutPath)),
		File::open(format!("{}/details.json", &entryPathStr)),
	) {
		(Ok(mut detailsF), Ok(mut f)) => {
			let mut buf = Vec::new();
			f.read_to_end(&mut buf).expect("failed to read input details.json");
			detailsF.write_all(&mut buf).expect("failed to write to ouput details.json");
			log!(format!("{} | Copied over details.json", &entryName));
		}
		(Ok(mut detailsF), Err(e)) if e.kind().eq(&ErrorKind::NotFound) => {
			let details = Details::barebone(entryName);
			let jsn = serde_json::to_string_pretty(&details).expect("failed to convert details to json");
			detailsF.write_all(jsn.as_bytes()).expect("failed tow rite to output.json");
			log!(format!("{} | Created new details.json", &entryName));
		}
		(Ok(_), Err(e)) => {
			errorln!(format!("{e}"));
			return;
		}
		(Err(e1), Err(e2)) => {
			errorln!(format!("{e1}"));
			errorln!(format!("{e2}"));
			return;
		}
		(Err(e), _) => {
			errorln!(format!("{e}"));
			return;
		}
	};

	log!(format!("{} | Done!", &entryName));
}

async fn amain() {
	fs::create_dir_all(SRC_PATH).expect("failed to create input directory");
	fs::create_dir_all(OUT_PATH).expect("failed to create output directory");

	let entries: Vec<_> = WalkDir::new(SRC_PATH)
		.max_depth(1)
		.contents_first(false)
		.into_iter()
		.filter_map(|res| {
			if res.is_err() {
				errorln!(format!("{:?}", &res));
			}
			res.ok()
		})
		.skip(1)
		.filter(isDir)
		.collect();

	let mut threads = Vec::with_capacity(entries.len());

	for entry in entries {
		threads.push(thread::spawn(move || makeCBZ(entry)));
	}

	for t in threads {
		drop(t.join());
	}

	log!("Done!");

	io::stdout().flush().unwrap();
	io::stdin().read_line(&mut String::new()).unwrap();
}

fn main() { block_on(amain()) }
