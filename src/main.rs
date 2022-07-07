#![allow(non_snake_case)]

use std::{
	fs::{self, File},
	io::{self, Error, ErrorKind, Read, Result, Write},
};

use cbzmaker::{errorln, info, infoln, Details};
use walkdir::{DirEntry, WalkDir};
use zip::ZipWriter;

const SRC_PATH: &str = "./cbzMaker/src";
const OUT_PATH: &str = "./cbzMaker/out";

pub fn isDir(entry: &DirEntry) -> bool { entry.path().is_dir() }

macro_rules! err {
	($msg:expr) => {
		Error::new(ErrorKind::Other, $msg)
	};
}

macro_rules! Err {
	($msg:expr) => {
		Err(err!($msg))
	};
}

fn makeCBZ(dirEntry: DirEntry) -> Result<()> {
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
	let entryPathStr = entryPath.to_str().ok_or(err!("failed to convert entry path to string"))?;
	let entryName = entryPathStr
		.get(SRC_PATH.len() + 1..)
		.ok_or(err!("failed to get entry name from path"))?;

	infoln!(format!("Creating: {}", entryName));

	let entryOutPath = format!("{}/{}", OUT_PATH, &entryName);
	fs::create_dir_all(&entryOutPath)?;

	for chapter in iterPath(entryPathStr, isDir) {
		let chapterPath = chapter.path();
		let chapterPathStr = chapterPath.to_str().ok_or(err!("failed to convert chapter path to string"))?;
		let chapterName = chapterPathStr
			.get(entryPathStr.len() + 1..)
			.ok_or(err!("failed to get name from chapter"))?;

		infoln!(format!("   Creating Chapter: {}", chapterName));

		let entryOutCBZ = format!("{}/{} {}.cbz", &entryOutPath, &entryName, &chapterName);

		let mut zipWriter = ZipWriter::new(File::create(entryOutCBZ)?);

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
				.ok_or(err!("failed to get extension from file"))?;

			File::open(pagePath)?.read_to_end(&mut buf)?;

			if buf.is_empty() {
				return Err!("something went wrong, buffer is empty");
			}

			let pageName = format!("{:0width$}.{ext}", i, width = pageCount, ext = pageExt);
			zipWriter.start_file(pageName, Default::default())?;
			zipWriter.write_all(&*buf)?;

			buf.clear();
		}

		zipWriter.finish()?;
	}

	// create .nomedia
	File::create(format!("{}/.nomedia", &entryOutPath))?;

	match (
		File::create(format!("{}/details.json", &entryOutPath)),
		File::open(format!("{}/details.json", &entryPathStr)),
	) {
		(Ok(mut detailsF), Ok(mut f)) => {
			let mut buf = Vec::new();
			f.read_to_end(&mut buf)?;
			detailsF.write_all(&mut buf)?;
			infoln!("   Copied over details.json");
			Ok(())
		}
		(Ok(mut detailsF), Err(e)) if e.kind().eq(&ErrorKind::NotFound) => {
			let details = Details::barebone(entryName);
			detailsF.write_all(serde_json::to_string_pretty(&details)?.as_bytes())?;
			infoln!("   Creted new details.json");
			Ok(())
		}
		(Ok(_), Err(e)) => Err(e),
		(Err(e), _) => Err(e),
	}
}

fn main() -> Result<()> {
	fs::create_dir_all(SRC_PATH)?;
	fs::create_dir_all(OUT_PATH)?;

	for entry in WalkDir::new(SRC_PATH)
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
	{
		if let Err(e) = makeCBZ(entry) {
			errorln!(format!("{:?}", e));
		}
		println!("");
	}

	info!("Done!");

	io::stdout().flush()?;
	io::stdin().read_line(&mut String::new())?;

	Ok(())
}
