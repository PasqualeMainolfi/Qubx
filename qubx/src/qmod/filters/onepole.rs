use super::{
    coeffstruct::OnePoleCoeffs, 
    filtertype::OnePoleFilter
};
use crate::qbuffers::DelayBuffer;
use crate::qubx_common::FilteredSample;

pub struct DesignOnePoleFilter {
    mode: OnePoleFilter,
    pub filt_coeffs: OnePoleCoeffs,
    alpha: f32
}

impl DesignOnePoleFilter {
    pub fn new(mode: OnePoleFilter, fc: f32, fs: f32) -> Self {
        let filt_coeffs = OnePoleCoeffs::new();
        let twopi = 2.0 * std::f32::consts::PI;
        let w = twopi * fc / fs;
        let alpha = (-2.0 * w).exp();

        Self {
            mode,
            filt_coeffs,
            alpha
        }
    }

    pub fn coeffs(&mut self) {
        match self.mode {
            OnePoleFilter::LowPass => {
                let b0 = 1.0 - self.alpha;
                let a1 = self.alpha;
                self.filt_coeffs.set_coeffs((b0, 0.0, a1));
            },
            OnePoleFilter::HighPass => {
                let b0 = (1.0 + self.alpha) / 2.0;
                let b1 = -b0;
                let a1 = self.alpha;
                self.filt_coeffs.set_coeffs((b0, b1, a1));
            }
        }
    } 

}

pub struct OnePole {
    fs: f32,
    coeffs: OnePoleCoeffs,
    x: DelayBuffer,
    y: DelayBuffer,
    order: usize,
}

impl OnePole {
   
    pub fn new(fs: f32, order: Option<usize>) -> Self {
        let filt_order = order.unwrap_or(1); 
        Self { 
            fs,
            coeffs: OnePoleCoeffs::new(), 
            x: DelayBuffer::new(filt_order), 
            y: DelayBuffer::new(filt_order),
            order: filt_order,
        }
    }

    pub fn design_filter(&mut self, mode: OnePoleFilter, fc: f32) {
        let mut design_filter = DesignOnePoleFilter::new(mode, fc, self.fs);
        design_filter.coeffs();
        self.coeffs = design_filter.filt_coeffs
    }

    pub fn filt_sample(&mut self, sample: f32) -> f32 {
        let mut x = sample;
        let mut y: f32 = 0.0;
        for _ in 0..self.order {
            y = self.coeffs.b0 * x + 
                self.coeffs.b1 * self.x.read_buffer() + 
                self.coeffs.a1 * self.y.read_buffer();

            self.x.write_buffer(x);
            self.y.write_buffer(y);
            x = y;
        }
        y
    }

    pub fn filt_frame(&mut self, frame: Vec<f32>) -> Vec<f32> {
        let y = frame
            .iter()
            .map(|&x| self.filt_sample(x))
            .collect();
        y
    }

    pub fn clear_delayed_samples_cache(&mut self) {
        self.x.reset_buffer();
        self.y.reset_buffer();
    }

}

impl FilteredSample<f32> for OnePole 
{ 
    fn filtered_sample(&mut self, sample: f32) -> f32 {
        self.filt_sample(sample)
    }
}