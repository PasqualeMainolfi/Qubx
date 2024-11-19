use super::{
    coeffstruct::{ ButterCoeffs, OnePoleCoeffs }, 
    filtertype::{ ButterFilter, FilterError }
};
use crate::{ qbuffers::DelayBuffer, qubx_common::FilteredSample };

struct DesignButterFilter {
    filt_coeffs: OnePoleCoeffs,
}

impl DesignButterFilter {
    fn new() -> Self {
        let filt_coeffs = OnePoleCoeffs::new();
        Self { filt_coeffs }
    }

    fn coeffs(&mut self, mode: ButterFilter, fc: f32, fs: f32) {
        let twopi = 2.0 * std::f32::consts::PI;
        let wc = twopi * fc;
        let ts = 1.0 / fs;
        let wtan = 2.0 * (wc * ts / 2.0).tan();
        match mode {
            ButterFilter::Lp => {
                let b0 = wtan / (2.0 + wtan);
                let b1 = b0;
                let a1 = (wtan - 2.0) / (2.0 + wtan);

                self.filt_coeffs.set_coeffs((b0, b1, a1))
            },
            ButterFilter::Hp => {
                let b0 = 2.0 / (2.0 + wtan);
                let b1 = -b0;
                let a1 = (wtan - 2.0) / (2.0 + wtan);

                self.filt_coeffs.set_coeffs((b0, b1, a1))
            },
            _ => {}
        }
    }
}


pub struct Butter {
    mode: ButterFilter,
    fs: f32,
    order: usize,
    coeffs: ButterCoeffs,
    xtemp: DelayBuffer,
    ytemp: DelayBuffer,
    x1: DelayBuffer,
    y1: DelayBuffer,
}

impl Butter {

    pub fn new(fs: f32, order: Option<usize>) -> Self {
        let filt_order = order.unwrap_or(1);

        Self { 
            mode: ButterFilter::Lp, 
            fs, 
            order: filt_order,
            coeffs: ButterCoeffs::new(),
            xtemp: DelayBuffer::new(filt_order), 
            ytemp: DelayBuffer::new(filt_order), 
            x1: DelayBuffer::new(filt_order), 
            y1: DelayBuffer::new(filt_order),
        }
    }

    pub fn design_filter(&mut self, mode: ButterFilter, fc: f32, bw: Option<f32>) -> Result<(), FilterError>{
        
        let mut design_filter = DesignButterFilter::new();
        let mut coeffs = ButterCoeffs::new();

        match mode {
            ButterFilter::Lp | ButterFilter::Hp => { 
                design_filter.coeffs(mode, fc, self.fs);
                let b0 = design_filter.filt_coeffs.b0; 
                let b1 = design_filter.filt_coeffs.b1;
                let a1 = design_filter.filt_coeffs.a1;

                coeffs.set_coeffs((b0, b1, a1), (0.0, 0.0, 0.0));
            },
            ButterFilter::Bp => { 
                match bw { 
                    Some(bw_value) => { 
                        design_filter.coeffs(ButterFilter::Lp, fc + bw_value / 2.0, self.fs);
                        let b0lp = design_filter.filt_coeffs.b0; 
                        let b1lp = design_filter.filt_coeffs.b1;
                        let a1lp = design_filter.filt_coeffs.a1;
        
                        design_filter.coeffs(ButterFilter::Hp, fc - bw_value / 2.0, self.fs);
                        let b0hp = design_filter.filt_coeffs.b0; 
                        let b1hp = design_filter.filt_coeffs.b1;
                        let a1hp = design_filter.filt_coeffs.a1;
                        
                        coeffs.set_coeffs((b0lp, b1lp, a1lp), (b0hp, b1hp, a1hp));
                    }
                
                    None => return Err(FilterError::BandWidthNotDefined)
                }
            },
            ButterFilter::Notch => { 
                match bw { 
                    Some(bw_value) => { 
                        design_filter.coeffs(ButterFilter::Lp, fc - bw_value / 2.0, self.fs);
                        let b0lp = design_filter.filt_coeffs.b0; 
                        let b1lp = design_filter.filt_coeffs.b1;
                        let a1lp = design_filter.filt_coeffs.a1;
        
                        design_filter.coeffs(ButterFilter::Hp, fc + bw_value / 2.0, self.fs);
                        let b0hp = design_filter.filt_coeffs.b0; 
                        let b1hp = design_filter.filt_coeffs.b1;
                        let a1hp = design_filter.filt_coeffs.a1;
                        
                        coeffs.set_coeffs((b0lp, b1lp, a1lp), (b0hp, b1hp, a1hp));
                    },
                    None => return Err(FilterError::BandWidthNotDefined)
                }
            }
        };
        self.mode = mode;
        self.coeffs = coeffs;
        Ok(())
    }

    pub fn filt_sample(&mut self, sample: f32) -> f32 {

        let mut x1 = sample;
        let mut x2 = sample;
        let mut y: f32 = 0.0;
        
        for _ in 0..self.order {
            match self.mode {
                ButterFilter::Bp => {
                    let _ytemp = self.coeffs.hp.b0 * x1 + 
                        self.coeffs.hp.b1 * self.xtemp.read_buffer() - 
                        self.coeffs.hp.a1 * self.ytemp.read_buffer();
    
                    y = self.coeffs.hp.b0 * _ytemp + 
                        self.coeffs.hp.b1 * self.x1.read_buffer() - 
                        self.coeffs.hp.a1 * self.y1.read_buffer();
    
                    self.xtemp.write_buffer(x1);
                    self.ytemp.write_buffer(_ytemp);
                    self.x1.write_buffer(_ytemp);
                    self.y1.write_buffer(y);
                    x1 = _ytemp;
                }, 
                ButterFilter::Notch => {
                    let _ylp = self.coeffs.lp.b0 * x1 + 
                        self.coeffs.lp.b1 * self.xtemp.read_buffer() - 
                        self.coeffs.lp.a1 * self.ytemp.read_buffer();
    
                    let _yhp = self.coeffs.hp.b0 * x2 + 
                        self.coeffs.hp.b1 * self.x1.read_buffer() - 
                        self.coeffs.hp.a1 * self.y1.read_buffer();
    
                    self.xtemp.write_buffer(x1);
                    self.ytemp.write_buffer(_ylp);
                    self.x1.write_buffer(x2);
                    self.y1.write_buffer(_yhp);
                    y = _ylp + _yhp;
                    x1 = _ylp;
                    x2 = _yhp;
                }, 
                _ => {
                    y = self.coeffs.hp.b0 * x1 + 
                        self.coeffs.hp.b1 * self.x1.read_buffer() - 
                        self.coeffs.hp.a1 * self.y1.read_buffer();
    
                    self.x1.write_buffer(x1);
                    self.y1.write_buffer(y);
                    x1 = y;
                }
            }
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
        self.xtemp.reset_buffer();
        self.ytemp.reset_buffer();
        self.x1.reset_buffer();
        self.y1.reset_buffer();
    }
}

impl FilteredSample<f32> for Butter 
{ 
    fn filtered_sample(&mut self, sample: f32) -> f32 {
        self.filt_sample(sample)
    }
}