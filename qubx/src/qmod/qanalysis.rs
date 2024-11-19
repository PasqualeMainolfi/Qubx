#![allow(unused)]

use core::f32;

use nalgebra::DMatrix;
use num_traits::Float;
use realfft::RealFftPlanner;
use rustfft::{ num_complex::{ Complex, ComplexFloat}, FftPlanner };
use rustdct::DctPlanner;
use crate::{
    next_power_ot_two_length,
    qubx_common::{
        FreqDomainToFloat,
        FreqDomainToComplex,
        TimeDomainFloat
    },
    atodb,
    meltof,
    ftomel
};
use super::{
    qoperations::{
        vector_zero_padding,
        ComplexNum
    },
    qwindow::QWindow,
    macros::{
        ctoangle,
        ctomag,
        rtoc,
        comp_conj
    }
};

#[derive(Debug)]
pub enum StftError
{
    WinSizeMustBeLessThanInput,
    HopSizeMustBeGratherThanZero
}


/// Fft object
/// 
pub struct FftObject
{
    pub fft_vector: Vec<Complex<f32>>
}

impl FreqDomainToFloat for FftObject
{
    type FftType = Vec<f32>;

    fn get_angle(&self) -> Vec<f32> {
        let mut angles = Vec::new();
        for value in self.fft_vector.iter() {
            angles.push(ctoangle(*value));
        }
        angles
    }

    fn get_mag(&self) -> Vec<f32> {
        let mut mags = Vec::new();
        for value in self.fft_vector.iter() {
            mags.push(ctomag(*value));
        }
        mags
    }

    fn get_db(&self) -> Vec<f32> {
        let mut db = Vec::new();
        for value in self.fft_vector.iter() {
            db.push(atodb!(ctomag(*value)))
        }
        db.clone()
    }
}

impl FreqDomainToComplex for FftObject
{
    type FftType = Vec<Complex<f32>>;

    fn get_conj(&self) -> Vec<Complex<f32>> {
        let mut conj = Vec::new();
        for value in self.fft_vector.iter() {
            conj.push(comp_conj(*value));
        }
        conj
    }
}


/// Stft object
/// 
#[derive(Debug)]
pub struct StftObject
{
    pub stft_matrix: Vec<Vec<Complex<f32>>>,
    pub frequencies: Vec<f32>,
    pub times: Vec<f32>,
    pub sr: f32,
    win_size: usize,
    hop_size: usize,
    nrows: usize,
    ncols: usize,
}

impl FreqDomainToFloat for StftObject
{
    type FftType = Vec<Vec<f32>>;

    fn get_angle(&self) -> Vec<Vec<f32>> {
        let angles = self.stft_matrix
            .iter()
            .map(|row| {
                row.iter().map(|&value| ctoangle(value)).collect::<Vec<f32>>()
            })
            .collect::<Vec<Vec<f32>>>();
        angles
    }

    fn get_mag(&self) -> Vec<Vec<f32>> {
        let mags = self.stft_matrix
            .iter()
            .map(|row| {
                row.iter().map(|&value| ctomag(value)).collect::<Vec<f32>>()
            })
            .collect::<Vec<Vec<f32>>>();
        mags
    }

    fn get_db(&self) -> Vec<Vec<f32>> {
        let db = self.stft_matrix
            .iter()
            .map(|row| {
                row.iter().map(|&value| atodb!(ctomag(value))).collect::<Vec<f32>>()
            })
            .collect::<Vec<Vec<f32>>>();
        db
    }

}

impl FreqDomainToComplex for StftObject
{
    type FftType = Vec<Vec<Complex<f32>>>;

    fn get_conj(&self) -> Vec<Vec<Complex<f32>>> {
        let mut conj = vec![vec![ComplexNum::new_complex(0.0, 0.0); self.ncols]; self.nrows];
        for (i, row) in self.stft_matrix.iter().enumerate() {
            for (j, value) in row.iter().enumerate() {
                conj[i][j] = comp_conj(*value)
            }
        }
       conj
    }
}

