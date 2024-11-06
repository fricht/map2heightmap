use clap::Parser;
use image::{open, ImageBuffer, Luma, Rgba};

#[derive(Parser, Debug)]
struct Cli {
    imgpath: std::path::PathBuf,
    #[arg(short, long)]
    debug: bool,
}

const ALLOWED_ERROR: i64 = 100_000;
const RELIEF_COLOR: [i64; 3] = [0, 0, 0];

fn main() {
    // parse args
    let args = Cli::parse();
    println!("Using image: {:?}", args.imgpath);
    if args.debug {
        println!("Debug mode enabled");
    }

    // load image
    let image = match open(args.imgpath) {
        Ok(it) => it,
        Err(err) => panic!("{}", err),
    }
    .to_rgba8();

    // extract relief color into mask
    let mut relief_mask_image = extract_color(&image);

    // save mask (if debug)
    if args.debug {
        println!("Saving raw mask at raw_mask.png");
        relief_mask_image
            .save("raw_mask.png")
            .expect("Failed to save image.");
    }

    // clean mask
    clean_mask(&mut relief_mask_image);

    // save mask (if debug)
    if args.debug {
        println!("Saving cleanned mask at mask.png");
        relief_mask_image
            .save("mask.png")
            .expect("Failed to save image.");
    }

    let (regions, insides, borders) = separate_regions(&relief_mask_image);

    // save regions (if debug)
    if args.debug {
        println!("Saving regions at regions.png");
        regions.save("regions.png").expect("Failed to save image.");
    }
}

fn extract_color(image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> ImageBuffer<Luma<u8>, Vec<u8>> {
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

fn clean_mask(_image: &mut ImageBuffer<Luma<u8>, Vec<u8>>) {
    println!("Mask cleaning not implemented");
}

fn separate_regions(
    image: &ImageBuffer<Luma<u8>, Vec<u8>>,
) -> (ImageBuffer<Luma<u8>, Vec<u8>>, Vec<u8>, Vec<u8>) {
    // clone image
    let mut image = image.clone();

    // initialize mask vec
    let mut height_data: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::new(image.width(), image.height());
    let mut insides = Vec::<u8>::new();
    let mut borders = Vec::<u8>::new();

    let mut n = 0;

    // detect edges
    for x in 0..image.width() {
        for y in 0..image.height() {
            // if pixel is not already clear, bucket here
            let px = image.get_pixel(x, y).0[0];
            if px != 0 {
                n += 1;
                let diag;
                if px == 255 {
                    diag = true;
                    insides.push(n);
                } else {
                    diag = false;
                    borders.push(n);
                }
                bucket_into(&mut image, (x, y), n, &mut height_data, diag);
            }
        }
    }

    // then detect insides
    for x in 0..image.width() {
        for y in 0..image.height() {
            // if pixel is not already clear, bucket here
            let px = image.get_pixel(x, y).0[0];
            if px != 0 {
                n += 1;
                let diag;
                if px == 255 {
                    diag = true;
                    insides.push(n);
                } else {
                    diag = false;
                    borders.push(n);
                }
                bucket_into(&mut image, (x, y), n, &mut height_data, diag);
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
) {
    let col = image.get_pixel(init_pos.0, init_pos.1).0[0];

    let mut positions = vec![init_pos];

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
}
