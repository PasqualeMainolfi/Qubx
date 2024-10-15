#![allow(unused)]




pub struct Sine 
{
    pub freq: f64,
    pub amp: f64,
    pub phase: Option<f64>,
    table: Vec<f64>
}

impl Sine
{
    pub fn generate(&mut self, freq: f64, amp: f64, phase: Option<f64>, table_length: Option<u64>) {
        todo!()
    }
}