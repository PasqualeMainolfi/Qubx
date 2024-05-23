#![allow(
    dead_code,
    unused_mut,
    unused_variables,
    unused_import_braces,
    unused_imports
)]

use crate::qubx_common::{Process, ProcessState, StreamParameters};
use crate::qubx_components::{DspProcess, DuplexProcess, MasterStreamoutProcess, MonitorProcess};
use crate::qubx_pmanage::{QubxDspProcess, QubxDuplexProcess, QubxMasterProcess};
use portaudio as pa;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// # Qubx: queue-based multithread real time parallel audio streams processing and managment
///
///
#[derive(Debug)]
pub struct Qubx {
    pub verbose: bool,
    master_streamouts: HashMap<String, Arc<Mutex<MasterStreamoutProcess>>>,
    duplex_streams: Vec<Arc<Mutex<DuplexProcess>>>,
    pub processes_monitor_ptr: Arc<Mutex<MonitorProcess>>,
    run: Arc<AtomicBool>,
    dsp_latency_amount: Arc<Mutex<Duration>>,
    // dsp_latency_amount: Arc<Atomi>,
    count_dsp_iterations: Arc<Mutex<f32>>,
}

impl Qubx {
    /// Create Qubx data structure
    ///
    /// # Args
    /// ------
    ///
    /// `verbose`: print out the state and the latency of the active processes
    ///
    pub fn new(verbose: bool) -> Self {
        let master_streamouts: HashMap<String, Arc<Mutex<MasterStreamoutProcess>>> = HashMap::new();
        let duplex_streams: Vec<Arc<Mutex<DuplexProcess>>> = Vec::new();
        let processes_monitor: MonitorProcess = MonitorProcess::new(verbose);
        let processes_monitor_ptr = Arc::new(Mutex::new(processes_monitor));

        Self {
            verbose,
            master_streamouts,
            duplex_streams,
            processes_monitor_ptr,
            run: Arc::new(AtomicBool::new(true)),
            dsp_latency_amount: Arc::new(Mutex::new(Duration::new(0, 0))),
            count_dsp_iterations: Arc::new(Mutex::new(0.0)),
        }
    }

    /// # Get devices index and info
    ///
    ///
    pub fn get_devices_info() {
        let port_audio = pa::PortAudio::new().unwrap();
        let devices = port_audio.devices().unwrap();
        for device in devices {
            let d = &device.unwrap();
            print!("\n[{:?}]: {:?}\n", d.0, d.1);
        }
    }

    /// Create master streamout
    ///
    /// # Args
    /// ------
    ///
    /// `name`: master streamout name (id)
    /// `params`: stream params
    ///
    /// # Return
    /// --------
    ///
    /// `QubxMasterProcess`
    pub fn create_master_streamout(
        &mut self,
        name: String,
        params: StreamParameters,
    ) -> QubxMasterProcess {
        let master_process =
            MasterStreamoutProcess::new(name.clone(), params, Arc::clone(&self.run), self.verbose);
        let shared_master = Arc::new(Mutex::new(master_process));
        self.master_streamouts
            .insert(name.clone(), Arc::clone(&shared_master));
        QubxMasterProcess::new(
            Arc::clone(&self.processes_monitor_ptr),
            Arc::clone(&shared_master),
        )
    }

    /// Create duplex streamout
    ///
    /// # Args
    /// ------
    ///
    /// `params`: duplex stream params
    ///
    /// # Return
    /// --------
    ///
    /// `QubxDuplexProcess`
    pub fn create_duplex_dsp_process(&mut self, params: StreamParameters) -> QubxDuplexProcess {
        let duplex_process = DuplexProcess::new(params, Arc::clone(&self.run), self.verbose);
        let shared_duplex = Arc::new(Mutex::new(duplex_process));
        self.duplex_streams.push(Arc::clone(&shared_duplex));
        QubxDuplexProcess::new(
            Arc::clone(&self.processes_monitor_ptr),
            Arc::clone(&shared_duplex),
        )
    }

    /// Create dsp process
    ///
    /// # Args
    /// ------
    ///
    /// `master_streamout_name`: the name of the master streamout to associate with
    ///
    /// # Return
    /// --------
    ///
    /// `QubxDspProcess`
    pub fn create_parallel_dsp_process(&self, master_streamout_name: String, use_parallel: bool) -> QubxDspProcess {

    	if use_parallel {
     		println!("[INFO] Parallel computation activated on DspProcess:::[{}]", master_streamout_name.clone());
     	}

        let master_ptr = self.master_streamouts.get(&master_streamout_name).unwrap();
        let dsp_process = DspProcess::new(
            Arc::clone(&self.processes_monitor_ptr),
            Arc::clone(master_ptr),
            Arc::new(AtomicBool::new(self.verbose)),
            Arc::clone(&self.dsp_latency_amount),
            Arc::clone(&self.count_dsp_iterations),
            use_parallel
        );
        QubxDspProcess::new(
            Arc::clone(&self.processes_monitor_ptr),
            Arc::new(Mutex::new(dsp_process)),
        )
    }

    /// Starts monitoring active processes
    ///
    pub fn start_monitoring_active_processes(&mut self) {
        let monitor_clone = Arc::clone(&self.processes_monitor_ptr);
        let local_run = Arc::clone(&self.run);

        let t = thread::spawn(move || {
            while local_run.load(Ordering::Acquire) {
                let mut m = monitor_clone.lock().unwrap();
                m.remove_inactive_processes();
                drop(m);
            }
        });

        println!("[INFO] Start monitoring process...");
        let mclone = Arc::clone(&self.processes_monitor_ptr);
        let mut pm = mclone.lock().unwrap();
        pm.add_process(Process::new(
            t,
            String::from("MONITOR ACTIVE PROCESSES"),
            ProcessState::On,
        ));
    }

    /// Terminate all processes still active and close Qubx
    ///
    pub fn close_qubx(&mut self) {
        println!("[INFO] Closing QUBX System...");
        self.run.store(false, Ordering::Release);

        let count = self.count_dsp_iterations.lock().unwrap();
        let lat_amount = self.dsp_latency_amount.lock().unwrap();
        let fac = if *count > 0.0 { *count } else { 1.0 };
        let lat_amount = lat_amount.as_secs_f32() / fac;
        print!(
            "\n[PROCESSES INFO]\n:::Process Name: \"DSP\"\n:::Number of started processes: {}\n:::Latency average: {:?}\n\n",
            *count as i32,
            std::time::Duration::from_secs_f32(lat_amount),
        );

        thread::sleep(std::time::Duration::from_secs(1));
        println!("[PROCESS INFO] Terminating last active processes...");
        let pclone = Arc::clone(&self.processes_monitor_ptr);
        let mut p = pclone.lock().unwrap();
        p.join_and_remove_all();

        thread::sleep(std::time::Duration::from_secs_f32(0.5));
        println!("[INFO] Done!");
    }
}
