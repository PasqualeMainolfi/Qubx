#![allow(unused)]

use std::default;
use std::io::{Read, Write};
use std::process::{ Command, Stdio };
use portaudio::stream::Buffer;
use std::path::Path;
use std::fs;

use crate::check_list::ensure_ffmpeg;
use crate::qubx_common::{ Channels, ChannelError };
use super::{
    qsignals::SignalObject,
    qoperations::split_into_nchannels,
    shared_tools::{ update_increment, update_and_reset_increment, interp_buffer_write },
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
    pub(crate) elapsed_time: usize
}

impl AudioObject
{
    pub fn new(vector_signal: Vec<f32>, n_channels: usize, sr: f32) -> Self {
        let n_samples = vector_signal.len() / n_channels;
        Self { vector_signal, n_channels, sr, read_speed: 1.0, read_offset: 0.0, read_again: false, n_samples, phase_motion: 0.0, interp_buffer: Vec::new(), elapsed_time: 0 }
    }

    pub fn set_read_speed(&mut self, value: f32) {
        self.read_speed = value
    }

    pub fn set_read_offset(&mut self, time: f32) {
        let phase = (time * self.sr).ceil();
        self.read_offset = phase % self.n_samples as f32
    }
    
    pub fn set_read_again(&mut self, value: bool) {
        self.read_again = value
    }

    pub fn procedural_sampler(&mut self, duration: f32, interp: Interp) -> Result<f32, BufferError> {
        let sample = if (self.elapsed_time as f32) < self.sr * duration { 
            AudioBuffer::read_from_audio_object(self, interp).unwrap()
        } else { 
            return Err(BufferError::SamplerDataDurationReached) 
        };
        self.elapsed_time += 1;
        Ok(sample)
    }

    pub(crate) fn update_and_set_pmotion(&mut self, value: f32, table_length: f32) {
        update_and_reset_increment(&mut self.phase_motion, value, table_length);
    }
    
    pub(crate) fn update_pmotion(&mut self, value: f32) {
        update_increment(&mut self.phase_motion, value);
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

pub struct AudioBuffer
{
    sr: i32,
}

impl AudioBuffer
{
    pub fn new(sr: i32) -> Self {
        ensure_ffmpeg();
        Self { sr }
    }

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

    pub fn write_to_file(file_name: &str, audio_object: &AudioObject) -> Result<(), BufferError> {
        if audio_object.vector_signal.is_empty() { return Err(BufferError::NullOpenFileBufferEmpty) }
        let mut name: String = file_name.split(".").collect::<Vec<&str>>().join("").to_string();
        name.push_str(".wav");

        if Path::new(&name).exists() {
            println!("[INFO] File {} exists, removing and rewriting...", &name); 
            fs::remove_file(&name).unwrap() 
        }

        let mut com = Command::new("ffmpeg")
            .arg("-f")
            .arg("f32le")
            .arg("-c:a")
            .arg("pcm_f32le")
            .arg("-ac")
            .arg(audio_object.n_channels.to_string())
            .arg("-ar")
            .arg(audio_object.sr.to_string())
            .arg("-i")
            .arg("pipe:0")
            .arg(&name)
            .stdin(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        
        if let Some(stdin) = com.stdin.as_mut() {
            for sample in audio_object.vector_signal.iter() {
                stdin.write_all(&sample.to_le_bytes()).unwrap();
            }
        }

        let status = com.wait().unwrap();
        if !status.success() { return Err(BufferError::ErrorInWritingFile) }
        println!("[INFO] File {} saved!", &name);
        Ok(())
    }
}