pub struct QSpectra
{
    planner: FftPlanner<f32>,
    real_planner: RealFftPlanner<f32>,
    dct_planner: DctPlanner<f32>

}

impl Default for QSpectra
{
    fn default() -> Self {
        Self {
            planner: FftPlanner::<f32>::new(),
            real_planner: RealFftPlanner::<f32>::new(),
            dct_planner: DctPlanner::new()
        }
    }
}

impl QSpectra
{
    /// Define new QSpectra object
    /// 
    pub fn new() -> Self {
       Self::default()
    }

    /// FFT  
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input vector  
    /// 
    /// # Return
    /// -------
    /// 
    /// `FftObject`
    /// 
    pub fn fft(&mut self, x: &[f32]) -> FftObject {
        let n = x.len();
        let xfft = self.planner.plan_fft_forward(n);
        let mut buffer_spectrum = x
            .iter()
            .map(|&value| rtoc(value))
            .collect::<Vec<Complex<f32>>>();
        xfft.process(&mut buffer_spectrum);
        FftObject { fft_vector: buffer_spectrum }
    }

    /// Real FFT  
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input vector  
    /// 
    /// # Return
    /// -------
    /// 
    /// `FftObject`
    /// 
    pub fn rfft(&mut self, x: &mut [f32]) -> FftObject {
        let n = x.len();
        let xfft = self.real_planner.plan_fft_forward(n);
        let mut buffer = xfft.make_output_vec();
        xfft.process(x, &mut buffer).unwrap();
        FftObject { fft_vector: buffer }
    }

    /// IFFT  
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input vector (complex)   
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<f32>`
    /// 
    pub fn ifft(&mut self, x: &[Complex<f32>]) -> Vec<f32> {
        let n = x.len();
        let xifft = self.planner.plan_fft_inverse(n);
        let mut buffer_inverse = x.to_vec();
        xifft.process(&mut buffer_inverse);
        let inverse = buffer_inverse.iter().map(|value| value.re).collect::<Vec<f32>>();
        inverse
    }

    /// IFFT from real FFT
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input vector (complex)   
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<f32>`
    /// 
    pub fn rifft(&mut self, x: &mut [Complex<f32>]) -> Vec<f32> {
        let n = x.len();
        let xifft = self.real_planner.plan_fft_inverse(n);
        let mut buffer_inverse = xifft.make_output_vec();
        xifft.process(x, &mut buffer_inverse).unwrap();
        buffer_inverse
    }

    /// STFT  
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input vector  
    /// `win_size`: frame size in samples  
    /// `hop_size`: overlap size in samples  
    /// `window`: window function type (see `QWindow`)  
    /// `sr`: sample rate 
    /// 
    /// # Return
    /// -------
    /// 
    /// `Result<StftObject, StftError>`
    /// 
    pub fn stft(&mut self, x: &[f32], win_size: usize, hop_size: usize, window: QWindow, sr: f32) -> Result<StftObject, StftError> {

        if hop_size == 0 { return Err(StftError::HopSizeMustBeGratherThanZero) }

        let xlen = x.len();
        let n = next_power_ot_two_length!(xlen);

        if win_size >= n { return Err(StftError::WinSizeMustBeLessThanInput) }

        let xpadded = vector_zero_padding::<f32>(x, n);

        let nrows = win_size / 2 + 1;
        let ncols = (n - win_size) / hop_size;
        let mut stft = vec![vec![Complex { re: 0.0, im: 0.0 }; ncols]; nrows];
        let mut chunk = vec![0.0; win_size];

        let planner = self.real_planner.plan_fft_forward(win_size);
        let mut fft_buffer = planner.make_output_vec();
        let mut scratch_buffer = planner.make_scratch_vec();

        let win = window.get_window(win_size);

        for i in 0..ncols {
            let start = i * hop_size;
            if start < n - 2 {
                let end = n.min(start + win_size);
                chunk.copy_from_slice(&xpadded[start..end]);

                for (k, value) in chunk.iter_mut().enumerate() {
                    *value *= win[k];
                }

                planner.process_with_scratch(&mut chunk, &mut fft_buffer, &mut scratch_buffer).unwrap();
                stft[..][i].copy_from_slice(&fft_buffer);
            }
        }

        let frequencies = (0..nrows).map(|i| sr * i as f32 / win_size as f32).collect::<Vec<f32>>();
        let times = (0..ncols).map(|i| hop_size as f32 / sr * i as f32).collect::<Vec<f32>>();

        Ok(StftObject { stft_matrix: stft, frequencies, times, sr, win_size, hop_size, nrows, ncols } )

    }

