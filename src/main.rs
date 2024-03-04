#![allow(unused_parens, clippy::needless_return)]

/* This program was made to geometrise an image.
Approximating it with circles */

// compilation command : cargo +nightly run --release -Z profile-rustflags 197 10 128

mod myimage;
mod object;
mod atlas;
use myimage::{MyImage, uhuit_to_fsoixquatr, fsoixquatr_to_uhuit};
use object::Object;
use atlas::Atlas;

//mpsc : multi-producer single-consumer
use std::sync::mpsc::{self, Receiver, Sender};

use image::{save_buffer_with_format, load_from_memory_with_format};

use std::time::Instant;
use std::io::Write;
use std::cmp::min;
use std::fs;
use rand::Rng;
use rayon::prelude::*;
//use vid2img::FileSource;
use std::path::Path;
//use rgb_hsv::{rgb_to_hsv, hsv_to_rgb};

#[derive(Clone, Copy)]
struct Competitor {
      object : Object,
      score : i64
}

#[derive(Clone, Copy)]
struct Adaptator {
	object : Object,
	original : Object,
	score : i64
}


/*fn geometrise(image : &MyImage, atlas : &Atlas, parameters : (usize,usize,usize), background_colour : (u8,u8,u8,u8)) -> (MyImage,Vec<Object>) {

	let nb_objects = parameters.0;
	let nb_evolutions = parameters.1;
	let nb_candidates = parameters.2;

	let internal_mutations = 3;

	let mut best_objects = Vec::with_capacity(nb_objects);
	let mut canvas = MyImage::coloured(image.width, image.height, background_colour);
	let mut scores = Vec::with_capacity(nb_candidates);

	//create a vector of 30 mpsc channels
	//the main thread will send the objects to the threads for them to calculate the score
	//the main thread will then receive the objects with their score
	//the main thread will then sort the objects by score and keep the best ones
	const NB_THREADS : usize = 30;
	let (main_tx, main_rx) = mpsc::channel();
	//let threads: Vec<(mpsc::Sender<Object>, Receiver<Object>)> = (0..NB_THREADS).into_iter().map(|_| mpsc::channel()).collect::<Vec<_>>();

	let mut handles = Vec::with_capacity(NB_THREADS);
	let image_ptr = image as *const MyImage as usize;
	let atlas_ptr = atlas as *const Atlas as usize;
	let canvas_ptr = (&canvas) as *const MyImage as usize;
	for i in 0..NB_THREADS {
		let (tx, rx) : (Sender<(Object, i64)>, Receiver<(Object, i64)>) = mpsc::channel();
		let main_tx = main_tx.clone();
		
		unsafe {
			let rx_ptr = (&rx) as *const Receiver<(Object, i64)> as usize;
			handles.push(((tx,rx), std::thread::spawn(move || {
				loop {
					let new_rx = rx_ptr as *const Receiver<(Object, i64)>;
					match (*new_rx).try_recv() {
						Ok((object, _)) => {
							println!("thread {} recieved an object", i);
							let new_canvas = canvas_ptr as *const MyImage;
							let score = (*new_canvas).scoring((atlas_ptr as *const Atlas).as_ref().unwrap(), &object, (image_ptr as *const MyImage).as_ref().unwrap());
							main_tx.send((object, score)).unwrap();
						}
						Err(_) => { 
							println!("thread {} didn't receive an object", i);
							std::thread::sleep(std::time::Duration::from_millis(100)); 
						}
					}
				}
			})));
		}
	}
	for o in 0..nb_objects {

		let mut candidates = Object::generate_randoms(nb_candidates, atlas, &canvas);

		for _e in 0..nb_evolutions {

			println!("{}% done : {} evolutions done", ((o+1) as f32 / nb_objects as f32) * 100.0, _e+1);
			
			//generate random objects
			for c in 0..nb_candidates {
				println!("sending object {} to thread {}", c, c%NB_THREADS);
				let candidate = candidates[c].clone();
				//send it to the c%NB_THREADS thread
				handles[c%NB_THREADS].0.0.send((candidate, 0)).unwrap();
				println!("object {} sent to thread {}", c, c%NB_THREADS);
			}

			//receive the objects with their score
			/*for i in 0..nb_candidates {
				let (object, score) = handles[i%NB_THREADS].0.1.recv().unwrap();
				scores.push(Competitor {object, score});
			}*/

			let mut total_received = 0;
			while total_received < nb_candidates {
				//try to receive an object
				match main_rx.try_recv() {
					Ok((object, score)) => {
						scores.push(Competitor {object, score});
						total_received += 1;
					},
					Err(_) => { std::thread::sleep(std::time::Duration::from_millis(100)); }
				}
				println!("{} objects received", total_received);
			}
			

			scores.sort_by(|a,b| b.score.partial_cmp(&a.score).unwrap());
			candidates.clear();

			for obj in scores.iter().map(|s| s.object).take(nb_candidates / 9 + 1) {
				candidates.push(obj);
				for _ in 0..8 {
					candidates.push(obj.clone_and_mutate(atlas, &canvas));
				}
			}

			//println!("{:.1}::{:.1}% done : {} evolutions done", ((o+1) as f32 / nb_objects as f32) * 100.0, ((e+1) as f32 / nb_evolutions as f32) * 100.0, e+1);
		}

		println!("{:.2}% done : {} objects total", ((o+1) as f32 / nb_objects as f32) * 100.0, o+1);
		let best = &scores[0].object;
		best_objects.push(*best);
		canvas.draw_sprite(atlas, best);
		scores.clear();
		
	}

	return (canvas, best_objects);
}*/

