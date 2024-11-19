use super::{ 
    coeffstruct::TwoPoleCoeffs, filtertype::{ BiquadFilter, FilterError } 
};
use crate::{qbuffers::DelayBuffer, qubx_common::FilteredSample};

struct DesignBiquadFilter 
{
    mode: BiquadFilter,
    filt_coeffs: TwoPoleCoeffs,
    theta_sine: f32,
    theta_cosine: f32,
    alpha: f32,
    a: Option<f32>,
    beta: Option<f32>,
}

impl DesignBiquadFilter 
{
    fn new(mode: BiquadFilter, fc: f32, fs: f32, q: f32, dbgain: Option<f32>) -> Self {
        const TWOPI: f32 = 2.0 * std::f32::consts::PI;
        let filt_coeffs = TwoPoleCoeffs::new();
        let w = TWOPI * fc / fs;
        let theta_sine = w.sin();
        let theta_cosine = w.cos();
        
        let alpha = theta_sine / (2.0 * q);

        // manual implementation of Option.map
        // self.a = match dbgain {            
        //     Some(dbvalue) => Some((10_f32).powf(dbvalue / 40_f32)),
        //     None => None
        // };

        let a = dbgain.map(|db_value| 10_f32.powf(db_value / 40_f32));
        let beta = a.map(|a_value| a_value.sqrt() / q);

        Self {
            mode,
            filt_coeffs,
            theta_sine,
            theta_cosine,
            alpha,
            a,
            beta
        }
    }