    /// ISTFT  
    /// 
    /// # Args:
    /// -----
    /// 
    /// `stft`: stft input as `StftObject`  
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<f32>`
    /// 
    pub fn istft(&mut self, stft: &StftObject) -> Vec<f32> {
        let xlen = (stft.ncols * stft.hop_size) + stft.nrows;
        let mut x: Vec<f32> = vec![0.0; xlen];
        let xifft = self.real_planner.plan_fft_inverse(stft.win_size);
        let mut buffer_ifft = xifft.make_output_vec();
        let mut scratch_buffer = xifft.make_scratch_vec();
        let mut buffer_chunk = vec![Complex { re: 0.0, im: 0.0 }; stft.nrows];
        for i in 0..stft.ncols {
            buffer_chunk.copy_from_slice(&stft.stft_matrix[..][i]);
            xifft.process_with_scratch(&mut buffer_chunk, &mut buffer_ifft, &mut scratch_buffer).unwrap();
            let hop = i * stft.hop_size;
            for (k, value) in buffer_ifft.iter().enumerate() {
                x[k + hop] += *value / stft.nrows as f32
            }
        }
        x
    }

    /// Mel frequency cepstral coefficients (MFCC)
    /// 
    /// # Args:
    /// -----
    /// 
    /// `stft`: stft input as `StftObject`  
    /// `n_filters`: number of filters  
    /// `freqrange`: frequencies range (low, high)  
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<Vec<f32>>`
    /// 
    pub fn mfcc(&mut self, stft: &StftObject, n_filters: usize, freqrange: (f32, f32)) -> Vec<Vec<f32>> {
        let power_spectrum = FromComplexData::get_db(stft);

        let mellow = ftomel!(freqrange.0);
        let melhigh = ftomel!(freqrange.1);
        let mel_range = melhigh - mellow;
        let step = mel_range / n_filters as f32;
        let mut bins: Vec<usize> = Vec::with_capacity(n_filters);
        
        for (i, bin) in bins.iter_mut().enumerate() {
            let f1 = mellow + (step * i as f32);
            let f2 = meltof!(f1);
            *bin = ((stft.win_size as f32 + 1.0) * f2 / stft.sr).floor() as usize;
        }

        let mut triangle = DMatrix::<f32>::zeros(stft.ncols, stft.nrows);

        for i in 1..triangle.nrows() - 1 {
            let (a, b, c) = (bins[i - 1], bins[i], bins[i + 1]);
            for j in 0..triangle.ncols() {
                let v = match j {
                    p if p < a => 0.0,
                    p if p >= a && p <= b => (j - a) as f32 / (b - a) as f32,
                    p if p >= b && p <= c => (c - j) as f32 / (c - b) as f32,
                    p if p > c => 0.0,
                    _ => 0.0
                };
                triangle[(i - 1, j)] = v;
            }
        }

        let flatten_stft = power_spectrum.into_iter().flatten().collect::<Vec<f32>>();
        let power_matrix = DMatrix::from_vec(stft.nrows, stft.ncols, flatten_stft);
        let mut filtered_db = triangle * power_matrix;

        for i in 0..filtered_db.nrows() {
            for j in 0..filtered_db.ncols() {
                filtered_db[(i, j)] = 20.0 * (filtered_db[(i, j)]).log10()
            }
        }
        
        filtered_db = filtered_db.transpose();
        let mut melfcc = DMatrix::<f32>::zeros(filtered_db.nrows(), filtered_db.ncols());
        let dct = self.dct_planner.plan_dct2(filtered_db.ncols());
        let mut scratch_buffer = vec![0.0; filtered_db.ncols()];

        for (i, row) in filtered_db.row_iter().enumerate() {
            let mut row_vec = row.iter().cloned().collect::<Vec<f32>>();
            dct.process_dct2_with_scratch(&mut row_vec, &mut scratch_buffer);
            for j in 0..filtered_db.ncols() {
                melfcc[(i, j)] = row_vec[j] / filtered_db.ncols() as f32;
            }
        }

        melfcc = melfcc.transpose();
        let melfcc: Vec<Vec<f32>> = (0..melfcc.nrows())
            .map(|i| melfcc.row(i).iter().cloned().collect()).collect();

        melfcc

    }

}

