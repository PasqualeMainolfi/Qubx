use crate::{filtertype::FilterError, qubx_common::FilteredSample};

use super::filters::{
    biquadeq::Biquad, 
    butter::Butter, 
    dc::Dc, 
    harmonic::Harmonic, 
    narrow::Narrow, 
    onepole::OnePole, 
    twozerotwopole::TwoZeroTwoPole, 
    zavalishin::Zavalishin,
    filtertype::{ 
        BiquadFilter, 
        ButterFilter, 
        HarmonicFilter,
        NarrowFilter,
        OnePoleFilter,
        TwoZeroTwoPoleFilter,
        ZavalishinFilter
    }
};

/// # QFilters
/// 
/// Implementation references:  
/// - V. Zavalishin, The Art of VA Filter Design, 2018  
/// - Audio EQ Cookbook by Robert Bristow-Johnson, <https://www.musicdsp.org/en/latest/Filters/197-rbj-audio-eq-cookbook.html>  
/// - musicdsp, <https://www.musicdsp.org/en/latest/index.html>  
/// - BQD filter design equation, AN2874 Applications note, STMicroelectronics, 2009  
/// - Filters: <https://www.dspguide.com/ch19/2.htm>  
/// 
/// 
/// # Simple Low Pass and High Pass Filter  
/// **Low Pass One Pole(IIR)**  
/// $$y[n] = b_0x[n] + a_1y[n - 1]$$  
/// where:  
/// $\alpha = e^{-2 \pi \frac{f_c}{f_s}}$  
/// $b_0 = 1 - \alpha$  
/// $a_1 = \alpha$  
/// **High Pass (IIR)**  
/// $$y[n] = b_0x[n] + b_1x[n - 1] + a_1y[n - 1]$$  
/// where:  
/// $\alpha = e^{-2 \pi \frac{f_c}{f_s}}$  
/// $b_0 = \frac{1 + \alpha}{2}$  
/// $b_1 = -b_0$  
/// $a_1 = \alpha$
/// 
/// #  Biquad Filter (AN2874.pdf)
/// The equation is:  
/// $$H(z) = \frac{b_0 + b_1z^{-1} + b_2z^{-2}}{a_0 + a_1z^{-1} + a_2z^{-2}}$$  
/// In time domain:  
/// $$y[n] = \frac{1}{a_0}(b_0x[n] + b_1x[n - 1] + b_2x[n - 2] - a_1y[n - 1] - a_2y[n - 2])$$  
/// Coefficients:  
/// $$a_0 = 1,\; a_1 = \frac{2 (W - 1)}{\alpha},\; a_2 = \frac{1 - \frac{K}{Q} + W}{\alpha}$$  
/// The `b` coefficients for the high pass are:  
/// $$b_0 = \frac{1}{\alpha},\; b_1 = -\frac{2}{\alpha},\; b_2 = b_0$$  
/// The `b` coefficients for the low pass are:  
/// $$b_0 = \frac{W}{\alpha},\; b_1 = \frac{2W}{\alpha},\; b_2 = b_0$$  
/// The band pass is:  
/// $$H_{BP}(z) = H_{LP}(z) \cdot H_{HP}(z)$$
/// where:  
/// $f_c = \text{corner frequency}$  
/// $Q = quality factor$  
/// $f_s = sampling frequency$  
/// $\theta_{c} = \frac{2\pi f_c}{f_s}$  
/// $K = \tan{\frac{\theta_{c}}{2}}$  
/// $W = K^2$  
/// $\alpha = 1 + \frac{K}{Q} + W$  
/// 
/// # Two-Zero Two-Pole Filters  
/// **Two-Zero (Notch Filter)**  
/// $$H(z) = b_0 \left (1 - 2R \cos{(\omega t)} z^{-1} + R^2z^{-2} \right )$$  
/// $$y[n] = b_0x[n] + b_1x[n - 1] + b_2x[n - 2]$$  
/// $b_0 = 1$  
/// $b_1 = - 2R \cos{(\omega t)}$  
/// $b_2 = R^2$  
/// $\text{antiresonance frequency } \omega = 2 \pi \frac{fc}{fs}$  
/// 
/// **Two-Pole (Band Pass Filter)**  
/// $$H(z) = \frac{b_0}{1 + a_1z^{-1} + a_2z^{-2}}$$  
/// $$y[n] = b_0x[n] - a_1y[n - 1] - a_2y[n - 2]$$  
/// $b_0 = 1$  
/// $a_1 = -2R \cos{(2 \pi f Ts)}$  
/// $a_2 = R^2$  
/// 
/// where:  
/// $BW_{Hz} = - \frac{\ln{(R)}}{\pi T}$  
/// $BW_{Hz} = f_{high} - f_{low}$  
/// $f_{low} = f_0 (1 - \frac{1}{2Q})$  
/// $f_{low} = f_0 (1 + \frac{1}{2Q})$  
/// $Q = \text{quality factor}$  
/// $R = e^{-\pi \frac{BW}{fs}}$  
/// $\text{Decay time } \Delta t \approx \frac{Q}{\omega_0}$  
/// $Q = \Delta t \cdot \omega_0$  
/// 
/// 
/// 

