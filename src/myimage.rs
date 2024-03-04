use std::convert::TryInto;
use std::cmp::min;

use image::{ImageBuffer, RgbaImage, Rgba, ImageOutputFormat, GenericImageView, DynamicImage, ImageFormat};
use rayon::prelude::*;
use crate::object;
use crate::atlas;

use object::Object;
use atlas::Atlas;

//format rgba8
#[derive(Clone, Debug)]
pub struct MyImage {
	pub pixels : Vec<(u8,u8,u8,u8)>,
	pub width : usize,
	pub height : usize
}

#[inline(always)]
pub fn uhuit_to_fsoixquatr(color : u8) -> f32 {
	return color as f32 / 255.0;
}

#[inline(always)]
pub fn fsoixquatr_to_uhuit(color : f32) -> u8 {
	return (color * 255.0) as u8;
}

#[derive(Clone, Copy)]
struct Barycentre {
	r : u8,
	g : u8,
	b : u8,
	a : u8,
}

#[derive(Clone, Copy)]
struct Pixel {
	r : u8,
	g : u8,
	b : u8,
	a : u8,
}

impl Barycentre {

	fn distance(&self, pixel : &(u8,u8,u8,u8)) -> u64 {
		let r = (self.r as i16 - pixel.0 as i16) as i64;
		let g = (self.g as i16 - pixel.1 as i16) as i64;
		let b = (self.b as i16 - pixel.2 as i16) as i64;
		let a = (self.a as i16 - pixel.3 as i16) as i64;
		return (((r*r) as u64 + (g*g) as u64 + (b*b) as u64 + (a*a) as u64) as f64).sqrt() as u64;
	}
}

impl MyImage {

	pub fn empty(width : usize, height : usize) -> MyImage {
		let pixels = vec![(0,0,0,0); width * height];
		return MyImage {pixels, width, height};
	}

	pub fn coloured(width : usize, height : usize, color : (u8,u8,u8,u8)) -> MyImage {
		let pixels = vec![(color.0,color.1,color.2,color.3); width * height];
		return MyImage {pixels, width, height};
	}
	
	pub fn pix_buf(&self) -> Vec<u8> {
		let mut pixels = Vec::new();
		for y in 0..self.height {
			for x in 0..self.width {
				let pixel = self.pixels[y * self.width + x];
				pixels.push(pixel.0);
				pixels.push(pixel.1);
				pixels.push(pixel.2);
				pixels.push(pixel.3);
			}
		}
		return pixels;
	}

	#[inline(always)]
	pub fn get_pixel(&self, x : usize, y : usize) -> (u8,u8,u8,u8) {
		return self.pixels[y * self.width + x];
	}

	pub fn scale(&self, scale : f32) -> (MyImage, usize, usize) {

		let new_width = (self.width as f32 * scale) as usize;
		let new_height = (self.height as f32 * scale) as usize;

		let x_offset = ((new_width as isize - self.width as isize) as f32 / 2.0) as isize;
		let y_offset = ((new_height as isize - self.height as isize) as f32 / 2.0) as isize;

		let mut new_pixels = vec![(0,0,0,0); new_width * new_height];

		for y in 0..new_height {
			for x in 0..new_width {
				let new_x = (x as f32 / scale) as usize;
				let new_y = (y as f32 / scale) as usize;

				if new_x < self.width && new_y < self.height {
					new_pixels[y * new_width + x] = self.pixels[new_y * self.width + new_x];
				}
			}
		}

		return (MyImage {pixels : new_pixels, width : new_width, height : new_height},
			x_offset as usize, y_offset as usize);

	}


