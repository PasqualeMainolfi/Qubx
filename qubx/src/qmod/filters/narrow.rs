use super::{
    coeffstruct::TwoPoleCoeffs, filtertype::NarrowFilter
};
use crate::qbuffers::DelayBuffer;
use crate::qubx_common::FilteredSample;

struct DesignNarrowFilter {
    mode: NarrowFilter,
    filt_coeffs: TwoPoleCoeffs,
    theta_cosine: f32,
    k: f32,
    r: f32
}

impl DesignNarrowFilter {
    fn new(mode: NarrowFilter, fc: f32, fs: f32, bw: f32) -> Self {
        const TWOPI: f32 = 2.0 * std::f32::consts::PI;
        let filt_coeffs = TwoPoleCoeffs::new();
        let w = TWOPI * fc / fs;
        let theta_cosine = w.cos();
        let r = 1.0 - 3.0 * bw / fs;
        let k = (1.0 - 2.0 * r * theta_cosine + r.powf(2.0)) / (2.0 - 2.0 * theta_cosine);

        Self {
            mode,
            filt_coeffs,
            theta_cosine,
            k,
            r
        }
    }

    fn coeffs(&mut self) {
        match self.mode {
            NarrowFilter::Bp => {
                let b0: f32 = 1.0 - self.k;
                let b1: f32 = 2.0 * (self.k - self.r) * self.theta_cosine;
                let b2: f32 = self.r.powf(2.0) - self.k;
                
                let a1: f32 = 2.0 * self.r * self.theta_cosine;
                let a2: f32 = -self.r.powf(2.0);

                self.filt_coeffs.set_coeffs((b0, b1, b2, 1.0, a1, a2))
            },
            NarrowFilter::Notch => {
                let b0: f32 = self.k;
                let b1: f32 = -2.0 * self.k * self.theta_cosine;
                let b2: f32 = self.k;
                
                let a1: f32 = 2.0 * self.r * self.theta_cosine;
                let a2: f32 = -self.r.powf(2.0);

                self.filt_coeffs.set_coeffs((b0, b1, b2, 1.0, a1, a2))
            }
        }
    }

}

pub struct Narrow {
    fs: f32,
    coeffs: TwoPoleCoeffs,
    x1: DelayBuffer,
    x2: DelayBuffer,
    y1: DelayBuffer,
    y2: DelayBuffer,
    order: usize,
}

impl Narrow {
    
    pub fn new(fs: f32, order: Option<usize>) -> Self {
        let filt_order = order.unwrap_or(1);
        Self { 
            fs,
            coeffs: TwoPoleCoeffs::new(), 
            x1: DelayBuffer::new(filt_order), 
            x2: DelayBuffer::new(filt_order), 
            y1: DelayBuffer::new(filt_order), 
            y2: DelayBuffer::new(filt_order),
            order: filt_order,
        }
    }
    
    pub fn design_filter(&mut self, mode: NarrowFilter, fc: f32, bw: f32) {
        let mut design_filter: DesignNarrowFilter = DesignNarrowFilter::new(mode, fc, self.fs, bw);
        design_filter.coeffs();
        self.coeffs = design_filter.filt_coeffs
    }

    pub fn filt_sample(&mut self, sample: f32) -> f32 {

        let mut x = sample;
        let mut y: f32 = 0.0;
        for _ in 0..self.order {
            y = self.coeffs.b0 * x + 
            self.coeffs.b1 * self.x1.read_buffer() + 
            self.coeffs.b2 * self.x2.read_buffer() + 
            self.coeffs.a1 * self.y1.read_buffer() + 
            self.coeffs.a2 * self.y2.read_buffer();

            self.x1.write_buffer(x);
            self.x2.write_buffer(x);
            self.y1.write_buffer(y);
            self.y2.write_buffer(y);

            x = y;
        }
        y
    }

    pub fn filt_frame(&mut self, frame: Vec<f32>) -> Vec<f32> {
        
        let y: Vec<f32> = frame
            .iter()
            .map(|&x| self.filt_sample(x))
            .collect();
        y
    }

    pub fn clear_delayed_samples_cache(&mut self) {
        self.x1.reset_buffer();
        self.x2.reset_buffer();
        self.y1.reset_buffer();
        self.y2.reset_buffer();
    }

}

impl FilteredSample<f32> for Narrow 
{ 
    fn filtered_sample(&mut self, sample: f32) -> f32 {
        self.filt_sample(sample)
    }
}