pub struct FromComplexData { }

impl FromComplexData {

    /// Get angle
    /// 
    pub fn get_angle<T: FreqDomainToFloat>(data: &T) -> T::FftType {
        data.get_angle()
    }

    /// Get mag
    /// 
    pub fn get_mag<T: FreqDomainToFloat>(data: &T) -> T::FftType {
        data.get_mag()
    }

    /// Get complex conj
    /// 
    pub fn get_conj<T: FreqDomainToComplex>(data: &T) -> T::FftType {
        data.get_conj()
    }

    /// Get dB
    /// 
    pub fn get_db<T: FreqDomainToFloat>(data: &T) -> T::FftType {
        data.get_db()
    }

    /// Spectral Centroid  
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input stft as `StftObject`  
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<f32>`
    /// 
    pub fn spectral_centroid(x: &StftObject) -> Vec<f32> { 
        let sr = x.sr / 2.0;
        let mut centroid: Vec<f32> = Vec::with_capacity(x.ncols);
        for (col, value) in centroid.iter_mut().enumerate() {
            let mut num = 0.0; 
            let mut den = 0.0;
            for row in 0..x.nrows {
                let r = row as f32;
                let f = sr * r / r;
                let value_in = x.stft_matrix[row][col].abs();
                num += value_in * f;
                den += value_in;
            }
            *value = num / den;
        }
        centroid
    }

    /// Spectral Spread  
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input stft as `StftObject`   
    /// `spectral_centroid`: spectral centroid  
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<f32>`
    /// 
    pub fn spectral_spread(x: &StftObject, spectral_centroid: &[f32]) -> Vec<f32> { 
        let sr = x.sr / 2.0;
        let mut spread: Vec<f32> = Vec::with_capacity(x.ncols);
        for (col, value) in spread.iter_mut().enumerate() {
            let mut num = 0.0; 
            let mut den = 0.0;
            for row in 0..x.nrows {
                let r = row as f32;
                let f = sr * r / r;
                let value_in = x.stft_matrix[row][col].abs().powi(2);
                let s = (f - spectral_centroid[col]).powi(2);
                num += value_in * s;
                den += value_in;
            }
            *value = (num / den).sqrt();
        }
        spread
    }

    fn _spectral_skewness(x: &StftObject, spectral_centroid: &[f32], spectral_spread: &[f32], p: i32) -> Vec<f32> { 
        let sr = x.sr / 2.0;
        let mut skew: Vec<f32> = Vec::with_capacity(x.ncols);
        for (col, value) in skew.iter_mut().enumerate() {
            let mut num = 0.0; 
            let mut den = 0.0;
            for row in 0..x.nrows {
                let r = row as f32;
                let f = sr * r / r;
                let value_in = x.stft_matrix[row][col].abs();
                let s = (f - spectral_centroid[col]).powi(p);
                num += value_in * s;
                den += value_in;
            }
            *value = num / (spectral_spread[col].powi(3) * den);
        }
        skew
    }
    
