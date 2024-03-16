#![allow(unused_variables, dead_code)]

use crate::qlist::QList;
use crate::qubx_common::{ StreamParameters, Process, ProcessState };
use portaudio as pa;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle, ThreadId};
use std::collections::HashMap;

/// # Master Stream-out
/// 
/// 
#[derive(Debug)]
pub struct MasterStreamoutProcess {

    pub name: String,
    pub params: Arc<StreamParameters>,
    pub qlist: Arc<Mutex<QList>>,
    pub verbose: bool,
    pub run: Arc<AtomicBool>

}

impl MasterStreamoutProcess {

    pub fn new(name: String, params: StreamParameters, run: Arc<AtomicBool>, verbose: bool) -> Self {
        let qlist = QList::default();

        Self {

            name,
            qlist: Arc::new(Mutex::new(qlist)),
            params: Arc::new(params),
            verbose,
            run

        }
    }

    /// Starting master streamout process
    /// 
    /// # Args
    /// ------
    /// 
    /// `dsp_function`: a closure that processes the summation of audio streams from all processes associated with the stream output. Take one arg 
    /// `frame`: `&mut [f32]` (frame to be processed)  
    /// 
    /// Example:
    /// 
    /// ```rust
    /// let mut master_out = q.create_master_streamout(String::from("M1"), stream_params);
    /// master_out.start(move |frame| {
    ///    for sample in frame.iter_mut() {
    ///        *sample *= 0.7;
    ///    }
    /// });
    /// ```
    /// 
    /// # Return
    /// --------
    /// 
    /// `JoinHandle<()>`
    /// 
    pub fn start<F>(&self, mut dsp_function: F) -> JoinHandle<()> 
    where
        F: for<'a> FnMut(&'a mut [f32]) + Send + Sync + 'static
    {

        let qlist_clone = Arc::clone(&self.qlist);
        let params_clone = Arc::clone(&self.params);
        let sr = params_clone.sr as f64;
        
        let p = pa::PortAudio::new().unwrap();
        
        let device = match params_clone.outdevice {
            Some(devout) => pa::DeviceIndex(devout),
            None => { p.default_output_device().unwrap() }
        };

        println!("[DEBUG] device {:?}", device);
        
        let channels = params_clone.outchannels;
        
        let chunk = params_clone.chunk;

        let run = Arc::clone(&self.run);
        
        thread::spawn(move || {

            let callback = move |pa::OutputStreamCallbackArgs { buffer, .. }| {
    
                let mut q = qlist_clone.lock().unwrap();
                let mut block = vec![0.0; buffer.len()];
    
                for i in 0..q.length as usize {
                    if !q.is_empty_at_index(i) {
                        let frame = q.get_frame(i);
                        for (b, f) in block.iter_mut().zip(frame.iter()) {
                            *b += f;
                        }
                    }
                }
    
                drop(q);
                
                // APPLY DSP TO MULTICHANNEL AUDIO OUT -> ON BUFFER VECTOR OR PASS DSP FUNCTION
                // .
                
                dsp_function(&mut block);
                
                // .
                
                for (i, sample )in buffer.iter_mut().enumerate() {
                    *sample = block[i];
                }
                
                pa::Continue
    
            };
    
            let p = pa::PortAudio::new().unwrap();
    
            let device_info = p.device_info(device).unwrap();
            let latency = device_info.default_low_output_latency;
    
            let output_params = pa::StreamParameters::<f32>::new(device, channels as i32, true, latency);
            let output_settings = pa::OutputStreamSettings::new(output_params, sr, chunk);
            let mut stream = p.open_non_blocking_stream(output_settings, callback).unwrap();
    
            stream.start().unwrap();
    
            while run.load(Ordering::Acquire) { continue }
    
            println!("[INFO] Closing stream-out PortAudio...");
            stream.stop().unwrap();
            stream.close().unwrap();
            p.terminate().unwrap();

        })

    }

}

/// # Dsp Duplex Stream
/// 
/// 
#[derive(Debug)]
pub struct DuplexProcess {

    params: Arc<StreamParameters>,
    verbose: bool,
    run: Arc<AtomicBool>
}

impl DuplexProcess {

    pub fn new(params: StreamParameters, run: Arc<AtomicBool>, verbose: bool) -> Self {

        Self {

            params: Arc::new(params),
            verbose,
            run
        
        }
    }