	pub fn rotate(&self, rotation : usize) -> (MyImage, usize, usize) {
		
		let rotation = rotation as f32;
		
		let angle = rotation.to_radians();
		let cosine = angle.cos();
		let sine = angle.sin();

		let new_height = (self.height as f32 * cosine).abs() as usize + (self.width as f32 * sine).abs() as usize + 1;
		let new_width = (self.width as f32 * cosine).abs() as usize + (self.height as f32 * sine).abs() as usize + 1;

		let original_center_x = (self.width as f32 / 2.0) as usize;
		let original_center_y = (self.height as f32 / 2.0) as usize;

		let new_center_x = (new_width as f32 / 2.0) as usize;
		let new_center_y = (new_height as f32 / 2.0) as usize;

		let mut new_pixels = vec![(0,0,0,0); new_width * new_height];

		for y in 0..self.height {
			for x in 0..self.width {
				let y0 = self.height as isize - 1 - y as isize - original_center_y as isize;
				let x0 = self.width as isize - 1 - x as isize - original_center_x as isize;

				let new_y0 = (x0 as f32 * sine + y0 as f32 * cosine).round() as isize;
				let new_x0 = (x0 as f32 * cosine - y0 as f32 * sine).round() as isize;

				let new_y0 = new_center_y as isize - new_y0;
				let new_x0 = new_center_x as isize - new_x0;

				if new_x0 >= 0 && new_y0 >= 0 && new_x0 < new_width as isize && new_y0 < new_height as isize {
					let index = new_y0 as usize * new_width + new_x0 as usize;
					new_pixels[index] = self.pixels[y * self.width + x];
					if index < new_width * new_height - 1 {
						new_pixels[index+1] = self.pixels[y * self.width + x];
					}
				}
			}
		}

		return (MyImage {pixels : new_pixels, width : new_width, height : new_height},
			(new_center_x - original_center_x), (new_center_y - original_center_y));

	}


	pub fn draw_sprite(&mut self, atlas : &Atlas, obj : &Object) {

		let sprite = atlas.get_sprite(obj.id);

		let (scaled_sprite, x_offset, y_offset) = sprite.scale(obj.size);
		let (scaled_sprite, x_offset_r, y_offset_r) = scaled_sprite.rotate(obj.rotation);

		let x_offset_true = x_offset + x_offset_r;
		let y_offset_true = y_offset + y_offset_r;

		for y in 0..scaled_sprite.height {
			for x in 0..scaled_sprite.width {

				//if the pixel is transparent, skip it
				let pixel = scaled_sprite.pixels[y * scaled_sprite.width + x];
				if pixel.3 == 0 {
					continue;
				}

				let new_x : usize = ((x + obj.coors.0 as usize) as isize - x_offset_true as isize).try_into().unwrap_or(0);
				let new_y : usize = ((y + obj.coors.1 as usize) as isize - y_offset_true as isize).try_into().unwrap_or(0);

				if new_x >= self.width || new_y >= self.height {
					continue;
				}
					
				let index = new_y * self.width + new_x;
				let image_pixel = self.pixels[index];
				let sprite_pixel = scaled_sprite.pixels[y * scaled_sprite.width + x];
				
				let sprite_color = (
					uhuit_to_fsoixquatr(obj.color.0) * uhuit_to_fsoixquatr(sprite_pixel.0),
					uhuit_to_fsoixquatr(obj.color.1) * uhuit_to_fsoixquatr(sprite_pixel.1),
					uhuit_to_fsoixquatr(obj.color.2) * uhuit_to_fsoixquatr(sprite_pixel.2),
				);

				let sprite_color = (
					fsoixquatr_to_uhuit(sprite_color.0),
					fsoixquatr_to_uhuit(sprite_color.1),
					fsoixquatr_to_uhuit(sprite_color.2),
					sprite_pixel.3,
				);

				let percentage_of_first_color = sprite_color.3 as f32 / 255.0;
				let percentage_of_second_color = (255 - sprite_color.3) as f32 / 255.0;

				let weighted_red = (sprite_color.0 as f32 * percentage_of_first_color + image_pixel.0 as f32 * percentage_of_second_color) as u8;
				let weighted_green = (sprite_color.1 as f32 * percentage_of_first_color + image_pixel.1 as f32 * percentage_of_second_color) as u8;
				let weighted_blue = (sprite_color.2 as f32 * percentage_of_first_color + image_pixel.2 as f32 * percentage_of_second_color) as u8;				
				let alpha = min(255, sprite_color.3 as u16 + image_pixel.3 as u16) as u8;
				let new_pixel = (weighted_red, weighted_green, weighted_blue, alpha);

				self.pixels[index] = new_pixel;
			}
		}
		
	}

