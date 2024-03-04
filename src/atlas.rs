use crate::myimage::MyImage;
use std::fs;

pub struct Atlas {
	pub nb_sprites : usize,
	pub ids : Vec<usize>,
	pub sprites : Vec<(usize, MyImage)>, // (ID, Image)
}


impl Atlas {

	pub fn new() -> Atlas {
		Atlas {
			nb_sprites : 0,
			ids : Vec::new(),
			sprites : Vec::new(),
		}
	}

	pub fn add_sprite(&mut self, path : &str) {
		// find the id of the object by looking at the path format : "..../id.png"
		let subdirectories : Vec<&str> = path.split(".").peekable().peek().unwrap().split("\\").collect();
		// on prend le dernier élément de la liste
		let id = subdirectories.last().unwrap().parse::<usize>().unwrap();

		let img = MyImage::from_path(path);
		self.ids.push(id);
		self.sprites.push((id, img));

		self.nb_sprites += 1;
	}

	pub fn load(paths : &[&str]) -> Atlas {
		let mut atlas = Atlas::new();
		for path in paths {
			atlas.add_sprite(path);
		}
		atlas
	}

	pub fn load_trim(paths : &[&str]) -> Atlas {
		let mut atlas = Atlas::load(paths);
		for sprite in &mut atlas.sprites {
			let trimmed = sprite.1.trim();
			sprite.1 = trimmed;
		}
		atlas
	}

	pub fn load_directory(path : &str) -> Atlas {
		let mut atlas = Atlas::new();
		let paths = fs::read_dir(path).unwrap();
		for path in paths {
			let path = path.unwrap().path().display().to_string();
			atlas.add_sprite(&path);
		}
		atlas
	}

	pub fn load_directory_trim(path : &str) -> Atlas {
		let mut atlas = Atlas::load_directory(path);
		for sprite in &mut atlas.sprites {
			let trimmed = sprite.1.trim();
			sprite.1 = trimmed;
		}
		atlas
	}

	pub fn get_sprite(&self, id : usize) -> &MyImage {
		for sprite in &self.sprites {
			if sprite.0 == id {
				return &sprite.1;
			}
		}
		panic!("{}", format!("Sprite with id {} not found", id));
	}
	
}