#![allow(clippy::only_used_in_recursion)]

use std::fmt::Debug;
use realfft::RealFftPlanner;
use rustfft::num_complex::Complex;

/// Convolution methods
/// 
#[derive(Debug, Clone, Copy)]
pub enum ConvolutionMode {
    OutputSide,
    InputSide,
    Fft,
    OlaFft(usize),
}

#[derive(Debug, Default)]
pub struct QConvolution { }

impl QConvolution
{
    /// Generate convolution 
    /// 
    /// # Args:
    /// -----
    /// 
    /// `x`: signal
    /// `h`: kernel
    /// `mode`: convolution mode (see `ConvolutionMode`)
    /// 
    /// 
    /// # Return
    /// -------
    /// 
    /// `Vec<f32>` with len equal to n + m - 1
    /// 
    pub fn convolve(x: &[f32], h: &[f32], mode: ConvolutionMode) -> Vec<f32> {
        let xlen = x.len();
        let hlen = h.len();
        let ylen = xlen + hlen - 1;
        let mut y: Vec<f32> = vec![0.0; ylen];

        match mode {
            ConvolutionMode::InputSide => QConvolution::_input_side_helper(x, h, &mut y),
            ConvolutionMode::OutputSide => QConvolution::_output_side_helper(x, h, &mut y),
            ConvolutionMode::Fft => {
                let pow_size = 1 << ((ylen as f32).log2() + 1.0) as usize;
                let mut xpad = vec![0.0; pow_size];
                let mut hpad = vec![0.0; pow_size];
                xpad[..xlen].copy_from_slice(x);
                hpad[..hlen].copy_from_slice(h);
                QConvolution::_fft_helper(&mut xpad, &mut hpad, &mut y)
            },
            ConvolutionMode::OlaFft(value) => QConvolution::_olafft_helper(x, h, &mut y, value)
        }
        y
    }

    fn _input_side_helper(x: &[f32], h: &[f32], buffer: &mut [f32]) {
        for k in 0..h.len() {
            for i in 0..x.len() {
                buffer[k + i] += h[k] * x[i]
            }
        }
    }
    
    fn _output_side_helper(x: &[f32], h: &[f32], buffer: &mut [f32]) {
        for (n, value) in buffer.iter_mut().enumerate() {
            let lower = 0.max(n as isize - x.len() as isize + 1) as usize;
            let upper = n.min(h.len() - 1);
            for k in lower..=upper {
                *value += h[k] * x[n - k];
            }
        }
    }

    fn _fft_helper(x: &mut [f32], h: &mut [f32], buffer: &mut [f32]) {
        let xlen = x.len();

        let mut xplanner = RealFftPlanner::<f32>::new();
        let mut hplanner = RealFftPlanner::<f32>::new();
        
        let xfft = xplanner.plan_fft_forward(xlen);
        let hfft = hplanner.plan_fft_forward(xlen);

        let mut xspectrum = xfft.make_output_vec();
        let mut hspectrum = hfft.make_output_vec();

        xfft.process(x, &mut xspectrum).unwrap();
        hfft.process(h, &mut hspectrum).unwrap();

        let mut fft_prod = xspectrum.iter().zip(hspectrum.iter()).map(|(&a, &b)| a * b).collect::<Vec<Complex<f32>>>();

        let ifft = xplanner.plan_fft_inverse(xlen);
        let mut ifft_time = ifft.make_output_vec();
        ifft.process(&mut fft_prod, &mut ifft_time).unwrap();

        for (i, value) in buffer.iter_mut().enumerate() {
            *value = ifft_time[i] / xlen as f32; // return the buffer with len n + m - 1
        }

    }

    fn _olafft_helper(x: &[f32], h: &[f32], buffer: &mut [f32], frame_size: usize) {
        let xlen = x.len();
        let hlen = h.len();

        let nframes: usize = (xlen as f32 / frame_size as f32).ceil() as usize; // get number of total frames
        let xlen_new = nframes * frame_size; // adjust x len = nframes * frames size
        
        let mut xnewlen = vec![0.0; xlen_new];
        xnewlen[..xlen].copy_from_slice(&x[..xlen]);

        let len_fft_buffer = frame_size + hlen - 1; // len of fft buffer -> frame size + kernel size - 1
        let ylen = nframes * frame_size + hlen - 1; // len of result vector

        let pow_len = 1 << ((len_fft_buffer as f32).log2() + 1.0) as usize;
        let mut hpad = vec![0.0; pow_len];
        hpad[..hlen].copy_from_slice(&h[..hlen]);

        let mut xframe = vec![0.0; pow_len];

        let mut fft_buffer: Vec<f32> = vec![0.0; len_fft_buffer]; // fft buffer
        let mut y = vec![0.0; ylen]; // result vector
        
        for i in 0..nframes {
            let frame_start = i * frame_size;
            let frame_end = (i + 1) * frame_size;
            xframe[..frame_size].copy_from_slice(&xnewlen[frame_start..frame_end]);
            QConvolution::_fft_helper(&mut xframe, &mut hpad, &mut fft_buffer);

            for j in 0.. len_fft_buffer {
                y[frame_start + j] += fft_buffer[j]; // rebuild from ola with len nframes * frame size + hsize - 1
            }
        }

        for (i, value) in buffer.iter_mut().enumerate() {
            *value = y[i]; // return the buffer with len n + m - 1
        }

    }

}