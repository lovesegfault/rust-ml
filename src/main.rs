use image::{GenericImageView, Rgba};
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use itertools::Itertools;
use structopt::StructOpt;
use tensorflow::{Graph, ImportGraphDefOptions, Session, SessionOptions, SessionRunArgs, Tensor};

use std::error::Error;
use std::path::PathBuf;

const LINE_COLOR: Rgba<u8> = Rgba {
    data: [0, 255, 0, 0],
};

#[derive(StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    #[structopt(parse(from_os_str))]
    output: PathBuf,
}

#[derive(Copy, Clone, Debug)]
struct BBox {
    pub y: i32,
    pub x: i32,
    pub width: u32,
    pub height: u32,
    pub prob: f32,
}

macro_rules! flatten_hack {
    ($x: expr) => (std::iter::once($x));
    ($x: expr, $($y: expr),+) => (flatten_hack!($x).chain(flatten_hack!($($y),+)));
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse otps
    let opt = Opt::from_args();
    // Open image
    let input_image = image::open(&opt.input)?;
    // Load model
    let model = include_bytes!("../models/mtcnn.pb");
    let mut graph = Graph::new();
    graph.import_graph_def(&*model, &ImportGraphDefOptions::new())?;
    // Flatten image
    let flattened: Vec<f32> = input_image
        .pixels()
        .flat_map(|(_, _, rgb)| flatten_hack!(rgb[2], rgb[1], rgb[0]))
        .map(|i| f32::from(i))
        .collect();
    // Load image into a tensor
    let input = Tensor::new(&[
        u64::from(input_image.height()),
        u64::from(input_image.width()),
        3,
    ])
    .with_values(&flattened)?;
    let session = Session::new(&SessionOptions::new(), &graph)?;
    // mtcnn model inputs
    let min_size = Tensor::new(&[]).with_values(&[40_f32])?;
    let thresholds = Tensor::new(&[3]).with_values(&[0.6_f32, 0.7_f32, 0.7_f32])?;
    let factor = Tensor::new(&[]).with_values(&[0.709_f32])?;
    // Load model parameters
    let mut args = SessionRunArgs::new();
    args.add_feed(&graph.operation_by_name_required("min_size")?, 0, &min_size);
    args.add_feed(
        &graph.operation_by_name_required("thresholds")?,
        0,
        &thresholds,
    );
    args.add_feed(&graph.operation_by_name_required("factor")?, 0, &factor);
    // Load input image
    args.add_feed(&graph.operation_by_name_required("input")?, 0, &input);
    // Set outputs
    let bbox = args.request_fetch(&graph.operation_by_name_required("box")?, 0);
    let prob = args.request_fetch(&graph.operation_by_name_required("prob")?, 0);
    // Run session
    session.run(&mut args)?;
    // Save results
    let bbox_res: Tensor<f32> = args.fetch(bbox)?;
    let prob_res: Tensor<f32> = args.fetch(prob)?;
    // Copy input image
    let mut output_image = input_image.clone();
    // Draw bboxes
    bbox_res
        .iter()
        .tuples::<(_, _, _, _)>() // Chunk the iterator into 4-tuples
        .zip(prob_res.iter()) // Zip it with the probabilities
        .map(|((y1, x1, y2, x2), prob)| {
            // Map values into struct
            BBox {
                y: *y1 as i32,
                x: *x1 as i32,
                width: (*x2 - *x1) as u32,
                height: (*y2 - *y1) as u32,
                prob: *prob,
            }
        })
        .map(|bbox| Rect::at(bbox.x, bbox.y).of_size(bbox.width, bbox.height))
        .for_each(|rect| draw_hollow_rect_mut(&mut output_image, rect, LINE_COLOR));
    // Save output
    output_image.save(&opt.output)?;
    Ok(())
}
