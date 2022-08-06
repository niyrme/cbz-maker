use serde::Serialize;

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
	pub fn new<T: ToString>(
		title: T,
		author: T,
		artist: T,
		description: T,
		genre: Vec<String>,
		status: T,
		statusValues: Vec<String>,
	) -> Self {
		Self {
			title: title.to_string(),
			author: author.to_string(),
			artist: artist.to_string(),
			description: description.to_string(),
			genre,
			status: status.to_string(),
			statusValues,
		}
	}

	pub fn barebone<T: ToString>(title: T) -> Self {
		Self {
			title: title.to_string(),
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
