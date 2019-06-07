#![allow(non_snake_case)]

use rayon::prelude::*;
pub struct HdrEncoder {
    pub width: u32,
    pub height: u32,
    pub frame: Vec<f32>,
}

impl HdrEncoder {
    pub fn new(width: u32, height: u32, y: &[u8], _u: &[u8], _v: &[u8]) -> Self {
        let frame = y.par_iter().map(|e| *e as f32 / 255.0).collect();
        Self {
            width,
            height,
            frame,
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

    pub fn encode(self, prev_lum: f32) -> (Vec<u8>, f32) {
        let minima = 0.0000001;
        let luminances = self.frame;
        let luminance_sum = luminances
            .par_iter()
            .map(|l| (*l + minima).log(10.0))
            .sum::<f32>();

        let lum_max = luminances.iter().max_by(|a, b| {a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)}).unwrap();
        // let lum_min = luminances.iter().min_by(|a, b| {a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)}).unwrap();
        // let lum_avg = luminances.iter().sum::<f32>() / luminances.len() as f32;
        let mut lum = (luminance_sum / (self.width * self.height) as f32).exp();

        if prev_lum >= 0.0 {
            lum = 0.6 * lum + 0.4 * prev_lum;
        }

        // let alpha_A = lum_max - lum_avg;
        // let alpha_B = lum_avg - lum_min;
        // let alpha = 0.18 * (2.0 as f32).powf(2.0 * (alpha_B - alpha_A) / (alpha_A + alpha_B));
        let alpha = 0.72;
        let scalar = alpha / lum;
        println!("{} {} {}", alpha, lum, scalar);

        let mut lum_d: Vec<f32> = luminances.par_iter().map(|l| {
            *l * scalar
        }).collect();

        // let luma_white = 1e10 as f32;
        let bias = 0.8 as f32;
        let weight = bias.ln() / (0.5 as f32).ln();
        lum_d.par_iter_mut().enumerate().for_each(|(index, l)| {
            // *l = (*l * (1.0 + *l / luma_white.powi(2))) / (1.0 + *l);
            *l = (*l / (1.0 + *l)) * (1.0 + 9.0 * (luminances[index] / lum_max).powf(weight)).log(10.0);
        });

        (
            lum_d
                .into_par_iter()
                .map(|l| (l * 255.0)/*.min(255.0).max(0.0)*/ as u8)
                .collect(),
            lum,
        )
    }
}
