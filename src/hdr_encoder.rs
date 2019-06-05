use image;
use rayon::prelude::*;
pub struct HdrEncoder {
    pub width: u32,
    pub height: u32,
    pub frame: image::RgbImage,
}

impl HdrEncoder {
    pub fn new(width: u32, height: u32, y: &[u8], u: &[u8], v: &[u8]) -> Self {
        let mut img = image::RgbImage::new(width, height);
        for y_ in 0..height {
            for x in 0..width {
                let index = (y_ * width + x) as usize;
                let rgb = Self::yuv_to_rgb(y[index], u[index], v[index]);
                let pixel = image::Rgb(rgb);
                img.put_pixel(x, y_, pixel);
            }
        }
        Self {
            width,
            height,
            frame: img,
        }
    }

    #[inline]
    pub fn yuv_to_rgb(y: u8, u: u8, v: u8) -> [u8; 3] {
        let d = u as f32 - 128.0;
        let e = v as f32 - 128.0;
        let r = (y as f32 + (1.13983 * e)).min(255.0).max(0.0) as u8;
        let g = (y as f32 - (0.39465 * d + 0.58060 * e as f32))
            .min(255.0)
            .max(0.0) as u8;
        let b = (y as f32 + (2.03211 * d)).min(255.0).max(0.0) as u8;
        [r, g, b]
    }

    #[inline]
    pub fn rgb_to_luminance(r: u8, g: u8, b: u8) -> f32 {
        0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32)
    }

    #[inline]
    pub fn rgb_to_yuv(r: u8, g: u8, b: u8) -> [u8; 3] {
        let y = Self::rgb_to_luminance(r, g, b) as u8;
        let u = (r as f32 * -0.169 - g as f32 * 0.331 + b as f32 * 0.5 + 128.0) as u8;
        let v = (r as f32 * 0.5 - g as f32 * 0.419 - b as f32 * 0.081 + 128.0) as u8;
        [y, u, v]
    }

    pub fn encode(&self) -> Vec<u8> {
        use image::Pixel;
        let mut luminance_sum = 0.0;
        let img = &self.frame;
        let minima = 0.0000001;
        let mut luminances = Vec::with_capacity((self.width * self.height) as usize);
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = img.get_pixel(x, y).channels();
                let luminance = Self::rgb_to_luminance(pixel[0], pixel[1], pixel[2]) / 255.0;
                luminances.push(luminance);
                luminance_sum += (luminance + minima).log(10.0);
            }
        }
        let lum = (luminance_sum / (self.width * self.height) as f32).exp();
        let alpha = 0.6;
        let scalar = alpha / lum;
        
        luminances.par_iter_mut().for_each(|l| {
            *l *= scalar;
        });

        let luminance_max = 1.0 as f32;
        luminances.par_iter_mut().for_each(|l| {
            *l = (*l * (1.0 + *l / luminance_max.powi(2))) / (1.0 + *l);
        });





        luminances.into_iter().map(|l| (l * 255.0) as u8).collect()
    }
}
