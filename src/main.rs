#![allow(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
	fs::{self, File},
	io::{self, Read, Write},
	thread,
};

use anyhow::{Context, Result};
use cbzmaker::{errorln, Details, Err};
use eframe::NativeOptions;
use walkdir::{DirEntry, WalkDir};
use window::{isDir, ITEMS};
use zip::ZipWriter;

mod window;

const SRC_PATH: &str = "./cbzMaker/src";
const OUT_PATH: &str = "./cbzMaker/out";

fn makeCBZ(dirEntry: DirEntry) -> Result<()> {
	let iterPath = |path: &str, filter: fn(&DirEntry) -> bool| {
		WalkDir::new(path)
			.max_depth(1)
			.contents_first(false)
			.into_iter()
			.filter_map(|res| res.ok())
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

	let entryOutPath = format!("{}/{}", OUT_PATH, &entryName);
	fs::create_dir_all(&entryOutPath).with_context(|| format!("failed to create entry output path: {entryOutPath}"))?;

	let chapters: Vec<_> = iterPath(entryPathStr, isDir).collect();

	let total = chapters.len() as f32;

	unsafe {
		ITEMS.insert(entryName.to_string(), (0.0, total));
	}

	for (idx, chapter) in chapters.iter().enumerate() {
		let chapterPath = chapter.path();
		let chapterPathStr = chapterPath
			.to_str()
			.with_context(|| format!("failed to convert chapter path to string: {chapterPath:?}"))?;
		let chapterName = chapterPathStr
			.get(entryPathStr.len() + 1..)
			.with_context(|| "failed to get chapter name from path")?;

		let entryOutCBZ = format!("{}/{} - {}.cbz", &entryOutPath, &entryName, &chapterName);

		let zipF =
			File::create(&entryOutCBZ).with_context(|| format!("failed to create output CBZ file: {entryOutCBZ}"))?;
		let mut zipWriter = ZipWriter::new(zipF);

		let pages: Vec<_> = iterPath(chapterPathStr, |entry| entry.path().is_file()).collect();
		let pageCount = pages.len();

		let mut i = 0;

		let mut buf: Vec<u8> = Vec::new();

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

		unsafe {
			ITEMS.insert(entryName.to_string(), ((idx as f32) + 1.0, total));
		}
	}

	// create .nomedia
	if let Err(e) = File::create(format!("{entryOutPath}/.nomedia")) {
		eprintln!("failed to create file: {entryOutPath}/.nomedia: {e}");
	}

	if let Err(e) = fs::copy(
		format!("{entryPathStr}/ComicInfo.xml"),
		format!("{entryOutPath}/ComicInfo.xml"),
	) {
		eprintln!("Failed to copy ComicInfo.xml: {e}");

		if let Err(e) = File::create(format!("{entryOutPath}/.noxml")) {
			eprintln!("failed to create file: {entryOutPath}.noxml: {e}");
		}
	}

	if let Err(e) = fs::copy(
		format!("{entryPathStr}/details.json"),
		format!("{entryOutPath}/details.json"),
	) {
		if e.kind() == io::ErrorKind::NotFound {
			File::create(format!("{entryOutPath}/details.json"))?
				.write_all(serde_json::to_string_pretty(&Details::barebone(entryName))?.as_bytes())?;
			Ok(())
		} else {
			Err(e.into())
		}
	} else {
		Ok(())
	}
}

fn run() {
	if let Err(e) = fs::create_dir_all(SRC_PATH) {
		errorln!(&e);
		return;
	}

	let entries: Vec<_> = WalkDir::new(SRC_PATH)
		.max_depth(1)
		.contents_first(false)
		.into_iter()
		.filter_map(|res| res.ok())
		.skip(1)
		.filter(isDir)
		.collect();

	let mut handles = Vec::with_capacity(entries.len());

	for entry in entries {
		handles.push(thread::spawn(move || makeCBZ(entry)));
	}

	for handle in handles {
		if let Ok(res) = handle.join() {
			if let Err(e) = res {
				errorln!(e);
			}
		}
	}
}

fn main() {
	eframe::run_native(
		"CBZMaker",
		NativeOptions::default(),
		Box::new(|cctx| {
			let t = thread::spawn(move || run());

			Box::new(window::Window::new(cctx, t))
		}),
	);
}
