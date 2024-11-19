use super::filtertype::{ FilterError, ZavalishinFilter, SvfSamples };
use crate::qubx_common::FilteredSample;

fn filt_sample(sample: &f32, g: f32, _z: f32) -> (f32, f32, f32) {
    let v = (sample - _z) * g;
    let lp = v + _z;
    let hp = sample - lp;
    let z = lp + v;
    (lp, hp, z)
}

fn filt_sample_svf(sample: &f32, coeffs: (f32, f32, f32), _z: f32, _s: f32) -> (f32, f32, f32, f32) {
    let hp = (sample - 2.0 * coeffs.2 * _z - coeffs.0 * _z - _s) / coeffs.1;
    let bp = coeffs.0 * hp + _z;
    let lp = coeffs.0 * bp + _s;
    let br = sample - 2.0 * coeffs.2 * bp;
    (lp, hp, bp, br)
}

pub struct Zavalishin {
    fs: f32,
    g: f32,
    g1: f32,
    r: f32,
    z_sample: f32,
    s_sample: f32,
    filt_type: Option<ZavalishinFilter>
}

impl Zavalishin {
   
    pub fn new(fs: f32) -> Self {
        Self { 

            fs, 
            g: 0.0, 
            g1: 0.0, 
            r: 0.0, 
            z_sample: 0.0, 
            s_sample: 0.0, 
            filt_type: None

        }
    }
    
    pub fn design_filter(&mut self, mode: ZavalishinFilter, fc: f32, fc_spread: Option<f32>) -> Result<(), FilterError>{
        
        let twopi = 2.0 * std::f32::consts::PI;
        let wc = twopi * fc;
        let ts = 1.0 / self.fs;
        let mut wa = (2.0 / ts) * (wc * ts / 2.0).tan();
        match mode {
            ZavalishinFilter::OnePoleZeroDelay => { 
                self.g = wa * ts / 2.0
            },
            ZavalishinFilter::NaiveOnePole => { 
                self.g = wa * ts
            },
            ZavalishinFilter::TrapIntOnePole => { 
                self.g = wa * ts / 2.0
            },
            ZavalishinFilter::StateVariable => { 
                match fc_spread {
                    Some(spread) => {
                        let w = twopi * (fc + spread);
                        let w_sqrt = (wc * w).sqrt();
                        self.r = ((wc + w) / 2.0) / w_sqrt;
                        wa = (2.0 / ts) * (w_sqrt * ts / 2.0).tan();
                        self.g = wa * ts / 2.0;
                        self.g1 = 1.0 + (2.0 * self.r * self.g) + self.g.powf(2.0)
                    },
                    None => return Err(FilterError::FcSpreadNotSpecified)
                }
            }
        }
        self.filt_type = Some(mode);
        Ok(())
    }
    
    pub fn filt_sample(&mut self, sample: f32) -> SvfSamples {
        let (lp, hp, ap, bp, br, z, s) = match &self.filt_type {
            Some(t) => { match t {
                ZavalishinFilter::OnePoleZeroDelay => {
                    let (_lp, _hp, _z) = filt_sample(&sample, self.g, self.z_sample);
                    let _ap = _lp - _hp;
                    (_lp, _hp, _ap, 0.0, 0.0, _z, 0.0)
                },
                ZavalishinFilter::NaiveOnePole => {
                    let (_lp, _hp, _z) = filt_sample(&sample, self.g, self.z_sample);
                    (_lp, _hp / 2.0, 0.0, 0.0, 0.0, _lp, 0.0)
                },
                ZavalishinFilter::TrapIntOnePole => {
                    let (_lp, _hp, _z) = filt_sample(&sample, self.g, self.z_sample);
                    (_lp, _hp, 0.0, 0.0, 0.0, _z, 0.0)
                },
                ZavalishinFilter::StateVariable => {
                    let (_lp, _hp, _bp, _br) = filt_sample_svf(&sample, (self.g, self.g1, self.r), self.z_sample, self.s_sample);
                    let _z = self.g * _hp + _bp;
                    let _s = self.g * _bp + _lp;
                    (_lp, _hp, 0.0, _bp, _br, _z, _s)
                    
                }
            }
        },
            None => (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
        };

        self.z_sample = z;
        self.s_sample = s;
        let mut svf_samples = SvfSamples::new();
        svf_samples.set_samples((lp, hp, ap, bp, br));
        svf_samples
        
    }

    pub fn clear_delayed_samples_cache(&mut self) {
        self.z_sample = 0.0;
        self.s_sample = 0.0;
    }

}

impl FilteredSample<SvfSamples> for Zavalishin
{
    fn filtered_sample(&mut self, sample: f32) -> SvfSamples {
        self.filt_sample(sample)
    }
}