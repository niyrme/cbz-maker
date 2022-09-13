use serde::ser::{Serialize, SerializeStruct, Serializer};

#[derive(Debug)]
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

impl Serialize for Details {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut s = serializer.serialize_struct("Details", 7)?;
		s.serialize_field("title", &self.title)?;
		s.serialize_field("author", &self.author)?;
		s.serialize_field("artist", &self.artist)?;
		s.serialize_field("description", &self.description)?;
		s.serialize_field("genre", &self.genre)?;
		s.serialize_field("status", &self.status)?;
		s.serialize_field("_status values", &self.statusValues)?;
		s.end()
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
				String::from("4 = Publishing finished"),
				String::from("5 = Cancelled"),
				String::from("6 = On hiatus"),
			],
		}
	}
}
