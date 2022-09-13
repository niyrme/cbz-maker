#![allow(non_snake_case)]

use std::{
	fs::{self, File},
	io::{self, Read, Write},
	thread,
};

use anyhow::{Context, Result};
use cbzmaker::{errorln, log, Details};
use futures::executor::block_on;
use walkdir::{DirEntry, WalkDir};
use zip::ZipWriter;

macro_rules! Err {
	($msg:expr) => {
		Err(anyhow::Error::new(std::io::Error::new(std::io::ErrorKind::Other, $msg)))
	};
	($msg:expr, $kind:expr) => {
		Err(anyhow::Error::new(std::io::Error::new($kind, $msg)))
	};
}

const SRC_PATH: &str = "./cbzMaker/src";
const OUT_PATH: &str = "./cbzMaker/out";

pub fn isDir(entry: &DirEntry) -> bool { entry.path().is_dir() }

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
	let entryPathStr = entryPath
		.to_str()
		.with_context(|| format!("failed to convert entry path to string: {entryPath:?}"))?;
	let entryName = entryPathStr
		.get(SRC_PATH.len() + 1..)
		.with_context(|| "failed to get entry name from path")?;

	log!(format!("Creating: {}", entryName));

	let entryOutPath = format!("{}/{}", OUT_PATH, &entryName);
	fs::create_dir_all(&entryOutPath).with_context(|| format!("failed to create entry output path: {entryOutPath}"))?;

	for chapter in iterPath(entryPathStr, isDir) {
		let chapterPath = chapter.path();
		let chapterPathStr = chapterPath
			.to_str()
			.with_context(|| format!("failed to convert chapter path to string: {chapterPath:?}"))?;
		let chapterName = chapterPathStr
			.get(entryPathStr.len() + 1..)
			.with_context(|| "failed to get chapter name from path")?;

		log!(&entryName, &chapterName);

		let entryOutCBZ = format!("{}/{} - {}.cbz", &entryOutPath, &entryName, &chapterName);

		let zipF =
			File::create(&entryOutCBZ).with_context(|| format!("failed to create output CBZ file: {entryOutCBZ}"))?;
		let mut zipWriter = ZipWriter::new(zipF);

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
				.with_context(|| format!("failed to get extension from file: {pagePath:?}"))?;

			File::open(pagePath)
				.with_context(|| format!("failed to open chapter page: {pagePath:?}"))?
				.read_to_end(&mut buf)
				.with_context(|| format!("failed to read chapter page: {pagePath:?}"))?;

			if buf.is_empty() {
				return Err!("something went wrong, buffer is empty");
			}

			let pageName = format!("{:0width$}.{ext}", i, width = pageCount, ext = pageExt);
			zipWriter
				.start_file(&pageName, Default::default())
				.with_context(|| format!("failed to start new zip file: {pageName}"))?;
			zipWriter
				.write_all(&buf)
				.with_context(|| "failed to write to zip file")?;

			buf.clear();
		}

		zipWriter.finish().with_context(|| "failed to finish zip file")?;
	}

	// create .nomedia
	// do not exit if this fails
	if let Err(e) = File::create(format!("{}/.nomedia", &entryOutPath))
		.with_context(|| format!("failed to create file: {entryOutPath}/.nomedia"))
	{
		eprintln!("{e}");
	}

	let res = match (
		File::create(format!("{}/details.json", &entryOutPath)),
		File::open(format!("{}/details.json", &entryPathStr)),
	) {
		(Ok(mut detailsF), Ok(mut f)) => {
			let mut buf = Vec::new();
			f.read_to_end(&mut buf)
				.with_context(|| format!("failed to read file: {entryOutPath}/details.json"))?;
			detailsF
				.write_all(&buf)
				.with_context(|| format!("failed to write file: {entryPathStr}/details.json"))?;
			log!(format!("{} | Copied over details.json", &entryName));
			Ok(())
		}
		(Ok(mut detailsF), Err(e)) if e.kind().eq(&io::ErrorKind::NotFound) => {
			let details = Details::barebone(entryName);
			detailsF
				.write_all(
					serde_json::to_string_pretty(&details)
						.with_context(|| "failed to convert details to json")?
						.as_bytes(),
				)
				.with_context(|| format!("failed to write file: {entryPathStr}/details.json"))?;
			log!(format!("{} | Created new details.json", &entryName));
			Ok(())
		}
		(Ok(_), Err(e)) => Err!(format!("{e}")),
		(Err(e), _) => Err!(format!("{e}")),
	};

	log!(format!("{} | Done!", &entryName));

	res
}

async fn amain() {
	if let Err(e) =
		fs::create_dir_all(SRC_PATH).with_context(|| format!("failed to create input directory: {}", SRC_PATH))
	{
		errorln!(&e);
		std::process::exit(1);
	}

	let entries: Vec<_> = WalkDir::new(SRC_PATH)
		.max_depth(1)
		.contents_first(false)
		.into_iter()
		.filter_map(|res| {
			if let Err(e) = &res {
				errorln!(format!("{e:?}"));
			}
			res.ok()
		})
		.skip(1)
		.filter(isDir)
		.collect();

	let mut handles = Vec::with_capacity(entries.len());

	for entry in entries {
		handles.push(thread::spawn(move || makeCBZ(entry)));
	}

	for handle in handles {
		match handle.join() {
			Ok(res) => drop(res.map_err(|e| errorln!(&e))),
			Err(e) => todo!("{e:?}"),
		};
	}

	log!("Done!");

	io::stdout().flush().unwrap();
	io::stdin().read_line(&mut String::new()).unwrap();
}

fn main() { block_on(amain()) }
