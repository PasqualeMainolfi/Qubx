
pub type MasterPatchType = Box<dyn FnMut(&mut [f32]) + Send + Sync>;
pub type DuplexPatchType = Box<dyn FnMut(&[f32]) -> Vec<f32> + Send + Sync>;
pub type DspHybridType = Box<dyn Fn(&[f32]) -> Vec<f32> + Send + Sync>;
pub type DspPatchType = Box<dyn Fn() -> Vec<f32> + Send + Sync>;