    fn coeffs(&mut self) {
        match self.mode {
            BiquadFilter::Lp => {
                let c = 1.0 - self.theta_cosine;
                let b0: f32 = c / 2.0;
                let b1: f32 = c;
                let b2: f32 = b0;
                
                let a0: f32 = 1.0 + self.alpha;
                let a1: f32 = -2.0 * self.theta_cosine;
                let a2: f32 = 1.0 - self.alpha;

                self.filt_coeffs.set_coeffs((b0, b1, b2, a0, a1, a2))
            },
            BiquadFilter::Hp => {
                let c = 1.0 + self.theta_cosine;
                let b0: f32 = c / 2.0;
                let b1: f32 = -c;
                let b2: f32 = b0;
                
                let a0: f32 = 1.0 + self.alpha;
                let a1: f32 = -2.0 * self.theta_cosine;
                let a2: f32 = 1.0 - self.alpha;

                self.filt_coeffs.set_coeffs((b0, b1, b2, a0, a1, a2))
            },
            BiquadFilter::Bp0dB => {
                let b0: f32 = self.alpha;
                let b1: f32 = 0.0;
                let b2: f32 = -b0;
                
                let a0: f32 = 1.0 + self.alpha;
                let a1: f32 = -2.0 * self.theta_cosine;
                let a2: f32 = 1.0 - self.alpha;

                self.filt_coeffs.set_coeffs((b0, b1, b2, a0, a1, a2))
            },
            BiquadFilter::Bpsg => {
                let b0: f32 = self.theta_sine / 2.0;
                let b1: f32 = 0.0;
                let b2: f32 = -b0;
                
                let a0: f32 = 1.0 + self.alpha;
                let a1: f32 = -2.0 * self.theta_cosine;
                let a2: f32 = 1.0 - self.alpha;

                self.filt_coeffs.set_coeffs((b0, b1, b2, a0, a1, a2))
            },
            BiquadFilter::Notch => {
                let b0: f32 = 1.0;
                let b1: f32 = -2.0 * self.theta_cosine;
                let b2: f32 = b0;
                
                let a0: f32 = 1.0 + self.alpha;
                let a1: f32 = -2.0 * self.theta_cosine;
                let a2: f32 = 1.0 - self.alpha;

                self.filt_coeffs.set_coeffs((b0, b1, b2, a0, a1, a2))
            },
            BiquadFilter::Ap => {
                let b0: f32 = 1.0 - self.alpha;
                let b1: f32 = -2.0 * self.theta_cosine;
                let b2: f32 = 1.0 + self.alpha;
                
                let a0: f32 = 1.0 + self.alpha;
                let a1: f32 = -2.0 * self.theta_cosine;
                let a2: f32 = 1.0 - self.alpha;

                self.filt_coeffs.set_coeffs((b0, b1, b2, a0, a1, a2))
            },
            BiquadFilter::Peq => {
                match self.a {
                    Some(a_value) => { 
                        let b0: f32 = 1.0 + self.alpha * a_value;
                        let b1: f32 = -2.0 * self.theta_cosine;
                        let b2: f32 = 1.0 - self.alpha * a_value;
                        
                        let a0: f32 = 1.0 + self.alpha / a_value;
                        let a1: f32 = -2.0 * self.theta_cosine;
                        let a2: f32 = 1.0 - self.alpha / a_value;

                        self.filt_coeffs.set_coeffs((b0, b1, b2, a0, a1, a2))

                    },
                    None => {
                        self.filt_coeffs.set_coeffs((0.0, 0.0, 0.0, 0.0, 0.0, 0.0))
                    }
                }
            },
            BiquadFilter::LpShelf => {
                match (self.a, self.beta) {
                    (Some(a_value), Some(beta_value)) => {
                        let b0: f32 = a_value * ((a_value + 1.0) - (a_value - 1.0) * self.theta_cosine + beta_value * self.theta_sine);
                        let b1: f32 = 2.0 * a_value * ((a_value - 1.0) - (a_value + 1.0) * self.theta_cosine);
                        let b2: f32 = a_value * ((a_value + 1.0) - (a_value - 1.0) * self.theta_cosine - beta_value * self.theta_sine);
                        
                        let a0: f32 = (a_value + 1.0) + (a_value - 1.0) * self.theta_cosine + beta_value * self.theta_sine;
                        let a1: f32 = -2.0 * ((a_value - 1.0) + (a_value + 1.0) * self.theta_cosine);
                        let a2: f32 = (a_value + 1.0) + (a_value - 1.0) * self.theta_cosine - beta_value * self.theta_sine;

                        self.filt_coeffs.set_coeffs((b0, b1, b2, a0, a1, a2))

                    },
                    (None, Some(_)) | (Some(_), None) | (None, None) => {
                        self.filt_coeffs.set_coeffs((0.0, 0.0, 0.0, 0.0, 0.0, 0.0))
                    }
                }
            },
            BiquadFilter::HpShelf => {
                match (self.a, self.beta) {
                    (Some(a_value), Some(beta_value)) => {
                        let b0: f32 = a_value * ((a_value + 1.0) + (a_value - 1.0) * self.theta_cosine + beta_value * self.theta_sine);
                        let b1: f32 = -2.0 * a_value * ((a_value - 1.0) + (a_value + 1.0) * self.theta_cosine);
                        let b2: f32 = a_value * ((a_value + 1.0) + (a_value - 1.0) * self.theta_cosine - beta_value * self.theta_sine);
                        
                        let a0: f32 = (a_value + 1.0) - (a_value - 1.0) * self.theta_cosine + beta_value * self.theta_sine;
                        let a1: f32 = 2.0 * ((a_value - 1.0) - (a_value + 1.0) * self.theta_cosine);
                        let a2: f32 = (a_value + 1.0) - (a_value - 1.0) * self.theta_cosine - beta_value * self.theta_sine;

                        self.filt_coeffs.set_coeffs((b0, b1, b2, a0, a1, a2))

                    },
                    (None, Some(_)) | (Some(_), None) | (None, None) => {
                        self.filt_coeffs.set_coeffs((0.0, 0.0, 0.0, 0.0, 0.0, 0.0))
                    }
                }
            }
        }
    }

}


pub struct Biquad 
{
    fs: f32,
    coeffs: TwoPoleCoeffs,
    x1: DelayBuffer,
    x2: DelayBuffer,
    y1: DelayBuffer,
    y2: DelayBuffer
}

impl Biquad 
{
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
    
    pub fn design_filter(&mut self, mode: BiquadFilter, fc: f32, q: f32, dbgain: Option<f32>) -> Result<(), FilterError> {
        let mut design_filter: DesignBiquadFilter = DesignBiquadFilter::new(mode, fc, self.fs, q, dbgain);
        design_filter.coeffs();
        self.coeffs = design_filter.filt_coeffs;
        Ok(())
    }

    pub fn filt_sample(&mut self, sample: f32) -> f32 {
        let y: f32 = (
            self.coeffs.b0 * sample + 
            self.coeffs.b1 * self.x1.read_buffer() + 
            self.coeffs.b2 * self.x2.read_buffer() - 
            self.coeffs.a1 * self.y1.read_buffer() -
            self.coeffs.a2 * self.y2.read_buffer()
        ) / self.coeffs.a0;

        self.x1.write_buffer(sample);
        self.x2.write_buffer(sample);
        self.y1.write_buffer(y);
        self.y2.write_buffer(y);

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

impl FilteredSample<f32> for Biquad 
{ 
    fn filtered_sample(&mut self, sample: f32) -> f32 {
        self.filt_sample(sample)
    }
}