/*fn geometrise(image : &MyImage, atlas : &Atlas, parameters : (usize,usize,usize), background_colour : (u8,u8,u8,u8)) -> (MyImage,Vec<Object>) {

	let nb_objects = parameters.0;
	let nb_evolutions = parameters.1;
	let nb_candidates = parameters.2;

	let internal_mutations = 3;

	let mut best_objects = Vec::with_capacity(nb_objects);
	let mut canvas = MyImage::coloured(image.width, image.height, background_colour);
	//let mut scores = Vec::with_capacity(nb_candidates);

	//create a vector of 30 mpsc channels
	//the main thread will send 64 integers to the threads for printing them
	//the main thread will then receive the integers but negatives
	//the main thread will then print them

	const NB_THREADS : usize = 30;
	let (main_tx, main_rx) = mpsc::channel();
	
	let mut handles = Vec::with_capacity(NB_THREADS);

	let image_ptr = image as *const MyImage as usize;
	let atlas_ptr = atlas as *const Atlas as usize;
	let canvas_ptr = (&canvas) as *const MyImage as usize;

	for i in (0..NB_THREADS) {
		let (tx, rx) = mpsc::channel::<(Object,i64)>();
		let main_tx = main_tx.clone();

		handles.push((tx, std::thread::spawn(move || {
			loop {
				match rx.try_recv() {
					Ok((object, _)) => {
						//println!("thread {} received {}", i, n);
						
						unsafe {//calculate the score
							let new_canvas = canvas_ptr as *const MyImage;
							let score = (*new_canvas).scoring((atlas_ptr as *const Atlas).as_ref().unwrap(), &object, (image_ptr as *const MyImage).as_ref().unwrap());
							main_tx.send((object, score)).unwrap();
						}
					},
					Err(_) => { std::thread::sleep(std::time::Duration::from_millis(100)); }
				}
			}
		})));
	}

	//send 64 integers to the threads
	let mut candidates = Object::generate_randoms(nb_candidates, atlas, &canvas);

	for i in (0..64) {
		println!("sending {}", i);
		handles[(i as usize)%NB_THREADS].0.send((candidates[i], 0)).unwrap();
		println!("sent {}", i);
	}

	//receive the integers but negatives
	for i in (0..64) {
		let (object, score) = main_rx.recv().unwrap();
		println!("{} : {}", i, score);
	}


	return (canvas, best_objects);
}*/