	pub fn scoring(&self, atlas : &Atlas, obj : &Object, goal_MyImage : &MyImage) -> i64 {

		let mut score = 0;
		let sprite = atlas.get_sprite(obj.id);

		let (scaled_sprite, x_offset, y_offset) = sprite.scale(obj.size);
		let (scaled_sprite, x_offset_r, y_offset_r) = scaled_sprite.rotate(obj.rotation);

		let x_offset_true = x_offset + x_offset_r;
		let y_offset_true = y_offset + y_offset_r;

		for y in 0..scaled_sprite.height {
			for x in 0..scaled_sprite.width {

				let pixel = scaled_sprite.pixels[y * scaled_sprite.width + x];
				if pixel.3 == 0 {
					continue;
				}

				let new_x : usize = ((x + obj.coors.0 as usize) as isize - x_offset_true as isize).try_into().unwrap_or(0);
				let new_y : usize = ((y + obj.coors.1 as usize) as isize - y_offset_true as isize).try_into().unwrap_or(0);

				if new_x >= self.width || new_y >= self.height {
					continue;
				}
					
				let index = new_y * self.width + new_x;
				let MyImage_pixel = self.pixels[index];
				let sprite_pixel = scaled_sprite.pixels[y * scaled_sprite.width + x];
				let goal_pixel = goal_MyImage.pixels[index];
				
				let sprite_color = (
					uhuit_to_fsoixquatr(obj.color.0) * uhuit_to_fsoixquatr(sprite_pixel.0),
					uhuit_to_fsoixquatr(obj.color.1) * uhuit_to_fsoixquatr(sprite_pixel.1),
					uhuit_to_fsoixquatr(obj.color.2) * uhuit_to_fsoixquatr(sprite_pixel.2),
				);

				let sprite_color = (
					fsoixquatr_to_uhuit(sprite_color.0),
					fsoixquatr_to_uhuit(sprite_color.1),
					fsoixquatr_to_uhuit(sprite_color.2),
					sprite_pixel.3,
				);

				let percentage_of_first_color = sprite_color.3 as f32 / 255.0;
				let percentage_of_second_color = (255 - sprite_color.3) as f32 / 255.0;

				let weighted_red = (sprite_color.0 as f32 * percentage_of_first_color + MyImage_pixel.0 as f32 * percentage_of_second_color) as u8;
				let weighted_green = (sprite_color.1 as f32 * percentage_of_first_color + MyImage_pixel.1 as f32 * percentage_of_second_color) as u8;
				let weighted_blue = (sprite_color.2 as f32 * percentage_of_first_color + MyImage_pixel.2 as f32 * percentage_of_second_color) as u8;				let alpha = min(255, sprite_color.3 as u16 + MyImage_pixel.3 as u16) as u8;
				let new_pixel = (weighted_red, weighted_green, weighted_blue, alpha);

				//calculate how much did the object contributed to the MyImage
				let diff_before = (
					(MyImage_pixel.0 as i16 - goal_pixel.0 as i16).unsigned_abs() as u64,
					(MyImage_pixel.1 as i16 - goal_pixel.1 as i16).unsigned_abs() as u64,
					(MyImage_pixel.2 as i16 - goal_pixel.2 as i16).unsigned_abs() as u64,
				);
				let diff_before = diff_before.0 + diff_before.1 + diff_before.2;
				let diff_after = (
					(new_pixel.0 as i16 - goal_pixel.0 as i16).unsigned_abs() as u64,
					(new_pixel.1 as i16 - goal_pixel.1 as i16).unsigned_abs() as u64,
					(new_pixel.2 as i16 - goal_pixel.2 as i16).unsigned_abs() as u64,
				);
				let diff_after = diff_after.0 + diff_after.1 + diff_after.2;

				score += diff_before as i64 - diff_after as i64;

			}
		}

		return score;
	}

	pub fn trim(&self) -> MyImage {
		//count how many lines of transparent pixels are there at the top in a row
		let mut top = 0;
		for y in 0..self.height {
			let mut transparent = true;
			for x in 0..self.width {
				let pixel = self.pixels[y * self.width + x];
				if pixel.3 != 0 {
					transparent = false;
					break;
				}
			}
			if transparent {
				top += 1;
			} else {
				break;
			}
		}

		//count how many lines of transparent pixels are there at the bottom in a row
		let mut bottom = 0;
		for y in (0..self.height).rev() {
			let mut transparent = true;
			for x in 0..self.width {
				let pixel = self.pixels[y * self.width + x];
				if pixel.3 != 0 {
					transparent = false;
					break;
				}
			}
			if transparent {
				bottom += 1;
			} else {
				break;
			}
		}

		//count how many columns of transparent pixels are there at the left in a row
		let mut left = 0;
		for x in 0..self.width {
			let mut transparent = true;
			for y in 0..self.height {
				let pixel = self.pixels[y * self.width + x];
				if pixel.3 != 0 {
					transparent = false;
					break;
				}
			}
			if transparent {
				left += 1;
			} else {
				break;
			}
		}

		//count how many columns of transparent pixels are there at the right in a row
		let mut right = 0;
		for x in (0..self.width).rev() {
			let mut transparent = true;
			for y in 0..self.height {
				let pixel = self.pixels[y * self.width + x];
				if pixel.3 != 0 {
					transparent = false;
					break;
				}
			}
			if transparent {
				right += 1;
			} else {
				break;
			}
		}

		let min_trim = min(top, min(bottom, min(left, right)));

		let new_width = self.width - min_trim * 2;
		let new_height = self.height - min_trim * 2;

		let mut new_pixels = vec![(0,0,0,0); new_width * new_height];

		for y in 0..new_height {
			for x in 0..new_width {
				let index = y * new_width + x;
				let old_index = (y + min_trim) * self.width + (x + min_trim);
				new_pixels[index] = self.pixels[old_index];
			}
		}

		return MyImage {pixels : new_pixels, width : new_width, height : new_height};
	}


