extern crate hdr_rs;
use hdr_rs::hdr_encoder::*;
use image::GenericImageView;
use image::Pixel;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let default = "../1.hdr".to_owned();
    let name = args.get(1).unwrap_or(&default);
    let img = image::open(name).unwrap();
    let rgb = img.as_rgb8().unwrap();
    println!("{}x{}", rgb.width(), rgb.height());
    image::save_buffer(
        "../origin.jpg",
        &rgb.clone().into_vec(),
        img.width(),
        img.height(),
        image::ColorType::RGB(8),
    )
    .unwrap();
    let mut y: Vec<u8> = vec![];
    let mut u: Vec<u8> = vec![];
    let mut v: Vec<u8> = vec![];
    for y_ in 0..rgb.height() {
        for x in 0..rgb.width() {
            let pix = rgb.get_pixel(x, y_).channels();
            let yuv = HdrEncoder::rgb_to_yuv(pix[0], pix[1], pix[2]);
            y.push(yuv[0]);
            // if y_ % 2 == 0 && x % 2 == 0 {
            u.push(yuv[1]);
            v.push(yuv[2]);
            // }
        }
    }
    let start = std::time::SystemTime::now();
    let mut encoder = HdrEncoder::new(rgb.width(), rgb.height(), &y, &u, &v);
    y = encoder.encode();
    println!("time: {}", start.elapsed().unwrap().as_millis());
    let width = rgb.width();
    let height = rgb.height();
    let mut img = image::RgbImage::new(width, height);
    for y_ in 0..height {
        for x in 0..width {
            let index = (y_ * width + x) as usize;
            let rgb = HdrEncoder::yuv_to_rgb(y[index], u[index], v[index]);
            let pixel = image::Rgb(rgb);
            img.put_pixel(x, y_, pixel);
        }
    }
    image::save_buffer(
        "../out.jpg",
        &img.into_vec(),
        width,
        height,
        image::ColorType::RGB(8),
    )
    .unwrap();
}