/*fn geometrise(image : &MyImage, atlas : &Atlas, parameters : (usize,usize,usize), background_colour : (u8,u8,u8,u8)) -> (MyImage,Vec<Object>) {

	let nb_objects = parameters.0;
	let nb_evolutions = parameters.1;
	let nb_candidates = parameters.2;

	let internal_mutations = 3;

	let mut best_objects = Vec::with_capacity(nb_objects);
	let mut canvas = MyImage::coloured(image.width, image.height, background_colour);
	//let mut scores = Vec::with_capacity(nb_candidates);

	//create a vector of 30 mpsc channels
	//the main thread will send 64 integers to the threads for printing them
	//the main thread will then receive the integers but negatives
	//the main thread will then print them

	const NB_THREADS : usize = 30;
	let (main_tx, main_rx) = mpsc::channel();
	
	let mut handles = Vec::with_capacity(NB_THREADS);

	let image_ptr = image as *const MyImage as usize;
	let atlas_ptr = atlas as *const Atlas as usize;
	let canvas_ptr = (&canvas) as *const MyImage as usize;

	for i in (0..NB_THREADS) {
		let (tx, rx) = mpsc::channel::<(Object,i64)>();
		let main_tx = main_tx.clone();

		handles.push((tx, std::thread::spawn(move || {
			loop {
				match rx.try_recv() {
					Ok((object, _)) => {
						//println!("thread {} received {}", i, n);
						
						unsafe {//calculate the score
							let new_canvas = canvas_ptr as *const MyImage;
							//find the best color for the object
							let mut new_object = object.clone();
							new_object.assign_best_color((image_ptr as *const MyImage).as_ref().unwrap(), (atlas_ptr as *const Atlas).as_ref().unwrap());
							let score = (*new_canvas).scoring((atlas_ptr as *const Atlas).as_ref().unwrap(), &new_object, (image_ptr as *const MyImage).as_ref().unwrap());
							main_tx.send((new_object, score)).unwrap();
						}
					},
					Err(_) => { std::thread::sleep(std::time::Duration::from_millis(100)); }
				}
			}
		})));
	}

	//send 64 integers to the threads
	for o in 0..nb_objects {

		let mut candidates = Object::generate_randoms(nb_candidates, atlas, &canvas);

		for i in (0..64) {
			//println!("sending {}", i);
			handles[(i as usize)%NB_THREADS].0.send((candidates[i], 0)).unwrap();
			//println!("sent {}", i);
		}

		let mut scores = Vec::with_capacity(nb_candidates);

		//receive the integers but negatives
		for i in (0..64) {
			let (object, score) = main_rx.recv().unwrap();
			scores.push(Competitor {object, score});
			//println!("{} : {}", i, score);
		}

		//find the best object
		let mut best_score = 0;
		let mut best_object = candidates[0].clone();
		for s in scores {
			if s.score > best_score {
				best_score = s.score;
				best_object = s.object;
			}
		}

		canvas.draw_sprite(atlas, &best_object);

		best_objects.push(best_object);
	}

	return (canvas, best_objects);
}*/

const NB_THREADS : usize = 8;

fn geometrise(image : &MyImage, atlas : &Atlas, parameters : (usize,usize,usize), background_colour : (u8,u8,u8,u8)) -> (MyImage,Vec<Object>) {

	let nb_objects = parameters.0;
	let nb_evolutions = parameters.1;
	let nb_candidates = parameters.2;

	let internal_mutations = 3;

	let mut best_objects = Vec::with_capacity(nb_objects);
	let mut canvas = MyImage::coloured(image.width, image.height, background_colour);
	//let mut scores = Vec::with_capacity(nb_candidates);

	//create a vector of 30 mpsc channels
	//the main thread will send 64 integers to the threads for printing them
	//the main thread will then receive the integers but negatives
	//the main thread will then print them

	let (main_tx, main_rx) = mpsc::channel();
	
	let mut handles = Vec::with_capacity(NB_THREADS);

	let image_ptr = image as *const MyImage as usize;
	let atlas_ptr = atlas as *const Atlas as usize;
	let canvas_ptr = (&canvas) as *const MyImage as usize;

	for i in (0..NB_THREADS) {
		let (tx, rx) = mpsc::channel::<(Object,i64)>();
		let main_tx = main_tx.clone();

		handles.push((tx, std::thread::spawn(move || {
			loop {
				match rx.try_recv() {
					Ok((object, _)) => {
						//println!("thread {} received {}", i, n);
						
						unsafe {
							//calculate the score
							let mut best_object = object.clone();
							let mut new_object = object.clone();
							let mut max_score = 0;

							let new_image = (image_ptr as *const MyImage).as_ref().unwrap(); // Wow wtf is that FIXME
							let new_atlas = (atlas_ptr as *const Atlas).as_ref().unwrap();
							let new_canvas = canvas_ptr as *const MyImage;
							for _ in 0..internal_mutations {
								new_object.mutate(new_atlas, &(*new_canvas));
								new_object.assign_best_color(new_image, new_atlas);
								let score = (*new_canvas).scoring(new_atlas, &new_object, new_image);
								if score > max_score {
									max_score = score;
									best_object = new_object.clone();
								}
							}

							main_tx.send((best_object, max_score)).unwrap();
						}
					},
					Err(_) => { std::thread::sleep(std::time::Duration::from_millis(1)); }
				}
			}
		})));
	}

	//send 64 integers to the threads

	let mut candidates = Object::generate_randoms_stepped(nb_candidates, atlas, 0, &canvas);
	let mut scores = Vec::with_capacity(nb_candidates);

	for o in 0..nb_objects {

		Object::refill_randoms_stepped(&mut candidates, atlas, o, &canvas);

		//let mut best_object = candidates[0].clone();
		//let mut best_score = 0;

		for e in 0..nb_evolutions {
			for i in (0..nb_candidates) {
				//println!("sending {}", i);
				handles[(i as usize)%NB_THREADS].0.send((candidates[i], 0)).unwrap();
				//println!("sent {}", i);
			}

			//let mut scores = Vec::with_capacity(nb_candidates);
			scores.clear();
			//receive the integers but negatives
			for i in (0..nb_candidates) {
				let (object, score) = main_rx.recv().unwrap();
				scores.push(Competitor {object, score});
				//println!("{} : {}", i, score);
			}

			//find the best object
			scores.sort_by(|a,b| b.score.partial_cmp(&a.score).unwrap());
			candidates.clear();

			for obj in scores.iter().map(|s| s.object).take(nb_candidates / 9 + 1) {
				candidates.push(obj);
				for _ in 0..8 {
					candidates.push(obj.clone_and_mutate(atlas, &canvas));
				}
			}

		}

		println!("{:.2}% done : {} objects total", ((o+1) as f32 / nb_objects as f32) * 100.0, o+1);
		
		let best = &scores[0].object;
		best_objects.push(*best);
		canvas.draw_sprite(atlas, best);
		scores.clear();

	}

	return (canvas, best_objects);
}



