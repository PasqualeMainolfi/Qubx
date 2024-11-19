use super::coeffstruct::OnePoleCoeffs; 
use crate::qbuffers::DelayBuffer;
use crate::qubx_common::FilteredSample;

struct DesignDcFilter {
    filt_coeffs: OnePoleCoeffs,
    r: f32
}

impl DesignDcFilter {
    fn new(fc: f32, fs: f32) -> Self {
        let filt_coeffs = OnePoleCoeffs::new();
        let twopi = 2.0 * std::f32::consts::PI;
        let w = twopi * fc / fs;
        let r = 1.0 - w;
        
        Self {
            filt_coeffs,
            r
        }
    }

    fn coeffs(&mut self) {
        self.filt_coeffs.set_coeffs((1.0, 0.0, self.r))
    }

}

fn _filt_sample(x: &f32, coeffs: &(f32, f32), x1: f32, y1: f32) -> f32 {
    coeffs.0 * x - x1 + coeffs.1 * y1
}

pub struct Dc {
    fs: f32,
    coeffs: OnePoleCoeffs,
    _x: DelayBuffer,
    _y: DelayBuffer
}

impl Dc {

    pub fn new(fs: f32) -> Self {
        Self { fs, coeffs: OnePoleCoeffs::new(), _x: DelayBuffer::new(1), _y: DelayBuffer::new(1) }
    }

    pub fn design_filter(&mut self, fc: f32) {
        let mut design_filter = DesignDcFilter::new(fc, self.fs);
        design_filter.coeffs();
        self.coeffs = design_filter.filt_coeffs
    }

    pub fn filt_sample(&mut self, sample: f32) -> f32 {
        let y = self.coeffs.b0 * sample - self._x.read_buffer() + self.coeffs.a1 * self._y.read_buffer();
        self._x.write_buffer(sample);
        self._y.write_buffer(y);
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
        self._x.read_buffer();
        self._y.read_buffer();
    }

}

impl FilteredSample<f32> for Dc 
{ 
    fn filtered_sample(&mut self, sample: f32) -> f32 {
        self.filt_sample(sample)
    }
}