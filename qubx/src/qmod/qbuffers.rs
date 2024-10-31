#![allow(unused)]

use std::default;
use std::io::{ Read, Write };
use std::process::{ Command, Stdio };
use portaudio::stream::Buffer;
use rustfft::Length;
use std::path::Path;
use std::fs;

use crate::qubx_common::{ Channels, ChannelError, WriteToFile, ToFileError };
use super::{
    qsignals::SignalObject,
    qoperations::split_into_nchannels,
    shared_tools::{ update_increment, update_and_reset_increment, interp_buffer_write, write_to_file },
    qinterp::{ Interp, PhaseInterpolationIndex }
};

// pub enum BitSize
// {
//     Eight,
//     Sixteen,
//     TwentyFour,
//     ThirtyTwo,
//     SixtyFour
// }

// pub enum AudioCodec
// {
//     PcmInt(BitSize),
//     PcmFloat(BitSize)
// }

// impl AudioCodec
// {
//     fn get_codec<'a >(&self) -> Result<&'a str, BufferError> {
//         let format = match self {
//             Self::PcmInt(bit) => {
//                 match bit {
//                     BitSize::Eight => "s8",
//                     BitSize::Sixteen => "s16le", 
//                     BitSize::TwentyFour => "s24le",
//                     BitSize::ThirtyTwo => "s32le",
//                     _ => return Err(BufferError::WriteFormatNotValid)
//                 }
//             },
//             Self::PcmFloat(bit) => {
//                 match bit {
//                     BitSize::ThirtyTwo => "f32le",
//                     BitSize::SixtyFour => "f64le",
//                     _ => return Err(BufferError::WriteFormatNotValid)
//                 }
//             }
//         };
//         Ok(format)
//     }
// }

#[derive(Debug, Clone, Copy)]
pub enum BufferError
{
    ErrorInReadingFile,
    ErrorInReadingChannelNumbers,
    NullOpenFileBufferEmpty,
    BufferLengthExceeded,
    ReadOffsetGratherThanAudioLength,
    SamplerDataDurationReached,
    WriteFormatNotValid,
    ErrorInWritingFile
}

#[derive(Debug)]
pub struct AudioObject
{
    pub vector_signal: Vec<f32>,
    pub n_channels: usize,
    pub sr: f32,
    pub(crate) read_speed: f32,
    pub(crate) read_offset: f32,
    pub(crate) read_again: bool,
    pub(crate) n_samples: usize,
    pub(crate) phase_motion: f32,
    pub(crate) interp_buffer: Vec<f32>,
    pub(crate) elapsed_time: usize,
    duration: f32
}

impl AudioObject
{
    /// Create new AudioObject
    /// 
    /// # Args
    /// -----
    /// 
    /// `vector_signal`: signal as vector
    /// `n_channels`: number of channels
    /// `sr`: sample rate
    /// 
    /// 
    pub fn new(vector_signal: Vec<f32>, n_channels: usize, sr: f32) -> Self {
        let n_samples = vector_signal.len() / n_channels;
        let duration = n_samples as f32 / sr;
        Self { 
            vector_signal, 
            n_channels,
            sr, read_speed: 1.0, 
            read_offset: 0.0, 
            read_again: false, 
            n_samples, 
            phase_motion: 0.0, 
            interp_buffer: Vec::new(), 
            elapsed_time: 0, 
            duration, 
        }
    }

    /// Set read speed 
    /// 
    /// # Args
    /// -----
    /// 
    /// `value`: reading speed [0, n]
    /// 
    /// 
    pub fn set_read_speed(&mut self, value: f32) {
        self.read_speed = value
    }

    /// Set read offset 
    /// 
    /// # Args
    /// -----
    /// 
    /// `value`: offset in samples [0, audio length - 1]
    /// 
    /// 
    pub fn set_read_offset(&mut self, time: f32) {
        let phase = (time * self.sr).ceil();
        self.read_offset = phase % self.n_samples as f32
    }
    
    /// Set read again 
    /// 
    /// # Args
    /// -----
    /// 
    /// `value`: read loop. If true read again at the end
    /// 
    pub fn set_read_again(&mut self, value: bool) {
        self.read_again = value
    }

    /// Get audio duration
    /// 
    /// # Return
    /// -------
    /// 
    /// `f32` duration in sec.
    /// 
    pub fn get_duration(&self) -> f32 {
        self.duration
    }

    /// Procedural samples
    /// 
    /// # Args
    /// -----
    /// 
    /// `duration`: duration in sec  
    /// `interp`: interpolation mode  
    /// 
    /// # Return
    /// -------
    /// 
    /// `Result<f32, BufferError>`
    /// 
    /// 
    pub fn procedural_sampler(&mut self, interp: Interp) -> f32 {
        AudioBuffer::read_from_audio_object(self, interp).unwrap_or(0.0)
    }

    pub(crate) fn update_and_set_pmotion(&mut self, value: f32, table_length: f32) {
        update_and_reset_increment(&mut self.phase_motion, value, table_length);
    }
    