    /// Starting duplex dsp stream
    /// 
    /// # Args
    /// ------
    /// 
    /// `dsp_function`: a closure that processes the audio streams. Take one arg frame: `&[f32]` (frame to be processed) and must be return
    /// a `Vec<f32>` (frame to output)  
    /// 
    /// Example: 
    /// ```rust
    /// let mut duplex = q.create_duplex_dsp_process(stream_params);
    /// duplex.start(|frame| { frame.to_vec() });
    /// ```
    /// 
    /// # Return
    /// --------
    /// 
    /// `JoinHandle<()>`
    /// 
    pub fn start<F>(&self, mut dsp_function: F) -> JoinHandle<()> 
    where 
        F: for<'a> FnMut(&'a [f32]) -> Vec<f32> + Send + Sync + 'static
    {

        let params_clone = Arc::clone(&self.params);
        let sr = params_clone.sr as f64;
        
        let p = pa::PortAudio::new().unwrap();
        
        let indevice = match params_clone.indevice {
            Some(dev) => pa::DeviceIndex(dev),
            None => { p.default_input_device().unwrap() }
        };
        
        let inchannels = params_clone.inchannels;
        
        let outdevice = match params_clone.outdevice {
            Some(dev) => pa::DeviceIndex(dev),
            None => { p.default_output_device().unwrap() }
        };
        
        let outchannels = params_clone.outchannels;
        
        let chunk = params_clone.chunk;
        let run = Arc::clone(&self.run);

        thread::spawn(move || {
            
            let callback = move |pa::DuplexStreamCallbackArgs { in_buffer, out_buffer, ..} | {
                
                let mut inblock = vec![0.0; (chunk * inchannels) as usize];
    
                for (insample, outsample) in in_buffer.iter().zip(inblock.iter_mut()) {
                    *outsample = *insample;
                }

                // ATTENTION: is interleaved format! length of inblock is chunk * chnls

                let dsp_inblock = dsp_function(&inblock);

                assert_eq!(dsp_inblock.len(), out_buffer.len(), "[ERROR] The frame returned by the closure must have the same number of channels as the out frame!");
                
                for (i, sample) in dsp_inblock.iter().enumerate() {
                    out_buffer[i] = *sample
                }
                
                pa::Continue
    
            };
    
            let devin_info = p.device_info(indevice).unwrap();
            let inlatency = devin_info.default_low_input_latency;
    
            let inparams = pa::StreamParameters::<f32>::new(indevice, inchannels as i32, true, inlatency);
            
            let devout_info = p.device_info(outdevice).unwrap();
            let outlatency = devout_info.default_low_output_latency;
    
            let outparams = pa::StreamParameters::<f32>::new(outdevice, outchannels as i32, true, outlatency);
    
            let stream_settings = pa::DuplexStreamSettings::new(inparams, outparams, sr, chunk);
    
            let mut stream = p.open_non_blocking_stream(stream_settings, callback).unwrap();
    
            stream.start().unwrap();
    
            while run.load(Ordering::Acquire) { continue }
    
            println!("[INFO] Closing PortaAudio duplex stream...");
            stream.stop().unwrap();
            stream.close().unwrap();
            p.terminate().unwrap();

        })


    }

}

/// # Dsp Process
/// 
/// 
pub struct DspProcess {

    monitor_processes: Arc<Mutex<MonitorProcess>>,
    master_streamout: Arc<Mutex<MasterStreamoutProcess>>,
    verbose: Arc<AtomicBool>

}

impl DspProcess {

    pub fn new(monitor_processes: Arc<Mutex<MonitorProcess>>, master_streamout: Arc<Mutex<MasterStreamoutProcess>>, verbose: bool) -> Self {
        Self { monitor_processes, master_streamout, verbose: Arc::new(AtomicBool::new(verbose)) }
    }
    