    /// Spectral Skewness  
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input stft as `StftObject`   
    /// `spectral_centroid`: spectral centroid  
    /// `spectral_spread`: spectral spread  
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<f32>`
    /// 
    pub fn spectral_skewness(x: &StftObject, spectral_centroid: &[f32], spectral_spread: &[f32]) -> Vec<f32> { 
        FromComplexData::_spectral_skewness(x, spectral_centroid, spectral_spread, 3)
    }
    
    /// Spectral Kurtosis  
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input stft as `StftObject`   
    /// `spectral_centroid`: spectral centroid  
    /// `spectral_spread`: spectral spread  
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<f32>`
    /// 
    pub fn spectral_kurtosis(x: &StftObject, spectral_centroid: &[f32], spectral_spread: &[f32]) -> Vec<f32> { 
        FromComplexData::_spectral_skewness(x, spectral_centroid, spectral_spread, 4)
    }
    
    pub fn spectral_entropy(x: &StftObject) -> Vec<f32> { 
        let sr = x.sr / 2.0;
        let mut entropy: Vec<f32> = Vec::with_capacity(x.ncols);
        for (col, value) in entropy.iter_mut().enumerate() {
            let mut num = 0.0; 
            for row in 0..x.nrows {
                let r = row as f32;
                let f = sr * r / r;
                let value_in = x.stft_matrix[row][col].abs();
                num += f * value_in.ln();
            }
            let b1 = (x.hop_size * col) as f32;
            let b2 = b1 + x.nrows as f32;
            *value = -num * (b2 - b1).ln();
        }
        entropy
    }
    
    /// Spectral Rolloff  
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input stft as `StftObject`   
    /// `k`: rolloff coefficient  
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<f32>`
    /// 
    pub fn spectral_rolloff(x: &StftObject, k: f32) -> Vec<f32> { 
        let mut rolloff: Vec<f32> = Vec::with_capacity(x.ncols);
        for (col, value) in rolloff.iter_mut().enumerate() {
            let mut num = 0.0; 
            for row in 0..x.nrows {
                num += x.stft_matrix[row][col].abs();
            }
            *value = num * k;
        }
        rolloff
    }
    
    /// Spectral Flux  
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input stft as `StftObject`   
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<f32>`
    /// 
    pub fn spectral_flux(x: &StftObject) -> Vec<f32> { 
        let mut flux: Vec<f32> = Vec::with_capacity(x.ncols);
        for (col, value) in flux.iter_mut().enumerate() {
            let mut num = 0.0; 
            for row in 0..(x.nrows - 1) {
                num += (x.stft_matrix[row + 1][col].abs() - x.stft_matrix[row][col].abs()).powi(2);
            }
            *value = num.sqrt() / x.nrows as f32;
        }
        flux 
    }
    
    /// Spectral Crest  
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input stft as `StftObject`   
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<f32>`
    /// 
    pub fn spectral_crest(x: &StftObject) -> Vec<f32> { 
        let mut crest: Vec<f32> = Vec::with_capacity(x.ncols);
        for (col, value) in crest.iter_mut().enumerate() {
            let mut num = 0.0; 
            for row in 0..x.nrows {
                num += x.stft_matrix[row][col].abs();
            }

            let mut e = 0.0;
            for i in 0..x.nrows {
                e = e.max(x.stft_matrix[i][col].abs())
            }

            let b1 = (x.hop_size * col) as f32;
            let b2 = b1 + x.nrows as f32;
            let fac = 1.0 / (b2 - b1);

            *value = e / fac * num;
        }
        crest 
    }

    /// Spectral Slope  
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input stft as `StftObject`   
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<f32>`
    /// 
    pub fn spectral_slope(x: &StftObject) -> Vec<f32> {
        let sr = x.sr / 2.0;
        let mut slope: Vec<f32> = Vec::with_capacity(x.ncols);

        let power = FromComplexData::get_mag(x);
        let fs = (0..x.ncols).map(|i| x.sr * i as f32 / x.ncols as f32).collect::<Vec<f32>>();
        let muf = fs.iter().sum::<f32>() / fs.len() as f32;
        let mut mup = Vec::new();
        for i in 0..x.ncols {
            let s = (0..x.nrows).map(|j| power[j][i]).sum::<f32>();
            mup.push(s / x.nrows as f32)
        }

        slope.iter_mut().enumerate().for_each(|(col, value)| {
            let (num, den) = (0..x.nrows).fold((0.0, 0.0), |(n, d), r| {
                let f = sr * r as f32 / r as f32;
                (
                    n + (f - muf) * power[r][col] - mup[col],
                    d + (f - muf).powi(2)
                )
            });
            *value = num / den
        });

        slope
    }
    