#[derive(Debug, Clone, Copy)]
pub enum FilterType
{
    Biquad,
    Butter(usize),
    TwoZeroTwoPole,
    OnePole(usize),
    Narrow(usize),
    Harmonic(usize),
    Zavalishin,
    Dc
}

pub enum FilterParams
{
    BiquadParams(BiquadFilter, f32, f32, Option<f32>),    // mode, fc, q, dbgain
    ButterParams(ButterFilter, f32, Option<f32>),         // mode, fc, bw
    HarmonicParams(HarmonicFilter, f32, Option<f32>),     // mode, t60, fc
    NarrowParams(NarrowFilter, f32, f32),                 // mode, fc, fw
    OnePoleParams(OnePoleFilter, f32),                    // mode, fc
    TwoZeroTwoPoleParams(TwoZeroTwoPoleFilter, f32, f32), // mode, fc, bw
    ZavalishinParams(ZavalishinFilter, f32, Option<f32>), // mode, fc, fc_spread
    DcParams(f32)                                         // fc
}

pub enum Filter
{
    Biquad(Biquad),
    Butter(Butter),
    TwoZeroTwoPole(TwoZeroTwoPole),
    OnePole(OnePole),
    Narrow(Narrow),
    Harmonic(Harmonic),
    Zavalishin(Zavalishin),
    Dc(Dc)
}

impl Filter
{
    pub fn design_filter(&mut self, params: FilterParams) -> Result<(), FilterError>{
        match self {
            Self::Biquad(biquad) => {
                match params {
                    FilterParams::BiquadParams(mode, fc, q, dbgain) => {
                        biquad.design_filter(mode, fc, q, dbgain).unwrap();
                    },
                    _ => return Err(FilterError::FilterCoeffsErrorNotCompatibleMode)
                }
            },
            Self::Butter(butter) => {
                match params {
                    FilterParams::ButterParams(mode, fc, bw) => {
                       butter.design_filter(mode, fc, bw).unwrap();
                    },
                    _ => return Err(FilterError::FilterCoeffsErrorNotCompatibleMode)
                }
            },
            Self::TwoZeroTwoPole(twozerotwopole) => {
                match params {
                    FilterParams::TwoZeroTwoPoleParams(mode, fc, bw) => {
                        twozerotwopole.design_filter(mode, fc, bw);
                    },
                    _ => return Err(FilterError::FilterCoeffsErrorNotCompatibleMode)
                }
            },
            Self::OnePole(onepole) => {
                match params {
                    FilterParams::OnePoleParams(mode, fc) => {
                        onepole.design_filter(mode, fc);
                    },
                    _ => return Err(FilterError::FilterCoeffsErrorNotCompatibleMode)
                }
            },
            Self::Narrow(narrow) => {
                match params {
                    FilterParams::NarrowParams(mode, fc, bw) => {
                        narrow.design_filter(mode, fc, bw);
                    },
                    _ => return Err(FilterError::FilterCoeffsErrorNotCompatibleMode)
                }
            },
            Self::Harmonic(harmonic) => {
                match params {
                    FilterParams::HarmonicParams(mode, t60, fc) => {
                        harmonic.design_filter(mode, t60, fc);
                    },
                    _ => return Err(FilterError::FilterCoeffsErrorNotCompatibleMode)
                }
            },
            Self::Zavalishin(zavalishin) => {
                match params {
                    FilterParams::ZavalishinParams(mode, fc, fc_spread) => {
                        zavalishin.design_filter(mode, fc, fc_spread).unwrap();
                    }
                    _ => return Err(FilterError::FilterCoeffsErrorNotCompatibleMode)
                }
            },
            Self::Dc(dc) => {
                match params {
                    FilterParams::DcParams(fc) => {
                        dc.design_filter(fc);
                    },
                    _ => return Err(FilterError::FilterCoeffsErrorNotCompatibleMode)
                }
            }
        };
        Ok(())
    }

