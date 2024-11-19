use super::{
    coeffstruct::OnePoleCoeffs, 
    filtertype::{ FilterError, HarmonicFilter, OnePoleFilter }, 
    onepole::DesignOnePoleFilter 
};
use crate::qbuffers::DelayBuffer;
use crate::qubx_common::FilteredSample;

fn _filt_sample_lowpass(x: &f32, coeffs: &(f32, f32), y1: f32) -> f32 {
    coeffs.0 * x + coeffs.1 * y1
}

fn _filt_sample_comb(mode: &str, x: &f32, g: &f32, _sample: f32) -> Result<f32, FilterError> {
    let sample = match mode {
        "fir" => x + g * _sample,
        "fir_freev" => -x + (1.0 + g) * _sample,
        "iir" => x - g * _sample,
        _ => return Err(FilterError::FilterModeNotAllowed)
    };
    Ok(sample)
}

fn _filt_sample_comb_lp(x: &f32, g: &f32, &lp_coeffs: &(f32, f32), x_lp: f32, y_lp: f32) -> (f32, f32) {
    let y_low_pass = _filt_sample_lowpass(&x_lp, &lp_coeffs, y_lp);
    let y = x - g * y_low_pass;
    (y, y_low_pass)
}

fn _filt_sample_allpass(mode: &str, x: &f32, g: &f32, x1: f32, y1: f32) -> Result<f32, FilterError> {
    let sample = match mode {
        "freev" => -x + (1.0 + g) * x1 - g * y1,
        "naive" => g * x + x1 - g * y1,
        _ => return Err(FilterError::FilterModeNotAllowed)
    };
    Ok(sample)
}

fn _filt_sample_allpass_lp(x: &f32, g: &f32, &lp_coeffs: &(f32, f32), x1: f32, x_lp: f32, y_lp: f32) -> (f32, f32) {
    let y_low_pass = _filt_sample_lowpass(&x_lp, &lp_coeffs, y_lp);
    let y = g * x + x1 - g * y_low_pass;
    (y, y_low_pass)
}

pub struct Harmonic {
    fs: f32,
    buffer_delay: usize,
    mode: HarmonicFilter,
    g: f32,
    x: DelayBuffer,
    y: DelayBuffer,
    ylp: DelayBuffer,
    low_pass_coeffs: OnePoleCoeffs
}

impl Harmonic {

    pub fn new(buffer_delay: usize, fs: f32) -> Self {
        let g = 0.0;
        let low_pass_coeffs = OnePoleCoeffs::new();

        Self {
            fs,
            buffer_delay,
            mode: HarmonicFilter::CombFIR,
            g,
            x: DelayBuffer::new(buffer_delay),
            y: DelayBuffer::new(buffer_delay),
            ylp: DelayBuffer::new(1),
            low_pass_coeffs
        }
    }

    pub fn design_filter(&mut self, mode: HarmonicFilter, t60: f32, fc: Option<f32>) {
        let d_time: f32 = (self.buffer_delay as f32) / self.fs;
        self.g = 10.0_f32.powf(-3.0 * d_time / t60);

        let (b0, a1) = match fc {
            Some(cutoff) => {
                let mut lowpass_coeffs = DesignOnePoleFilter::new(OnePoleFilter::LowPass, cutoff, self.fs);
                lowpass_coeffs.coeffs();
                (lowpass_coeffs.filt_coeffs.b0, lowpass_coeffs.filt_coeffs.a1)
            },
            None => { (0.0, 0.0) }             
        };
        self.low_pass_coeffs.set_coeffs((b0, 0.0, a1));
        self.mode = mode;
    }

    pub fn filt_sample(&mut self, sample: f32) -> f32 {
        let lp_coeffs = (self.low_pass_coeffs.b0, self.low_pass_coeffs.a1);

        let (yout, ylpass) = match self.mode {
            HarmonicFilter::CombFIR => {
                (_filt_sample_comb("fir", &sample, &self.g, self.x.read_buffer()).unwrap(), 0.0)
            },
            HarmonicFilter::CombFreeverbFIR => {
                (_filt_sample_comb("fir_freev", &sample, &self.g, self.x.read_buffer()).unwrap(), 0.0)
            },
            HarmonicFilter::CombIIR => {
                (_filt_sample_comb("iir", &sample, &self.g, self.y.read_buffer()).unwrap(), 0.0)
            },
            HarmonicFilter::Allpass => {
                (_filt_sample_allpass("naive", &sample, &self.g, self.x.read_buffer(), self.y.read_buffer()).unwrap(), 0.0)
            },
            HarmonicFilter::AllpassFreeverb => {
                (_filt_sample_allpass("freev", &sample, &self.g, self.x.read_buffer(), self.y.read_buffer()).unwrap(), 0.0)
            },
            HarmonicFilter::LPFBCombFilter => {
                let (y_out, y_out_lp) = _filt_sample_comb_lp(&sample, &self.g, &lp_coeffs, self.y.read_buffer(), self.ylp.read_buffer());
                (y_out, y_out_lp)
            },
            HarmonicFilter::LPFBAllpassFilter => {
                let (y_out, y_out_lp) = _filt_sample_allpass_lp(&sample, &self.g, &lp_coeffs, self.x.read_buffer(), self.y.read_buffer(), self.ylp.read_buffer());
                (y_out, y_out_lp)
            }
        };
        
        self.ylp.write_buffer(ylpass);
        self.x.write_buffer(sample);
        self.y.write_buffer(yout);
        yout

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
        self.ylp.reset_buffer();
    }

}

impl FilteredSample<f32> for Harmonic 
{ 
    fn filtered_sample(&mut self, sample: f32) -> f32 {
        self.filt_sample(sample)
    }
}