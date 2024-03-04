use crate::myimage::{MyImage, uhuit_to_fsoixquatr, fsoixquatr_to_uhuit};
use crate::atlas::Atlas;

use rand::Rng;

#[derive(Clone, Copy)]
pub struct Object {
	pub id : usize,
	pub rotation : usize,
      pub coors : (isize, isize),
      pub size : f32,
      pub color : (u8,u8,u8)
}

impl Object {

	pub fn generate_randoms(number : usize, atlas : &Atlas, canvas : &MyImage) -> Vec<Object> {
		let mut rng = rand::thread_rng();
		let mut objects = Vec::new();
		for _ in 0..number {
			let num = rng.gen_range(0..atlas.nb_sprites);
			let id = atlas.ids[num];
			let rotation = rng.gen_range(0..360);
			let coors = (rng.gen_range(0..canvas.width as isize), rng.gen_range(0..canvas.height as isize));
			let size = rng.gen_range(0.1..10.0);
			let color = (rng.gen_range(0..255), rng.gen_range(0..255), rng.gen_range(0..255));
			objects.push(Object {id, rotation, coors, size, color});
		}
		return objects;
	}

	pub fn generate_randoms_stepped(number : usize, atlas : &Atlas, step_nr : usize, canvas : &MyImage) -> Vec<Object> {
		let mut rng = rand::thread_rng();
		let mut objects = Vec::new();
		
		let min_scale = match step_nr {
			x if x <= 20 => 1.0,
			x if x <= 1000 => 0.1,
			_ => 0.05 
		};
		let max_scale = match step_nr {
			x if x <= 20 => 5.0,
			x if x <= 200 => 2.5,
			x if x <= 1000 => 1.0,
			_ => 0.5,
		};

		for _ in 0..number {
			let num = rng.gen_range(0..atlas.nb_sprites);
			let id = atlas.ids[num];
			let rotation = rng.gen_range(0..360);
			let coors = (rng.gen_range(0..canvas.width as isize), rng.gen_range(0..canvas.height as isize));
			let size = rng.gen_range(min_scale..max_scale);
			let color = (rng.gen_range(0..255), rng.gen_range(0..255), rng.gen_range(0..255));
			objects.push(Object {id, rotation, coors, size, color});
		}

		return objects;
	}

	pub fn refill_randoms_stepped(vec_of_objects : &mut Vec<Object>, atlas : &Atlas, step_nr : usize, canvas : &MyImage) {
		//does the same as generate_randoms_stepped but it doesn't generate a new vector

		let mut rng = rand::thread_rng();

		let min_scale = match step_nr {
			x if x <= 20 => 1.0,
			x if x <= 1000 => 0.1,
			_ => 0.05 
		};

		let max_scale = match step_nr {
			x if x <= 20 => 5.0,
			x if x <= 100 => 2.5,
			x if x <= 1000 => 1.0,
			_ => 0.5,
		};

		for object in vec_of_objects.iter_mut() {
			let num = rng.gen_range(0..atlas.nb_sprites);
			object.id = atlas.ids[num];
			object.rotation = rng.gen_range(0..360);
			object.coors = (rng.gen_range(0..canvas.width as isize), rng.gen_range(0..canvas.height as isize));
			object.size = rng.gen_range(min_scale..max_scale);
			object.color = (rng.gen_range(0..255), rng.gen_range(0..255), rng.gen_range(0..255));
		}
	}
	

	pub fn sample(&self, sprite : &MyImage, x_img : usize, y_img : usize) -> (u8, u8, u8, u8) {

		//let rotation = self.rotation as f32;
		let rotation = (self.rotation as f32).to_radians();
		let rotation_sin = rotation.sin();
		let rotation_cos = rotation.cos();

		let x_new = (x_img as f32 - self.coors.0 as f32);
		let y_new = (y_img as f32 - self.coors.1 as f32);

		let x_sample = x_new * rotation_cos - y_new * rotation_sin;
		let y_sample = x_new * rotation_sin + y_new * rotation_cos;

		let x_sample = x_sample / self.size;
		let y_sample = y_sample / self.size;

		if x_sample < 0.0 || x_sample >= sprite.width as f32 || y_sample < 0.0 || y_sample >= sprite.height as f32 {
			return (0,0,0,0);
		}

		let x_sample = x_sample as usize;
		let y_sample = y_sample as usize;

		return sprite.get_pixel(x_sample, y_sample);
		
	}
		

