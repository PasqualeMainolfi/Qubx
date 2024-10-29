use crate::qubx_common::{ Process, ProcessState, DspProcessArg, ProcessArg };
use crate::qubx_components::{ DspProcess, DuplexProcess, MasterStreamoutProcess, MonitorProcess };
use std::sync::{ Arc, Mutex };

pub struct QubxMasterProcess {
    processes_monitor: Arc<Mutex<MonitorProcess>>,
    process: Arc<Mutex<MasterStreamoutProcess>>,
}

impl QubxMasterProcess {
    pub fn new(
        processes_monitor: Arc<Mutex<MonitorProcess>>,
        process: Arc<Mutex<MasterStreamoutProcess>>,
    ) -> Self {
        Self {
            processes_monitor,
            process,
        }
    }

    pub fn start<F>(&self, arg: ProcessArg<F>)
    where
        F: for<'a> FnMut(&'a mut [f32]) + Send + Sync + 'static,
    {
        let pclone = Arc::clone(&self.process);
        let p = pclone.lock().unwrap();
        println!("[PROCESS INFO] Starting {} stream-out...", p.name);
        let t = p.start(arg);
        let pmonitor = Arc::clone(&self.processes_monitor);
        let mut pm = pmonitor.lock().unwrap();
        pm.add_process(Process::new(t, p.name.clone(), ProcessState::On));

        drop(p);
        drop(pm);
    }
}

pub struct QubxDuplexProcess {
    processes_monitor: Arc<Mutex<MonitorProcess>>,
    process: Arc<Mutex<DuplexProcess>>,
}

impl QubxDuplexProcess {
    pub fn new(
        processes_monitor: Arc<Mutex<MonitorProcess>>,
        process: Arc<Mutex<DuplexProcess>>,
    ) -> Self {
        Self {
            processes_monitor,
            process,
        }
    }

    pub fn start<F>(&self, arg: ProcessArg<F>)
    where
        F: for<'a> FnMut(&'a [f32]) -> Vec<f32> + Send + Sync + 'static,
    {
        let pclone = Arc::clone(&self.process);
        let p = pclone.lock().unwrap();
        println!("[PROCESS INFO] Starting stream-duplex...");
        let t = p.start(arg);
        let pmonitor = Arc::clone(&self.processes_monitor);
        let mut pm = pmonitor.lock().unwrap();
        pm.add_process(Process::new(
            t,
            String::from("DUPLEX STREAM OUT"),
            ProcessState::On,
        ));

        drop(p);
        drop(pm);
    }
}
pub struct QubxDspProcess {
    processes_monitor: Arc<Mutex<MonitorProcess>>,
    process: Arc<Mutex<DspProcess>>,
}

impl QubxDspProcess {
    pub fn new(
        processes_monitor: Arc<Mutex<MonitorProcess>>,
        process: Arc<Mutex<DspProcess>>,
    ) -> Self {
        Self {
            processes_monitor,
            process,
        }
    }

    // pub fn start<F>(&self, audio_data: Vec<f32>, dsp_function: Option<F>)
    pub fn start<F1, F2>(&self, args: DspProcessArg<F1, F2>)
    where
        F1: Fn() -> Vec<f32> + Send + Sync + 'static,
        F2: for<'a> Fn(&'a [f32]) -> Vec<f32> + Send + Sync + 'static,
    {
        let pclone = Arc::clone(&self.process);
        let p = pclone.lock().unwrap();
        let t = p.start(args);
        let pmonitor = Arc::clone(&self.processes_monitor);
        let mut pm = pmonitor.lock().unwrap();
        pm.add_process(Process::new(t, String::from("DSP"), ProcessState::On));

        drop(p);
        drop(pm);
    }
}