    pub(crate) fn update_pmotion(&mut self, value: f32) {
        update_increment(&mut self.phase_motion, value);
        self.phase_motion %= self.vector_signal.len() as f32
    }

    pub(crate) fn write_interp_buffer(&mut self, interp: Interp, sample: f32) {
        interp_buffer_write(&mut self.interp_buffer, interp, sample);
    }

}

impl Channels for AudioObject
{
    fn to_nchannels(&mut self, out_channels: usize) -> Result<(), ChannelError> {
        let prev_channels = self.n_channels;
        self.n_channels = out_channels;
        split_into_nchannels(&mut self.vector_signal, prev_channels, out_channels)
    }
}

impl<'a> WriteToFile<'a> for AudioObject
{
    fn to_file(&self, name: &'a str) -> Result<(), ToFileError> {
        write_to_file(name, &self.vector_signal, self.n_channels, self.sr)
    }
}

pub struct AudioBuffer
{
    sr: i32,
}

impl AudioBuffer
{
    /// Audio buffer
    /// 
    /// 
    /// # Args
    /// -----
    /// 
    /// `sr`: sample rate, must be equal to sample rate passed in `StreamParams`  
    /// 
    pub fn new(sr: i32) -> Self {
        Self { sr }
    }

    /// Open audio file and convert it into `AudioObject`
    /// 
    /// # Args
    /// -----
    /// 
    /// `path`: path to audio file  
    /// 
    /// # Result
    /// -------
    /// 
    /// ` Result<AudioObject, BufferError>`
    /// 
    pub fn to_audio_object(&self, path: &str) -> Result<AudioObject, BufferError> {
        let output = Command::new("ffprobe")
            .arg("-v")
            .arg("error")
            .arg("-select_streams")
            .arg("a:0")
            .arg("-show_entries")
            .arg("stream=channels")
            .arg("-of")
            .arg("default=noprint_wrappers=1:nokey=1")
            .arg(path)
            .output();
        
        let cn = match output {
            Ok(out) => {
                let num_chn = String::from_utf8_lossy(&out.stdout);
                let num_chn = num_chn.trim().parse::<i32>().unwrap_or(1);
                num_chn
            },
            Err(_) => return Err(BufferError::ErrorInReadingChannelNumbers) 
        };

        let mut com = Command::new("ffmpeg")
            .arg("-i")
            .arg(path)
            .arg("-f")
            .arg("f32le")
            .arg("-c:a")
            .arg("pcm_f32le")
            .arg("-ar")
            .arg(self.sr.to_string())
            .arg("-ac")
            .arg(cn.to_string())
            .arg("-")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

        let mut buffer = Vec::new();
        if let Some(out) = com.stdout.as_mut() {
            out.read_to_end(&mut buffer).unwrap();
        }

        let status = com.wait().unwrap();
        if !status.success() { return Err(BufferError::ErrorInReadingFile) }
        if buffer.is_empty() { return Err(BufferError::NullOpenFileBufferEmpty) }

        let n_samples = buffer.len() / std::mem::size_of::<f32>();
        let samples: Vec<f32> = unsafe {
            let slice_buffer = std::slice::from_raw_parts(buffer.as_ptr() as *const f32, n_samples);
            Vec::from(slice_buffer)
        };

        let mut audiobj = AudioObject::new(samples, cn as usize, self.sr as f32);
        Ok(audiobj)
    }

    /// Read audio file from `AudioObject` sample by sample
    /// 
    /// # Args
    /// -----
    /// 
    /// `audio_object`: `AudioObject`  
    /// `interp`: interpolation mode    
    /// 
    /// # Result
    /// -------
    /// 
    /// ` Result<f32, BufferError>`
    /// 
    fn read_from_audio_object(audio_object: &mut AudioObject, interp: Interp) -> Result<f32, BufferError> {
        if audio_object.read_offset >= audio_object.n_samples as f32 { return Err(BufferError::ReadOffsetGratherThanAudioLength) }
        let phase = audio_object.phase_motion + audio_object.read_offset;

        if audio_object.read_again { 
            audio_object.update_and_set_pmotion(audio_object.read_speed, audio_object.n_samples as f32 - audio_object.read_offset);
        } else {
            audio_object.update_pmotion(audio_object.read_speed);
            if phase >= audio_object.n_samples as f32 { return Err(BufferError::BufferLengthExceeded) }
        }

        let table_index = PhaseInterpolationIndex::new(phase);
        let index_int = table_index.int_part;
        let frac_part = table_index.frac_part;
        audio_object.write_interp_buffer(interp, audio_object.vector_signal[index_int]);
        let sample = interp.get_table_interpolation(frac_part, &audio_object.interp_buffer).unwrap();
        Ok(sample)
    }

    pub fn write_to_file<'a, T: WriteToFile<'a>>(name: &'a str, signal: &'a T) -> Result<(), ToFileError> {
        signal.to_file(name)
    }

}

#[derive(Debug)]
pub enum DelayBufferError
{
    DelayBufferIndexError,
    TapLengthMustBeLessThanBufferLength
}

