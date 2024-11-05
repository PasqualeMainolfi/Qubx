#![allow(unused)]

use num_traits::Float;
use rustfft::num_traits::sign;

use crate::qubx_common::ChannelError;
use super::{
    qsignals::SignalObject,
    qenvelopes::{ 
        EnvelopeObject, 
        EnvelopeError 
    },
    qspaces::{ 
        PolarPoint,
        CartesianPoint
    }
};

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
    
    let mut new_sig = Vec::new();
    for i in (0..input.len()).step_by(in_channels) {
        if i + in_channels < input.len() {
            for j in 0..out_channels {
                let start = j * in_channels / out_channels;
                let end = (j + 1) * in_channels / out_channels;
                let sample: f32 = input[start..end].iter().sum();
                new_sig.push(sample / (end - start) as f32)
            }
        }
    }
    *input = new_sig;
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
            for i in 0..(slen - elen) {
                env.push(envelope.vector_envelope[elen - 1])
            }
        },
        m if m < elen => {
            for i in 0..(elen - slen) {
                sig.push(0.0);
            }
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

/// Midi to Frequency
/// 
/// # Args
/// -----
/// 
/// `value`: midi value
/// 
/// # Return
/// --------
/// 
/// `f32`
/// 
#[macro_export]
macro_rules! mtof {
    (&midi:expr) => {{
        let m = $midi.abs();
        440.0 * ((m - 69.0) / 12.0).powi(2)
    }};
}

/// Frequency to Midi
/// 
/// # Args
/// -----
/// 
/// `value`: frequency value in Hz
/// 
/// # Return
/// --------
/// 
/// `f32`
/// 
#[macro_export]
macro_rules! ftom {
    (&freq:expr) => {{
        let f = $freq.abs();
        (12.0 * (f / 440.0).log2() + 69.0).floor()
    }};
}

/// Amp to dB
/// 
/// # Args
/// -----
/// 
/// `value`: amp value
/// 
/// # Return
/// --------
/// 
/// `f32`
/// 
#[macro_export]
macro_rules! atodb {
    (&amp:expr) => {
        (&amp / 20.0).powi(10)
    };
}

/// dB to amp
/// 
/// # Args
/// -----
/// 
/// `value`: dB value
/// 
/// # Return
/// --------
/// 
/// `f32`
///  
#[macro_export]
macro_rules! dbtoa {
    (&db:expr) => {
        (&db / 20.0).powi(10)    
    };
}

/// From degree to rad
/// 
/// # Args
/// -----
/// 
/// `value`: degree value
/// 
/// # Return
/// --------
/// 
/// `f32`
///  
#[macro_export]
macro_rules! degtorad {
    ($degree:expr) => {
        $degree * std::f64::consts::PI / 180.0
    };
}

/// From rad to degree
/// 
/// # Args
/// -----
/// 
/// `value`: rad value
/// 
/// # Return
/// --------
/// 
/// `f32`
///  
#[macro_export]
macro_rules! radtodeg {
    ($rad:expr) => {
        $rad * 180.0 / std::f64::consts::PI
    };
}

/// Make degree in a range from [0, pi]
/// 
/// # Args
/// -----
/// 
/// `value`: degree value
/// 
/// # Return
/// --------
/// 
/// `f32`
///  
#[macro_export]
macro_rules! clamp_angle {
    ($deg:expr) => {{
        if $deg > 180.0 { $deg - 360.0 } else { $deg }
    }};
}

#[macro_export]
macro_rules! cartopol {
    ($car:expr) => {{
        let r = ($car.x * $car.x + $car.y * $car.y).sqrt();
        let theta = $car.y.atan2($car.x);
        PolarPoint { r, theta }
    }};
}

#[macro_export]
macro_rules! poltocar {
    ($pol:expr) => {{
        let x = $pol.r * $pol.theta.cos();
        let y = $pol.r * $pol.theta.sin();
        CartesianPoint { x, y }
    }};
}

#[macro_export]
macro_rules! scale_in_range {
    ($angle:expr, $in_min:expr, $in_max:expr, $out_min:expr, $out_max:expr) => {{
        $out_min + (($angle - $in_min) / ($in_max - $in_min)) * ($out_max - $out_min)
    }};
}


