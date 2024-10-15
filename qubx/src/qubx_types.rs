
pub type MasterClosureType = Box<dyn FnMut(&mut [f32]) + Send + Sync>;
pub type DuplexClosureType = Box<dyn FnMut(&[f32]) -> Vec<f32> + Send + Sync>;
pub type DspClosureNoArgsType = Box<dyn Fn() -> Vec<f32> + Send + Sync>;
pub type DspClosureWithArgsType = Box<dyn Fn(&[f32]) -> Vec<f32> + Send + Sync>;