	pub fn difference(&self, other : &MyImage) -> u64 {
		let mut difference = 0;
		for y in 0..self.height {
			for x in 0..self.width {
				let index = y * self.width + x;
				let pixel = self.pixels[index];
				let other_pixel = other.pixels[index];
				difference += (pixel.0 as i16 - other_pixel.0 as i16).unsigned_abs() as u64;
				difference += (pixel.1 as i16 - other_pixel.1 as i16).unsigned_abs() as u64;
				difference += (pixel.2 as i16 - other_pixel.2 as i16).unsigned_abs() as u64;
				//difference += (pixel.3 as i16 - other_pixel.3 as i16).abs() as u64;
			}
		}
		return difference;
	}


	pub fn from_path(path : &str) -> MyImage {
		println!("Loading MyImage... {}", path);
		let img = image::open(path).unwrap().to_rgba8();
		let (width, height) = img.dimensions();
		let pixels = img.pixels().map(|x| (x[0], x[1], x[2], x[3])).collect();
		return MyImage {pixels, width : width as usize, height : height as usize};
	}

	pub fn find_best_bg_color(&self) -> (u8,u8,u8,u8) {
		
		const GROUPS : usize = 20;
		const ITERATIONS : usize = 20;
		
		//generate random colors
		let mut groups : Vec<(Barycentre, Vec<Pixel>)> = vec![
				(Barycentre {
					r : rand::random::<u8>(),
					g : rand::random::<u8>(),
					b : rand::random::<u8>(),
					a : rand::random::<u8>(),
				}, Vec::new()) 
				;GROUPS
			];

		for i in 1..=ITERATIONS {

			println!("Iteration {}/{}", i, ITERATIONS);
			//for each pixel, find the closest barycentre
			for y in 0..self.height {
				for x in 0..self.width {
					let pixel = self.pixels[y * self.width + x];
					let mut best_group = 0;
					let mut best_distance = 0;
					for (i, group) in groups.iter().enumerate() {
						let distance = group.0.distance(&pixel);
						if i == 0 || distance < best_distance {
							best_distance = distance;
							best_group = i;
						}
					}
					groups[best_group].1.push(Pixel {
						r : pixel.0,
						g : pixel.1,
						b : pixel.2,
						a : pixel.3,
					});
				}
			}

			//calculate the new barycentre
			groups.par_iter_mut().for_each(|group| {

				if group.1.is_empty() {
					//give it a new random color
					group.0.r = rand::random::<u8>();
					group.0.g = rand::random::<u8>();
					group.0.b = rand::random::<u8>();
					group.0.a = rand::random::<u8>();
					return;
				}
				let mut r = 0;
				let mut g = 0;
				let mut b = 0;
				let mut a = 0;
				for pixel in &group.1 {
					r += pixel.r as u64;
					g += pixel.g as u64;
					b += pixel.b as u64;
					a += pixel.a as u64;
				}
				let len = group.1.len() as u64;
				group.0.r = (r / len) as u8;
				group.0.g = (g / len) as u8;
				group.0.b = (b / len) as u8;
				group.0.a = (a / len) as u8;
				if i != ITERATIONS {
					group.1.clear();
				}
			});
		}

		//find the group with the most pixels
		let mut best_group = 0;
		let mut best_len = 0;
		for (i, group) in groups.iter().enumerate() {
			if i == 0 || group.1.len() > best_len {
				best_len = group.1.len();
				best_group = i;
			}
		}
		return unsafe { std::mem::transmute(groups[best_group].0) };
	}
}