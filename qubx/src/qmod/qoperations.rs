#![allow(unused)]

use rustfft::num_traits::sign;

use crate::qubx_common::ChannelError;
use super::qsignals::SignalObject;
use super::qenvelopes::{ EnvelopeObject, EnvelopeError };

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