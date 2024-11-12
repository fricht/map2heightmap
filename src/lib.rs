use image::{ImageBuffer, Luma, Rgba};
use std::collections::HashMap;

const ALLOWED_ERROR: i64 = 100_000;
const RELIEF_COLOR: [i64; 3] = [0, 0, 0];

#[derive(Debug)]
pub struct Region {
    pub relief_lines: Vec<u8>,
}

#[derive(Debug)]
pub struct ReliefLine {
    pub up_region: Option<u8>,
    pub down_region: Option<u8>,
    pub height: Option<i64>,
}

pub enum RelativeHeight {
    Up,
    Down,
}

impl ReliefLine {
    pub fn empty() -> Self {
        ReliefLine {
            up_region: None,
            down_region: None,
            height: None,
        }
    }

    pub fn try_add_region(&mut self, d: u8) -> RelativeHeight {
        if let None = self.up_region {
            self.up_region = Some(d);
            return RelativeHeight::Up;
        }
        if let None = self.down_region {
            self.down_region = Some(d);
            return RelativeHeight::Down;
        }
        panic!("Tried to add a region to a line, but both are already set")
    }

    pub fn get_region_status(&self, region: u8) -> Option<RelativeHeight> {
        if self.up_region == Some(region) {
            Some(RelativeHeight::Up)
        } else if self.down_region == Some(region) {
            Some(RelativeHeight::Down)
        } else {
            None
        }
    }
}

pub fn extract_color(image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let mut relief_mask_image = ImageBuffer::new(image.width(), image.height());

    for (x, y, pixel) in image.enumerate_pixels() {
        let dist = (RELIEF_COLOR[0] - pixel.0[0] as i64).pow(2)
            + (RELIEF_COLOR[1] - pixel.0[1] as i64).pow(2)
            + (RELIEF_COLOR[2] - pixel.0[2] as i64).pow(2);
        let col = if dist <= ALLOWED_ERROR {
            Luma([255])
        } else {
            Luma([1])
        };
        relief_mask_image.put_pixel(x, y, col);
    }

    relief_mask_image
}

pub fn clean_mask(_image: &mut ImageBuffer<Luma<u8>, Vec<u8>>) {
    println!("Mask cleaning not implemented");
}

pub fn separate_regions(
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
) -> (
    ImageBuffer<Luma<u8>, Vec<u8>>,
    HashMap<u8, Region>,
    HashMap<u8, ReliefLine>,
) {
    // clone image
    let mut image = image.clone();

    // initialize mask vec
    let mut height_data: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::new(image.width(), image.height());
    let mut borders = HashMap::new();
    let mut insides = HashMap::new();

    let mut n = 0;

    // detect edges
    for x in 0..image.width() {
        for y in 0..image.height() {
            // if pixel is border, bucket here
            let px = image.get_pixel(x, y).0[0];
            if px == 255 {
                n += 1;
                borders.insert(n, ReliefLine::empty());
                bucket_into(&mut image, (x, y), n, &mut height_data, true);
            }
        }
    }

    // detect regions
    for x in 0..image.width() {
        for y in 0..image.height() {
            // if pixel is region, bucket here
            let px = image.get_pixel(x, y).0[0];
            if px == 1 {
                n += 1;
                insides.insert(
                    n,
                    Region {
                        relief_lines: bucket_into(&mut image, (x, y), n, &mut height_data, false),
                    },
                );
            }
        }
    }

    (height_data, insides, borders)
}