#[derive(Debug)]
pub struct DelayBuffer
{
    pub delay_length: usize,
    pub(crate) dbuffer: Vec<f32>,
    pub(crate) read_index: usize,
    pub(crate) write_index: usize,
    pub(crate) tap_cache: Vec<usize>,
}

impl DelayBuffer 
{
    /// Create new delay buffer
    /// 
    /// # Args
    /// -----
    /// 
    /// `delay_length`: buffer length in samples  
    /// 
    /// 
    pub fn new(delay_length: usize) -> Self {
        Self {
            delay_length,
            dbuffer: vec![0.0; delay_length],
            read_index: 0,
            write_index: 0,
            tap_cache: Vec::new()
        }
    }

    /// Generate delayed feed-forward delayed sample
    /// 
    /// # Args
    /// -----
    /// 
    /// `sample`: sample in  
    /// 
    /// # Return
    /// --------
    /// 
    /// `f32`: delayed sample
    /// 
    pub fn feedforward_delayed_sample(&mut self, sample: f32) -> f32 {
        let sample_out = self.read_buffer() + self.read_internal_tap();
        self.write_buffer(sample);
        sample_out
    }

    /// Generate delayed feed-back delayed sample
    /// 
    /// # Args
    /// -----
    /// 
    /// `sample`: sample in  
    /// `g`: feed-back gain factor
    /// 
    /// # Return
    /// --------
    /// 
    /// `f32`: delayed sample
    /// 
    pub fn feedback_delayed_sample(&mut self, sample: f32, g: f32) -> f32 {
        let sample_out = g * self.read_buffer() + sample;
        self.write_buffer(sample_out);
        sample_out + self.read_internal_tap()
    }
    
    /// Generate internal tap sample by sample  
    /// This method must precede `feedforward_delayed_sample()` or `feedback_delayed_sample()` method.  
    /// Each tapped sample will be summed internally in a main delay line and putted out a single sample.
    /// 
    /// ```rust 
    /// let mut d = DelayBuffer::new(44100);
    /// 
    /// while true {
    ///     ...generate sample x
    ///     d.internal_tap(1200).unwrap();
    ///     d.internal_tap(7000).unwrap();
    ///     let delayed_sample = d.feedforward_delayed_sample(x);
    /// }
    /// ```
    /// 
    /// # Args
    /// -----
    /// 
    /// `length`: tap length in samples  
    /// 
    /// # Return
    /// --------
    /// 
    /// `Result<(), DelayBufferError>`
    /// 
    pub fn internal_tap(&mut self, length: usize) -> Result<(), DelayBufferError> {
        if length > self.delay_length { return Err(DelayBufferError::TapLengthMustBeLessThanBufferLength) }
        let tap_index = self.delay_length - length;
        if !self.tap_cache.contains(&tap_index) { self.tap_cache.push(tap_index) }
        Ok(())
    }

    /// Generate external tap sample by sample  
    /// This method must used in a block `read_buffer()` - `write_buffer()`.  
    /// Each tap line return an indipendent sample  
    /// 
    /// ```rust 
    /// let mut d = DelayBuffer::new(44100);
    ///
    /// while true {
    ///     ...generate sample x
    ///     let _ = d.read_buffer();
    ///     let tap1 = d.external_tap(1200).unwrap_or(0.0);
    ///     let tap2 = d.external_tap(7000).unwrap_or(0.0);
    ///     d.write_buffer(x)
    /// }
    /// 
    /// ```
    /// 
    /// # Args
    /// -----
    /// 
    /// `length`: tap length in samples  
    /// 
    /// # Return
    /// --------
    /// 
    /// `Result<(), DelayBufferError>`
    /// 
    pub fn external_tap(&mut self, length: usize) -> Result<f32, DelayBufferError> {
        if length > self.delay_length { return Err(DelayBufferError::TapLengthMustBeLessThanBufferLength) }
        let index = (self.delay_length - length) + self.read_index;
        Ok(self.dbuffer[index % self.delay_length])
    }

    /// Read delay buffer
    /// 
    pub fn read_buffer(&mut self) -> f32 {
        let sample = self.dbuffer[self.read_index];
        self.advance_read_index();
        sample
    }

    /// Write delay buffer
    ///
    /// # Args
    /// -----
    /// 
    /// `sample`: sample in
    /// 
    pub fn write_buffer(&mut self, sample: f32) {
        self.dbuffer[self.write_index] = sample;
        self.advance_write_index();
    }

    /// Reset delay buffer
    ///
    pub fn reset_buffer(&mut self) {
        self.dbuffer = vec![0.0; self.delay_length];
        self.read_index = 0;
        self.write_index = 0;
    }
    
    fn read_internal_tap(&self) -> f32 {
        let mut tap_sum = 0.0;
        if !self.tap_cache.is_empty() {
            for tap in self.tap_cache.iter() {
                tap_sum += self.dbuffer[(tap + self.read_index) % self.delay_length];
            }
        }
        tap_sum
    }

    fn advance_read_index(&mut self) {
        self.read_index += 1;
        self.read_index %= self.delay_length;
    }

    fn advance_write_index(&mut self) {
        self.write_index += 1;
        self.write_index %= self.delay_length;
    }

}