    /// Starting dsp process
    /// 
    /// # Args
    /// ------
    /// 
    /// `audio_data`: audio data to be processed   
    /// `dsp_function`: a closure that processes the audio streams. Take one arg frame `&mut [f32]`    
    /// 
    /// Example:
    /// ```rust
    /// let mut dsp_process = q.create_parallel_dsp_process(String::from("M1"));
    /// 
    /// dsp_process.start(audio_data, move |frame| {
    /// for sample in frame.iter_mut() {
    ///     *sample *= 0.7;
    /// }
    /// });
    /// 
    /// ```
    /// 
    /// # Return
    /// --------
    /// 
    /// `JoinHandle<()>`
    /// 
    pub fn start<F>(&self, audio_data: Vec<f32>, mut dsp_function: F) -> JoinHandle<()> 
    where
        F: for<'a> FnMut(&'a mut [f32]) + Send + Sync + 'static
    {

        let pclone = Arc::clone(&self.monitor_processes);
        let mclone = Arc::clone(&self.master_streamout);
        let verbose = Arc::clone(&self.verbose);

        thread::spawn(move || {

            let start = std::time::Instant::now();

            let m = mclone.lock().unwrap();
            let chunk_size = (m.params.chunk * m.params.outchannels) as usize; // Frame length must be chunk size * nchnls out -> streamout 

            drop(m);
            
            let mut frames: Vec<Vec<f32>> = Vec::new();
            
            for i in (0..audio_data.len()).step_by(chunk_size) {
                let start = i;
                let end = std::cmp::min(i + chunk_size, audio_data.len());
                
                let mut frame_padded = vec![0.0; chunk_size];
                let size = end - start;
                
                frame_padded[0..size].copy_from_slice(&audio_data[start..end]);
                
                // APPLY DSP TO SINGLE AUDIO STREAM OR PASS DSP FUNCTION
                // .
                
                dsp_function(&mut frame_padded);
                
                // .
                
                frames.push(frame_padded);
            }
            
            let m = mclone.lock().unwrap();
            let qclone = Arc::clone(&m.qlist);
            let mut q = qclone.lock().unwrap();
            for frame in frames.iter() {
                q.put_frame(frame.clone());
            }

            q.get_next_empty_queue();

            drop(q);
            drop(m);

            let end = start.elapsed();

            let id = thread::current().id();

            if verbose.load(Ordering::Acquire) { 
                println!("[PROCESS INFO] Thread:::[Name: \"DSP\"]:::[ID: {:?}]:::[LATENCY: {:?}]", thread::current().id(), end) 
            }
            
            let mut pm = pclone.lock().unwrap();
            if let Some(p) = pm.processes.get_mut(&id) { p.state = ProcessState::Off };
            drop(pm);

        })

    }

}

/// # Monitoring active processes
#[derive(Debug)]
pub struct MonitorProcess {
    pub processes: HashMap<ThreadId, Process>,
    verbose: Arc<AtomicBool>
}

impl MonitorProcess {
    pub fn new(verbose: bool) -> Self {
        let processes: HashMap<ThreadId, Process> = HashMap::new();
        Self { processes, verbose: Arc::new(AtomicBool::new(verbose)) }
    }

    pub fn add_process(&mut self, process: Process) {

        if Arc::clone(&self.verbose).load(Ordering::Acquire) {
            println!("[PROCESS INFO] Thread:::[Name: {:?}]:::[ID: {:?}]:::[State: {:?}]", process.name, process.handle.thread().id(), process.state)
        }

        self.processes.insert(process.handle.thread().id(), process);
    }

    pub fn remove_inactive_processes(&mut self) {
        let mut inactive_processes: Vec<ThreadId> = Vec::new();
        for (id, process) in self.processes.iter() {
            if process.state == ProcessState::Off {
                inactive_processes.push(*id);
            }
        }

        for id in inactive_processes {
            if let Some(p) = self.processes.remove(&id) {

                if Arc::clone(&self.verbose).load(Ordering::Acquire) {
                    println!("[PROCESS INFO] Thread:::[Name: {:?}]:::[ID: {:?}]:::[State: {:?}]", p.name, p.handle.thread().id(), p.state)
                }

                p.handle.join().unwrap()
            }
        }

    }

    pub fn join_and_remove_all(&mut self) {
        let mut to_remove: Vec<ThreadId> = Vec::new();
        for (id, process) in self.processes.iter_mut() {
            process.state = ProcessState::Off;
            to_remove.push(*id);
        }

        for key in to_remove {
            if let Some(process) = self.processes.remove(&key) {

                if Arc::clone(&self.verbose).load(Ordering::Acquire) {
                    println!("[PROCESS INFO] Thread:::[Name: {:?}]:::[ID: {:?}]:::[State: {:?}]", process.name, process.handle.thread().id(), process.state)
                }

                process.handle.join().unwrap();
            }
        }
        self.processes.clear();
    }
}
