use std::{collections::BTreeMap, thread::JoinHandle};

use egui::ProgressBar;
use walkdir::DirEntry;

pub(crate) fn isDir(entry: &DirEntry) -> bool { entry.path().is_dir() }

pub(crate) static mut ITEMS: BTreeMap<String, (f32, f32)> = BTreeMap::new();

#[derive(Debug)]
pub(crate) struct Window {
	t: JoinHandle<()>,
}

impl Window {
	pub(crate) fn new(cctx: &eframe::CreationContext<'_>, t: JoinHandle<()>) -> Self {
		cctx.egui_ctx.set_visuals(egui::Visuals::dark());

		Self { t }
	}
}

impl eframe::App for Window {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ctx, |ui| unsafe {
			for (name, (current, total)) in ITEMS.iter() {
				ui.add(ProgressBar::new(current / total).show_percentage().text(name));
			}

			self.t.is_finished().then(|| ui.heading("DONE!"));
		});

		ctx.request_repaint();
	}
}