	pub fn get_bound_box(&self, image : &MyImage, sprite : &MyImage) -> (usize, usize, usize, usize) {
		let sprite_length = ((sprite.width.pow(2) + sprite.height.pow(2)) as f32).sqrt() * self.size;
		let center = (self.coors.0 as f32 + sprite.width as f32 / 2.0, self.coors.1 as f32 + sprite.height as f32 / 2.0);
		
		let x_min = (center.0 - sprite_length);
		let x_max = (center.0 + sprite_length);
		let y_min = (center.1 - sprite_length);
		let y_max = (center.1 + sprite_length);

		let x_min = (x_min.clamp(0.0, image.width as f32) as i32).try_into().unwrap_or(0);
		let x_max = (x_max.clamp(0.0, (image.width-1) as f32) as i32).try_into().unwrap_or(image.width-1);
		let y_min = (y_min.clamp(0.0, image.height as f32) as i32).try_into().unwrap_or(0);
		let y_max = (y_max.clamp(0.0, (image.height-1) as f32) as i32).try_into().unwrap_or(image.height-1);

		return (x_min, x_max, y_min, y_max);
		//return (0, image.width, 0, image.height);
	}

	pub fn assign_best_color(&mut self, goal_image : &MyImage, atlas : &Atlas) {

		//on fait la moyenne pondérée des couleurs de l'image
		let sprite = atlas.get_sprite(self.id);
		let mut red : f32 = 0.0;
		let mut green : f32 = 0.0;
		let mut blue : f32 = 0.0;
		let mut total : f32 = 0.0;

		let (scaled_sprite, x_offset, y_offset) = sprite.scale(self.size);
		let (scaled_sprite, x_offset_r, y_offset_r) = scaled_sprite.rotate(self.rotation);

		let x_offset_true = x_offset + x_offset_r;
		let y_offset_true = y_offset + y_offset_r;

		for y in 0..scaled_sprite.height {
			for x in 0..scaled_sprite.width {

				let pixel = scaled_sprite.pixels[y * scaled_sprite.width + x];
				if pixel.3 == 0 {
					continue;
				}

				let new_x : usize = ((x + self.coors.0 as usize) as isize - x_offset_true as isize).try_into().unwrap_or(0);
				let new_y : usize = ((y + self.coors.1 as usize) as isize - y_offset_true as isize).try_into().unwrap_or(0);

				if new_x >= goal_image.width || new_y >= goal_image.height {
					continue;
				}
					
				let index = new_y * goal_image.width + new_x;
				let goal_pixel = goal_image.pixels[index];
				let sprite_pixel = scaled_sprite.pixels[y * scaled_sprite.width + x];
				
				let (_,_,_, a) = sprite_pixel;
				let (r,g,b,_) = goal_pixel;
				let alpha = uhuit_to_fsoixquatr(a);
				red += uhuit_to_fsoixquatr(r) * alpha;
				green += uhuit_to_fsoixquatr(g) * alpha;
				blue += uhuit_to_fsoixquatr(b) * alpha;
				total += alpha;
			}
		}

		if total == 0.0 {
			return;
		}

		red /= total;
		green /= total;
		blue /= total;

		let (r,g,b) = (
			fsoixquatr_to_uhuit(red),
			fsoixquatr_to_uhuit(green),
			fsoixquatr_to_uhuit(blue)
		);
		self.color = (r,g,b);

	}

	pub fn mutate(&mut self, atlas : &Atlas, canvas : &MyImage) {
		let mut rng = rand::thread_rng();
		//10% de chance de changer de sprite
		if rng.gen_range(0..10) == 0 {
			self.id = atlas.ids[rng.gen_range(0..atlas.nb_sprites)];
		}
		let mut new_rotation = self.rotation as isize + rng.gen_range(-10..10);
		if new_rotation < 0 {
			new_rotation += 360;
		}
		self.rotation = (new_rotation % 360) as usize;
		self.coors = (self.coors.0 + rng.gen_range(-20..20), self.coors.1 + rng.gen_range(-20..20));
		self.coors.0 = self.coors.0.clamp(0, canvas.width as isize);
		self.coors.1 = self.coors.1.clamp(0, canvas.height as isize);
		//self.size *= rng.gen_range(0.8..1.2);
		self.size += rng.gen_range(-0.2..0.2);
		self.size = self.size.clamp(0.03,10.0);
		self.color = (0,0,0);
	}

	pub fn clone_and_mutate(&self, atlas : &Atlas, canvas : &MyImage) -> Object {
		let mut new_object = *self;
		new_object.mutate(atlas, canvas);
		return new_object;
	}

	pub fn difference(&self, other : &Object) -> i64 {
		let mut difference : i64 = 0;
		if self.id != other.id {
			difference += 100;
		}
		difference += ((self.rotation as isize - other.rotation as isize) as f64 * 10.0) as i64; //de base c'est 100 mais on s'en fout car on travaille avec des cercles
		difference += (((self.coors.0 - other.coors.0).abs() + (self.coors.1 - other.coors.1).abs()) as usize * 2) as i64;
		difference += ((self.size - other.size) * 10.0) as i64;
		difference += ((self.color.0 as i64 - other.color.0 as i64).abs() + (self.color.1 as i64 - other.color.1 as i64).abs() + (self.color.2 as i64 - other.color.2 as i64).abs());
		return difference;
	}

}