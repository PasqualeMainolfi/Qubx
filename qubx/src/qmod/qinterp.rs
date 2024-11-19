#[derive(Debug)]
pub enum InterpError
{
    BufferEmpty,
    BufferOverSize
}

pub struct PhaseInterpolationIndex
{
    pub int_part: usize,
    pub frac_part: f32
}

impl PhaseInterpolationIndex
{
    pub fn new(index: f32) -> Self {
        let ip = if index < 0.0 { 0.0 } else { index };
        let frac_part = ip.fract();
        let int_part = ip as usize;
        Self { int_part, frac_part }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Interp
{
    NoInterp,
    Linear,
    Cosine,
    Cubic,
    Hermite
}

impl Interp {
    /// Make interpolation
    /// 
    /// # Args
    /// ------
    /// 
    /// `mu`: interpolation parameter [0, 1]. Relative position between points (t)   
    /// `buffer`: previous samples. For Linear and Cosine must be length of 2 
    /// for Cubic and Hermite must be length of 4; for NoInterp must be 1.
    /// 
    /// # Return
    /// --------
    /// 
    /// `Result<f32, InterpError>`
    /// 
    pub fn get_table_interpolation(&self, mu: f32, buffer: &[f32]) -> Result<f32, InterpError> {
        match buffer.len() {
            0 => {
                Err(InterpError::BufferEmpty)
            }
            1 => Ok(buffer[0]),
            2 => {
                match self {
                    Interp::Linear => Ok((1.0 - mu) * buffer[0] + mu * buffer[1]),
                    Interp::Cosine => {
                        let mu2 = (1.0 - (mu * std::f32::consts::PI).cos()) / 2.0;
                        Ok(buffer[0] * (1.0 - mu2) + mu2 * buffer[1])
                    },
                    _ => Ok(0.0)
                }
            },
            4 => {
                match self {
                    Interp::Cubic => {
                        let y0 = buffer[0];
                        let y1 = buffer[1];
                        let y2 = buffer[2];
                        let y3 = buffer[3];
                        let a0 = y3 - y2 - y0 + y1;
                        let a1 = y0 - y1 - a0;
                        let a2 = y2 - y0;
                        let a3 = y1;
                        Ok(a0 * mu.powi(3) + a1 * mu.powi(2) + a2 * mu + a3)
                    },
                    Interp::Hermite => {
                        let y0 = buffer[0];
                        let y1 = buffer[1];
                        let y2 = buffer[2];
                        let y3 = buffer[3];
                        let diff = y1 - y2;
                        let a1 = y2 - y0;
                        let a3 = y3 - y0 + 3.0 * diff;
                        let a2 = -(2.0 * diff + a1 + a3);
                        Ok(0.5 * ((a3 * mu + a2) * mu + a1) * mu + y1)
                    },
                    _ => Ok(0.0)
                }
            },
            _ => {
                Err(InterpError::BufferOverSize)
            }
        }
    }

}
