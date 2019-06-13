#![allow(non_snake_case)]

use rayon::prelude::*;
pub struct HdrEncoder {
    pub width: u32,
    pub height: u32,
    pub frame: Vec<f32>,
}

impl HdrEncoder {
    #[inline]
    pub fn inRange(index: usize, left: usize, right: usize) -> bool {
        index >= left && index < right
    }

    #[inline]
    pub fn yuv_to_hsv(y: u8, u: u8, v: u8) -> f32 {
        let d = u as f32 - 128.0;
        let e = v as f32 - 128.0;
        let r = (y as f32 + (1.13983 * e)).min(255.0).max(0.0);
        let g = (y as f32 - (0.39465 * d + 0.58060 * e as f32))
            .min(255.0)
            .max(0.0);
        let b = (y as f32 + (2.03211 * d)).min(255.0).max(0.0);
        *[r / 255.0, g / 255.0, b / 255.0].iter().max_by(|a, b| {a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)}).unwrap()
    }

    pub fn new(width: u32, height: u32, y: &[u8], _u: &[u8], _v: &[u8]) -> Self {
        let frame = y.par_iter().map(|e| *e as f32 / 255.0).collect();

        Self {
            width,
            height,
            frame,
        }
    }

    pub fn encode_v2(self, _prev_lum: f32) -> (Vec<u8>, f32) {
        // Set params
        let Lmax = 2.5;
        let mut inverse_lum = self.frame.clone();
        let mut filter_lum = self.frame.clone();
        let mut result_lum = self.frame.clone();

        // Inverse tone-mapping
        inverse_lum.par_iter_mut().for_each(|l| {
            *l = 0.5 * Lmax * Lmax * ((*l - 1.0) + ((1.0 - *l).powi(2) + (*l * 4.0) / (Lmax * Lmax)).sqrt())
        });

        // Threshold filtering
        let threshold_rate = 0.95;
        let max_light = self.frame.iter().max_by(|a, b| {a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)}).unwrap();
        let filter_rate = max_light * threshold_rate;

        filter_lum.par_iter_mut().for_each(|l| {
            if *l < filter_rate {
                *l = 0.0;
            }
        });

        // Signle point rusting
        let temp_result: Vec<f32> = filter_lum.par_iter().enumerate().map(|(index, l)| {
            let mut res_ele = *l;
            if *l >= 0.0 && *l <= 1.0 {
                let core_indexes = [index - self.width as usize, index as usize, index + self.width as usize];
                let _isZero = false;
                for id in core_indexes.iter() {
                    let (left, right) = (id-1, id+1);
                    // Out of range judging
                    let length = self.frame.len();
                    if Self::inRange(left, 0, length) && Self::inRange(right, 0, length) && Self::inRange(*id, 0, length) {
                        if filter_lum[left] == 0.0 || filter_lum[right] == 0.0 || filter_lum[*id] == 0.0 {
                            res_ele = 0.0;
                            break;
                        }
                    }
                }
            }
            else {
                if res_ele < 0.0 {
                    res_ele = 0.0;
                }
                if res_ele > 1.0 {
                    res_ele = 1.0;
                }
            }
            res_ele
        }).collect();

        filter_lum = temp_result;

        // Perform gaussian filtering
        let _gaussian_kernel = vec![1,4,7,4,1,4,16,26,16,4,7,26,41,26,7,4,16,26,16,4,1,4,7,4,1];
        let kernel_weight: Vec<i32> = vec![-2, -1, 0, 1, 2];

        let gauss_result: Vec<f32> = filter_lum.par_iter().enumerate().map(|(index, l)| {
            let row = index as u32 / self.width;
            let col = index as u32 - row * self.width;
            let mut sum = 0.0;
            let mut gauss_res = *l;

            if row >= 2 && col >= 2 && row < self.height- 2 && col < self.width - 2 {
                for i in kernel_weight.iter() {
                    for j in kernel_weight.iter() {
                        sum += filter_lum[(index as i32 + *i * self.width as i32 + *j as i32) as usize];
                    }
                }
                sum /= 273.0;
                if sum > 255.0 {
                    sum = 255.0;
                }
                gauss_res = sum;
            }
            gauss_res
        }).collect();

        filter_lum = gauss_result;

        // Merge results
        let min_light = self.frame.iter().min_by(|a, b| {a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)}).unwrap();
        let (sigma, belta, gamma, delta) = (0.5, 2.0, 0.7, 0.02);

        result_lum.par_iter_mut().enumerate().for_each(|(index, l)| {
            if inverse_lum[index] < belta * min_light {
                *l = sigma * inverse_lum[index];
            } 
            else {
                *l = gamma * inverse_lum[index] + delta * filter_lum[index];
            }
            // Overlap filtering
            if *l > 1.0 {
                *l = 1.0;
            }   
            if *l < 0.0 {
                *l = 0.0;
            }
        });

        (
            result_lum
                .into_par_iter()
                .map(|l| (l * 255.0) as u8)
                .collect(),
            0.0,
        )
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