    pub fn filt_sample<T>(&mut self, sample: f32) -> T 
    where
        Biquad: FilteredSample<T>,
        Butter: FilteredSample<T>,
        TwoZeroTwoPole: FilteredSample<T>,
        OnePole: FilteredSample<T>,
        Narrow: FilteredSample<T>,
        Harmonic: FilteredSample<T>,
        Zavalishin: FilteredSample<T>,
        Dc: FilteredSample<T>
    {
        match self {
            Self::Biquad(biquad) => biquad.filtered_sample(sample),
            Self::Butter(butter) => butter.filtered_sample(sample),
            Self::TwoZeroTwoPole(twozerotwopole) => twozerotwopole.filtered_sample(sample),
            Self::OnePole(onepole) => onepole.filtered_sample(sample),
            Self::Narrow(narrow) => narrow.filtered_sample(sample),
            Self::Harmonic(harmonic) => harmonic.filtered_sample(sample),
            Self::Zavalishin(zavalishin) => zavalishin.filtered_sample(sample),
            Self::Dc(dc) => dc.filtered_sample(sample)
        }
    }

    pub fn filt_frame(&mut self, frame:Vec<f32>) -> Result<Vec<f32>, FilterError> {
        let f = match self {
            Self::Biquad(biquad) => biquad.filt_frame(frame),
            Self::Butter(butter) => butter.filt_frame(frame),
            Self::TwoZeroTwoPole(twozerotwopole) => twozerotwopole.filt_frame(frame),
            Self::OnePole(onepole) => onepole.filt_frame(frame),
            Self::Narrow(narrow) => narrow.filt_frame(frame),
            Self::Harmonic(harmonic) => harmonic.filt_frame(frame),
            Self::Dc(dc) => dc.filt_frame(frame),
            Self::Zavalishin(_) => return Err(FilterError::FilterModeNotAllowed),
        };
        Ok(f)
    }


}

pub struct QFilter
{
    sr: f32,
}

impl QFilter
{
    /// Create new Filter Object
    /// 
    /// # Args
    /// -----
    /// 
    /// `sr`: sample rate  
    /// 
    ///
    pub fn new(sr: f32) -> Self 
    {
        Self { sr }
    }

    /// Generate Filter
    /// 
    /// # Args
    /// -----
    /// 
    /// `filter`: type of filter (see `FilterType`)  
    /// 
    /// 
    /// # Return
    /// -------
    /// 
    /// `Filter`  
    ///
    pub fn get_filter(&self, filter: FilterType) -> Filter {
        match filter {
            FilterType::Biquad => Filter::Biquad(Biquad::new(self.sr)),
            FilterType::Butter(order) => Filter::Butter(Butter::new(self.sr, Some(order))),
            FilterType::TwoZeroTwoPole => Filter::TwoZeroTwoPole(TwoZeroTwoPole::new(self.sr)),
            FilterType::OnePole(order) => Filter::OnePole(OnePole::new(self.sr, Some(order))),
            FilterType::Narrow(order) => Filter::Narrow(Narrow::new(self.sr, Some(order))),
            FilterType::Harmonic(buffer_delay) => Filter::Harmonic(Harmonic::new(buffer_delay, self.sr)), 
            FilterType::Zavalishin => Filter::Zavalishin(Zavalishin::new(self.sr)),
            FilterType::Dc => Filter::Dc(Dc::new(self.sr))
        }
    }
}