    /// Band energy ratio  
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input stft as `StftObject` 
    /// `frequency_split`: frequency in Hz that divide the whole spectrum into two ranges: low frequency range f < frequency_split and high frequency range for f >= frequency_split      
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<f32>`
    /// 
    pub fn band_energy_ratio(x: &StftObject, frequency_split: f32) -> Vec<f32> {
        let frange = x.sr / 2.0;
        let fdelta = frange / x.nrows as f32;
        let fsplit_bin = (frequency_split / fdelta).floor() as usize;
        let mut ps = FromComplexData::get_mag(x);
        let ps = ps
            .iter_mut().map(|row| {
                row.iter_mut().map(|v| v.powi(2)).collect::<Vec<f32>>() 
            })
            .collect::<Vec<Vec<f32>>>();
        
        let mut ber = Vec::with_capacity(x.ncols);
        ber.iter_mut().enumerate().for_each(|(col, value)| {
            let (lf, hf) = (0..fsplit_bin).fold((0.0, 0.0), |(lowf, highf), row| {
                (
                    lowf + ps[row][col],
                    highf + ps[row + fsplit_bin][col]
                )
            });
            *value = lf / hf
        });
        ber
    }

}

pub struct FromRealData { }

impl FromRealData
{
    /// Amplitude Envelope  
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input object  
    /// `chunk_length`: frame length in samples     
    /// `hop_size`: overlap size in samples     
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<f32>`
    /// 
    pub fn amplitude_envelope<T: TimeDomainFloat>(x: &T, chunk_length: usize, hop_size: usize) -> Vec<f32> {
        let signal = x.get_vector();
        let mut peeks = Vec::new();
        for i in (0..signal.len()).step_by(hop_size) {
            let end = signal.len().min(i + chunk_length);
            let mut peek = 0.0;
            for value in signal[i..end].iter() {
                peek = if value.abs() > peek { *value } else { peek }
            }
            peeks.push(peek);
        }
        peeks
    }
    
    /// Zero crossing rate  
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input object  
    /// `chunk_length`: frame length in samples     
    /// `hop_size`: overlap size in samples     
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<f32>`
    /// 
    pub fn zero_crossing_rate<T: TimeDomainFloat>(x: &T, chunk_length: usize, hop_size: usize) -> Vec<f32> {
        let signal = x.get_vector();
        let mut zcr = Vec::new();

        for i in (0..signal.len()).step_by(hop_size) {
            let end = signal.len().min(i + chunk_length);
            let mut z = 0.0;
            for j in 0..(end - i - 1) {
                z += (signal[j + hop_size + 1].signum() - signal[j + hop_size].signum()).abs();
            }
            z /= 2.0 * chunk_length as f32;
            zcr.push(z);
        }
        zcr
    }

    /// Energy (RMS)    
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: input object  
    /// `chunk_length`: frame length in samples     
    /// `hop_size`: overlap size in samples     
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<f32>`
    /// 
    pub fn energy<T: TimeDomainFloat>(x: &T, chunk_length: usize, hop_size: usize) -> Vec<f32> {
        let signal = x.get_vector();
        let mut rms = Vec::new();

        for i in (0..signal.len()).step_by(hop_size) {
            let end = signal.len().min(i + chunk_length);
            let mut r = 0.0;
            for j in 0..(end - i) {
                r += signal[j + hop_size].abs().powi(2);
            }
            r = (r / chunk_length as f32).sqrt();
            rms.push(r);
        }
        rms
    }
}
