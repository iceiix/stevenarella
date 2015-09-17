
pub struct Atlas {
	width: usize,
	height: usize,	
	free_space: Vec<Rect>,
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
	pub x: usize,
	pub y: usize,
	pub width: usize,
	pub height: usize,
}

impl Atlas {
	pub fn new(width: usize, height: usize) -> Atlas {
		let mut a = Atlas {
			width: width,
			height: height,
			free_space: Vec::new(),
		};
		a.free_space.push(Rect{
			x: 0, y: 0,
			width: width, height: height,
		});
		a
	}

	pub fn add(&mut self, width: usize, height: usize) -> Option<Rect> {
		let mut priority = usize::max_value();
		let mut target: Option<Rect> = None;
		let mut index = 0;
		let mut target_index = 0;
		// Search through and find the best fit for this texture
		for free in &self.free_space {
			if free.width >= width && free.height >= height {
				let current_priority = (free.width - width) * (free.height - height);
				if target.is_none() || current_priority < priority {
					target = Some(*free);
					priority = current_priority;
					target_index = index;
				}
				// Perfect match, we can break early
				if priority == 0 {
					break;
				}
			}
			index += 1;
		}
		if target.is_none() {
			return None;
		}
		let mut t = target.unwrap();
		let ret = Rect{
			x: t.x, y: t.y,
			width: width, height: height,
		};

		if width == t.width {
			t.y += height;
			t.height -= height;
			if t.height == 0 {
				// Remove empty sections
				self.free_space.remove(target_index);
			} else {
				self.free_space[target_index] = t;
			}
		} else {
			if t.height > height {
				// Split by height
				self.free_space.insert(0, Rect{
					x: t.x, y: t.y + height,
					width: width, height: t.height - height,
				});
				target_index += 1;
			}
			t.x += width;
			t.width -= width;
			self.free_space[target_index] = t;
		}

		Some(ret)
	}
}