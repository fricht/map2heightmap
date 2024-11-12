use clap::Parser;
use image::open;
use map2heightmap::{clean_mask, extract_color, separate_regions, set_heights};

#[derive(Parser, Debug)]
struct Cli {
    imgpath: std::path::PathBuf,
    #[arg(short, long)]
    debug: bool,
}

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

    let (regions, mut insides, mut borders) = separate_regions(&relief_mask_image);

    // save regions (if debug)
    if args.debug {
        println!("Saving regions at regions.png");
        regions.save("regions.png").expect("Failed to save image.");
    }

    set_heights(&mut insides, &mut borders);

    // save regions (if debug)
    if args.debug {
        println!("Regions : {:#?}", insides);
        println!("Heights : {:#?}", borders);
    }
}