fn geometrise_old(image : &MyImage, atlas : &Atlas, parameters : (usize,usize,usize), background_colour : (u8,u8,u8,u8)) -> (MyImage,Vec<Object>) {

	//println!("Geometrising image...");

	let nb_objects = parameters.0;
	let nb_evolutions = parameters.1;
	let nb_candidates = parameters.2;

	let internal_mutations = 3;

	let mut best_objects = Vec::with_capacity(nb_objects);
	let mut canvas = MyImage::coloured(image.width, image.height, background_colour);
	let mut scores = Vec::with_capacity(nb_candidates);

	for o in 0..nb_objects {
		
		let mut candidates = Object::generate_randoms_stepped(nb_candidates, atlas, o, &canvas);

		for _e in 0..nb_evolutions {
			
			scores = (candidates).par_iter_mut().map(|c| {

                        c.assign_best_color(image, atlas);
                        let mut max_score = canvas.scoring(atlas, c, image);
				//alter the score depending on how much more red the object has than blue
				let (r,g,b) = c.color;
				max_score /= ((r as i64 - b as i64).abs() + 1);

				let mut best_object = c.clone();
                        //return Competitor {score, object : *c};
				//let m
				for m in 0..internal_mutations {
					c.mutate(atlas, &canvas);
					c.assign_best_color(image, atlas);
					let score = canvas.scoring(atlas, c, image);
					if score > max_score {
						max_score = score;
						best_object = c.clone();
					}
				}
				Competitor {score : max_score, object : best_object}

                  }).collect();

			scores.sort_by(|a,b| b.score.partial_cmp(&a.score).unwrap());
			candidates.clear();

			for obj in scores.iter().map(|s| s.object).take(nb_candidates / 9 + 1) {
				candidates.push(obj);
				for _ in 0..8 {
					candidates.push(obj.clone_and_mutate(atlas, &canvas));
				}
			}

			//println!("{:.1}::{:.1}% done : {} evolutions done", ((o+1) as f32 / nb_objects as f32) * 100.0, ((e+1) as f32 / nb_evolutions as f32) * 100.0, e+1);
		}

		println!("{:.2}% done : {} objects total", ((o+1) as f32 / nb_objects as f32) * 100.0, o+1);
		let best = &scores[0].object;
		best_objects.push(*best);
		canvas.draw_sprite(atlas, best);
		scores.clear();
		
	}

	return (canvas, best_objects);
}

