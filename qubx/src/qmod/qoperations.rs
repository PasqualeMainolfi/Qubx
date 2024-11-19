// #![allow(unused)]

use num_traits::Float;
use rustfft::num_complex::Complex;

use crate::qubx_common::ChannelError;
use super::{
    qsignals::SignalObject,
    qenvelopes::{ 
        EnvelopeObject, 
        EnvelopeError 
    }
};


pub struct ComplexNum { }

impl ComplexNum
{
    pub fn new_complex<T>(re: T, im: T) -> Complex<T> {
        Complex { re, im }
    } 
}

/// Split signal into n-channels  
/// 
/// Split channels using uniform distribution  
/// 
/// # Args:
/// ------
/// 
/// `input`: input signal  
/// `in_channels`: input channel numbers
/// `out_channels`: output signal channel numbers
/// 
/// # Return
/// --------
/// 
/// `Result<(), ChannelError>`
/// 
pub fn split_into_nchannels(input: &mut Vec<f32>, in_channels: usize, out_channels: usize) -> Result<(), ChannelError> {
    if input.is_empty() { return Err(ChannelError::VectorIsEmpty) }
    if in_channels == 0 { return Err(ChannelError::ChannelNumbersError) }
    
    if in_channels != out_channels {
        let mut new_sig = Vec::new();
        for i in (0..input.len()).step_by(in_channels) {
            let segment = &input[i..i + in_channels];
            for j in 0..out_channels {
                let sample = if in_channels == 1 {
                    segment[0]
                } else {
                    let ratio = (j as f32) / (out_channels as f32 - 1.0);
                    let interpolated_sample = segment
                        .iter()
                        .zip((0..in_channels).map(|k| k as f32 / (in_channels as f32 - 1.0)))
                        .map(|(sample, pos)| sample * (1.0 - (pos - ratio).abs()))
                        .sum::<f32>();
                    interpolated_sample
                };
                new_sig.push(sample / out_channels as f32);
            }
        }
        *input = new_sig;
        return Ok(())
    }
    Ok(())
}

/// Apply envelope to signal
/// 
/// # Args
/// -----
/// 
/// `signal`: Input Signal must be `SignalObject`
/// `envelope`: Envelope shape must be `EnvelopeObject`
/// 
/// # Return
/// --------
/// 
/// `Result<SignalObject, EnvelopeError>`
/// 
pub fn envelope_to_signal(signal: &SignalObject, envelope: &EnvelopeObject) -> Result<SignalObject, EnvelopeError>{
    if signal.n_channels != envelope.n_channels { return Err(EnvelopeError::EnvToSignalErrorDifferentChannelNumbers) }
    let slen = signal.vector_signal.len();
    let elen = envelope.vector_envelope.len();

    let mut sig = vec![0.0; slen];
    sig.copy_from_slice(&signal.vector_signal);
    let mut env = vec![0.0; elen];
    env.copy_from_slice(&envelope.vector_envelope);

    match slen {
        m if m > elen => {
            env = vec![envelope.vector_envelope[elen - 1]; slen];
            env[..elen].copy_from_slice(&envelope.vector_envelope)
        },
        m if m < elen => {
            sig = vec![0.0; elen];
            sig[..slen].copy_from_slice(&signal.vector_signal)
        }
        _ => { }
    }

    let enveloped_signal = signal.vector_signal
        .iter()
        .zip(envelope.vector_envelope.iter())
        .map(|(s, e)| s * e)
        .collect::<Vec<f32>>();

    Ok(SignalObject { vector_signal: enveloped_signal, n_channels: signal.n_channels, sr: signal.sr })
}

pub fn precision_float<T: Float>(value: T, n_decimals: i32) -> T {
    let f = T::from(10.0.powi(n_decimals)).unwrap();
    (value * f).round() / f
}

// ---

pub fn vector_zero_padding<T: Float>(x: &[T], length: usize) -> Vec<T> {
    let mut padded = Vec::<T>::with_capacity(length);
    padded.copy_from_slice(x);
    padded
}

