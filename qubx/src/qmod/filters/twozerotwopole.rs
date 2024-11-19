use super::{ 
    coeffstruct::TwoPoleCoeffs, 
    filtertype::TwoZeroTwoPoleFilter 
};
use crate::qbuffers::DelayBuffer;
use crate::qubx_common::FilteredSample;

struct DesignTwoPoleTwoZeroFilter {
    mode: TwoZeroTwoPoleFilter,
    filt_coeffs: TwoPoleCoeffs,
    theta_cosine: f32,
    r: f32
}

impl DesignTwoPoleTwoZeroFilter {
    fn new(mode: TwoZeroTwoPoleFilter, fc: f32, fs: f32, bw: f32) -> Self {

        let pi = std::f32::consts::PI;
        let filt_coeffs = TwoPoleCoeffs::new();
        let theta_cosine: f32 = (2.0 * pi * fc / fs).cos();
        let r = (-pi * bw / fs).exp();

        Self {
            mode,
            filt_coeffs,
            theta_cosine,
            r
        }
    }

    fn coeffs(&mut self) {
        match self.mode {
            TwoZeroTwoPoleFilter::Notch => {
                let b0 = 1.0;
                let b1 = -2.0 * self.r * self.theta_cosine;
                let b2 = self.r * self.r;
                self.filt_coeffs.set_coeffs((b0, b1, b2, 0.0, 0.0, 0.0))
            },
            TwoZeroTwoPoleFilter::Bp => {
                let b0 = 1.0;
                let a1 = -2.0 * self.r * self.theta_cosine;
                let a2 = self.r * self.r;
                self.filt_coeffs.set_coeffs((b0, 0.0, 0.0, 0.0, a1, a2))
            }
        }
    }
}

pub struct TwoZeroTwoPole {
    fs: f32,
    coeffs: TwoPoleCoeffs,
    x1: DelayBuffer,
    x2: DelayBuffer,
    y1: DelayBuffer,
    y2: DelayBuffer
}

impl TwoZeroTwoPole {
   
    pub fn new(fs: f32) -> Self { 
        Self { 
            fs,
            coeffs: TwoPoleCoeffs::new(), 
            x1: DelayBuffer::new(1), 
            x2: DelayBuffer::new(2), 
            y1: DelayBuffer::new(1), 
            y2: DelayBuffer::new(2) 
        }
    }

    pub fn design_filter(&mut self, mode: TwoZeroTwoPoleFilter, fc: f32, bw: f32) {
        let mut design_filter = DesignTwoPoleTwoZeroFilter::new(mode, fc, self.fs, bw);
        design_filter.coeffs();
        self.coeffs = design_filter.filt_coeffs
    }

    pub fn filt_sample(&mut self, sample: f32) -> f32 {

        let y = self.coeffs.b0 * sample + 
            self.coeffs.b1 * self.x1.read_buffer() + 
            self.coeffs.b2 * self.x2.read_buffer() - 
            self.coeffs.a1 * self.y1.read_buffer() - 
            self.coeffs.a2 * self.y2.read_buffer();

        self.x1.write_buffer(sample);
        self.x2.write_buffer(sample);
        self.y1.write_buffer(y);
        self.y2.write_buffer(y);

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
        self.x1.reset_buffer();
        self.x2.reset_buffer();
        self.y1.reset_buffer();
        self.y2.reset_buffer();
    }

}

impl FilteredSample<f32> for TwoZeroTwoPole
{
    fn filtered_sample(&mut self, sample: f32) -> f32 {
        self.filt_sample(sample)
    }
}