fn geometrise_scaled(image : &MyImage, atlas : &Atlas, parameters : (usize,usize,usize), background_colour : (u8,u8,u8,u8)) -> (MyImage,Vec<Object>) {

	//println!("Geometrising image...");

	let nb_objects = parameters.0;
	let nb_evolutions = parameters.1;
	let nb_candidates = parameters.2;

	let internal_mutations = 3;

	let mut best_objects = Vec::with_capacity(nb_objects);
	let mut canvas = MyImage::coloured(image.width, image.height, background_colour);
	let mut scores = Vec::with_capacity(nb_candidates);

	for o in 0..nb_objects {
		
		let mut candidates = Object::generate_randoms_stepped(nb_candidates, atlas, o, &canvas);

		for _e in 0..nb_evolutions {
			
			scores = (candidates).par_iter_mut().map(|c| {

                        c.assign_best_color(image, atlas);
                        let mut max_score = canvas.scoring(atlas, c, image);
				let mut best_object = c.clone();
                        //return Competitor {score, object : *c};
				//let m
				for m in 0..internal_mutations {
					c.mutate(atlas, &canvas);
					c.assign_best_color(image, atlas);
					let score = canvas.scoring(atlas, c, image);
					if score > max_score {
						max_score = score;
						best_object = c.clone();
					}
				}
				Competitor {score : max_score, object : best_object}

                  }).collect();

			scores.sort_by(|a,b| b.score.partial_cmp(&a.score).unwrap());
			candidates.clear();

			for obj in scores.iter().map(|s| s.object).take(nb_candidates / 9 + 1) {
				candidates.push(obj);
				for _ in 0..8 {
					candidates.push(obj.clone_and_mutate(atlas, &canvas));
				}
			}

			//println!("{:.1}::{:.1}% done : {} evolutions done", ((o+1) as f32 / nb_objects as f32) * 100.0, ((e+1) as f32 / nb_evolutions as f32) * 100.0, e+1);
		}

		println!("{:.2}% done : {} objects total", ((o+1) as f32 / nb_objects as f32) * 100.0, o+1);
		let best = &scores[0].object;
		best_objects.push(*best);
		canvas.draw_sprite(atlas, best);
		scores.clear();
		
	}

	return (canvas, best_objects);
}


fn adapt_to_next_frame(image : &MyImage, atlas : &Atlas, old_objects : &Vec<Object>, parameters : (usize,usize,usize), background_colour : (u8,u8,u8,u8)) -> (MyImage,Vec<Object>) {

	let nb_evolutions = parameters.0;
	let nb_candidates = parameters.1;
	let nb_objects = parameters.2;

	let internal_mutations = 4;

	let mut new_objects = Vec::with_capacity(nb_objects);
	let mut canvas = MyImage::coloured(image.width, image.height, background_colour);

	for o in 0..nb_objects {
		
		let mut candidates = old_objects.clone();
		//if the candidates are not fully filled, fill them with random objects from that same vector
		while candidates.len() < nb_candidates {
			candidates.push(old_objects[rand::thread_rng().gen_range(0..old_objects.len())].clone());
		}

		let mut scores : Vec<Adaptator> = (candidates).iter().map(|c| {
			Adaptator {
				object : c.clone(),
				original : c.clone(),
				score : 0
			}
		}).collect();

		for _e in 0..nb_evolutions {
			
			scores.par_iter_mut().for_each(|c| {
				c.object.assign_best_color(image, atlas);
                        let mut max_score = canvas.scoring(atlas, &c.object, image) - (c.object.difference(&c.original) as f32 * 0.05) as i64;
				let mut best_object = c.object.clone();
                        //return Competitor {score, object : *c};
				//let m
				for m in 0..internal_mutations {
					c.object.mutate(atlas, &canvas);
					c.object.assign_best_color(image, atlas);
					let score = canvas.scoring(atlas, &c.object, image) - (c.object.difference(&c.original) as f32 * 0.2) as i64;
					if score > max_score {
						max_score = score;
						best_object = c.object.clone();
					}
				}
				c.score = max_score;
				c.object = best_object;
			});

			scores.sort_by(|a,b| b.score.partial_cmp(&a.score).unwrap());
			candidates.clear();

			for obj in scores.iter().map(|s| s.object).take(nb_candidates / 9 + 1) {
				candidates.push(obj);
				for _ in 0..8 {
					candidates.push(obj.clone_and_mutate(atlas, &canvas));
				}
			}

			//println!("{:.1}::{:.1}% done : {} evolutions done", ((o+1) as f32 / nb_objects as f32) * 100.0, ((e+1) as f32 / nb_evolutions as f32) * 100.0, e+1);

		}

		println!("{:.2}% done : {} objects total", ((o+1) as f32 / old_objects.len() as f32) * 100.0, o+1);

		let best = scores[0].object.clone();
		new_objects.push(best);

		canvas.draw_sprite(atlas, &best);

	}

	return (canvas, new_objects);
}