fn bucket_into(
    image: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
    init_pos: (u32, u32),
    color: u8,
    target_image: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
    count_diagonal: bool,
) -> Vec<u8> {
    let col = image.get_pixel(init_pos.0, init_pos.1).0[0];

    let mut positions = vec![init_pos];

    let mut touching_cols = Vec::<u8>::new();

    loop {
        let pos = match positions.pop() {
            Some(p) => p,
            None => break,
        };

        // do nothing if pixel already computed
        if target_image.get_pixel(pos.0, pos.1).0[0] == color {
            continue;
        }

        // do nothing if border reached
        if image.get_pixel(pos.0, pos.1).0[0] != col {
            let px = target_image.get_pixel(pos.0, pos.1).0[0];
            if !touching_cols.contains(&px) {
                touching_cols.push(px);
            }
            continue;
        }

        // otherwise continue
        // color target & erase image
        target_image.put_pixel(pos.0, pos.1, Luma([color]));
        image.put_pixel(pos.0, pos.1, Luma([0]));
        // add next positions
        if pos.0 > 0 {
            positions.push((pos.0 - 1, pos.1));
        }
        if pos.0 < image.width() - 1 {
            positions.push((pos.0 + 1, pos.1));
        }
        if pos.1 > 0 {
            positions.push((pos.0, pos.1 - 1));
        }
        if pos.1 < image.height() - 1 {
            positions.push((pos.0, pos.1 + 1));
        }
        if count_diagonal {
            if pos.0 > 0 {
                if pos.1 > 0 {
                    positions.push((pos.0 - 1, pos.1 - 1));
                }
                if pos.1 < image.height() - 1 {
                    positions.push((pos.0 - 1, pos.1 + 1));
                }
            }
            if pos.0 < image.width() - 1 {
                if pos.1 > 0 {
                    positions.push((pos.0 + 1, pos.1 - 1));
                }
                if pos.1 < image.height() - 1 {
                    positions.push((pos.0 + 1, pos.1 + 1));
                }
            }
        }
    }

    touching_cols
}

pub fn set_heights(regions: &mut HashMap<u8, Region>, heights: &mut HashMap<u8, ReliefLine>) {
    for r in regions.iter() {
        for l in r.1.relief_lines.iter() {
            match heights.get_mut(l) {
                Some(h) => {
                    h.try_add_region(*r.0);
                }
                None => println!("Warning : Height not found, you might get error at some point"),
            }
        }
    }
    /* started this, but it quickly became a nightmare, so as always i'm gonna do random labelling and keep that for later...
    // enougn to not reach 0 (if reach 0 then another err would have occured somewhere else, bc it would mean +255 regions)
    // 1280 = 5 * 256
    let mut to_analyse = vec![(heights.iter_mut().next().expect("No relief lines").1, 1280)];
    while !to_analyse.is_empty() {
        let (line, h) = to_analyse.pop().expect("Rust is broken. Really.");
        line.height = Some(h);
        // upper region
        for l in regions
            .get_mut(
                &line
                    .down_region
                    .expect("Error : aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            )
            .expect("Tired of writing error msg, something is fuched up")
            .relief_lines
            .iter()
        {
            let mut a = heights.get_mut(l).expect("msg"); // this name doesn't mean anything.
            match a.height {
                Some(_) => (),
                None => {
                    to_analyse.push((a, h + 5));
                }
            }
        }
    }
    */// so here is the random version
    let mut h = 0;
    for l in heights.values_mut() {
        h += 5;
        l.height = Some(h);
    }
}

pub fn compute_general_heights(
    map: &ImageBuffer<Luma<u8>, Vec<u8>>,
) -> ImageBuffer<Luma<f32>, Vec<f32>> {
    let mut heightmap = ImageBuffer::new(map.width(), map.height());
    //
    heightmap
}

pub fn get_region_dist(
    map: &ImageBuffer<Luma<u8>, Vec<u8>>,
    pos: (u32, u32),
    col: u8,
) -> Option<f64> {
    let mut dist: Option<u64> = None;
    for x in 0..map.width() {
        for y in 0..map.height() {
            // if pixel is border, bucket here
            let px = map.get_pixel(x, y).0[0];
            if px == col {
                let new_dist =
                    ((x as i64 - pos.0 as i64).pow(2) + (y as i64 - pos.1 as i64).pow(2)) as u64;
                match dist {
                    Some(d) => {
                        dist = Some(u64::min(d, new_dist));
                    }
                    None => {
                        dist = Some(new_dist);
                    }
                }
            }
        }
    }
    match dist {
        Some(d) => Some(f64::sqrt(d as f64)),
        None => None,
    }
}
