
pub type MasterCType = Box<dyn FnMut(&mut [f32]) + Send + Sync>;
pub type DuplexCType = Box<dyn FnMut(&[f32]) -> Vec<f32> + Send + Sync>;
pub type DspCAType = Box<dyn Fn(&[f32]) -> Vec<f32> + Send + Sync>;
pub type DspCNAType = Box<dyn Fn() -> Vec<f32> + Send + Sync>;