fn save_objects(objects : &Vec<Object>, filename : &str) {

	let mut file = std::fs::File::create(filename).unwrap();

	for obj in objects {
		let line = format!("{},{},({}|{}),{},[{}|{}|{}]\n", obj.id, obj.rotation, obj.coors.0, obj.coors.1, obj.size, obj.color.0, obj.color.1, obj.color.2);
		file.write_all(line.as_bytes()).unwrap();
	}

	file.flush().unwrap();
}

fn main() {

	// TODO : clean that up
	rayon::ThreadPoolBuilder::new().num_threads(NB_THREADS).build_global().unwrap();

	let args : Vec<String> = std::env::args().collect();
	let nb_objects = args[1].parse::<usize>().unwrap();
	let nb_evolutions = args[2].parse::<usize>().unwrap();
	let nb_candidates = args[3].parse::<usize>().unwrap();

	let image_to_geometrise: MyImage = MyImage::from_path("src\\video\\ng.png");
	let (image_to_geometrise, _, _) = image_to_geometrise.scale(2.0);

	let atlas = Atlas::load_trim(&[
		"src\\objects\\1764.png", 
		"src\\objects\\1765.png",
		"src\\objects\\1766.png",
		"src\\objects\\1767.png",
		"src\\objects\\1768.png",
		"src\\objects\\917.png",
		"src\\objects\\1608.png",
		"src\\objects\\1609.png",
		"src\\objects\\1610.png",
		"src\\objects\\1753.png",
	]);

	println!("Finding best background color...");
	//let background_color = image_to_geometrise.find_best_bg_color();
	let background_color = (255u8, 255u8, 255u8, 255u8);
	println!("Background color found : {:?}", background_color);

	/*println!("Geometrising image... (old algorithm)");
	let time_to_geometrise = Instant::now();
	let (geometrised_image, objects) = geometrise_old(&image_to_geometrise, &atlas, (nb_objects, nb_evolutions, nb_candidates), background_color);
	println!("Geometrised image in {}.{:03} seconds", time_to_geometrise.elapsed().as_secs(), time_to_geometrise.elapsed().subsec_millis());

	save_buffer_with_format("src\\video\\result\\coin_coin_old.png", 
		&geometrised_image.pix_buf(), 
		geometrised_image.width as u32,
		geometrised_image.height as u32,
		image::ColorType::Rgba8,
		image::ImageFormat::Png).unwrap();
	save_objects(&objects, "src\\video\\result\\coin_coin_old.txt");*/

	println!("Geometrising image... (new algorithm)");
	let time_to_geometrise = Instant::now();
	let (geometrised_image, objects) = geometrise(&image_to_geometrise, &atlas, (nb_objects, nb_evolutions, nb_candidates), background_color);
	println!("Geometrised image in {}.{:03} seconds", time_to_geometrise.elapsed().as_secs(), time_to_geometrise.elapsed().subsec_millis());

	save_buffer_with_format("src\\video\\result\\ng.png", 
		&geometrised_image.pix_buf(), 
		geometrised_image.width as u32,
		geometrised_image.height as u32,
		image::ColorType::Rgba8,
		image::ImageFormat::Png).unwrap();
	save_objects(&objects, "src\\video\\result\\ng.txt");

	/*for i in (6..=20).into_iter().step_by(4) {

		//println!("i : {}", nb_objects + 8 * i);
		
		let image_to_geometrise = Image::from_path(&format!("src\\video\\project_{}.png", i));
		//let (image_to_geometrise, _, _) = image_to_geometrise.scale(1.0);

		let time_to_adapt = Instant::now();

		let nb_objects = nb_objects + 3 * i;

		(geometrised_image, objects) = adapt_to_next_frame(&image_to_geometrise, &atlas, &objects, (nb_evolutions, nb_candidates, nb_objects), (255u8,0u8,0u8));

		save_buffer_with_format(&format!("src\\video\\result\\project_{}.png", i), 
			&geometrised_image.pix_buf(), 
			geometrised_image.width as u32,
			geometrised_image.height as u32,
			image::ColorType::Rgba8,
			image::ImageFormat::Png).unwrap();
		save_objects(&objects, &format!("src\\video\\result\\project_{}.txt", i));
		
		println!("Adapted frame {} in {}.{:03} seconds", i, time_to_adapt.elapsed().as_secs(), time_to_adapt.elapsed().subsec_millis());

		println!("Frame {} done", i);
	}*/

	//println!("Done in {}:{}.{}", time.elapsed().as_secs()/60, time.elapsed().as_secs()%60, time.elapsed().subsec_millis());
}