const TWOPI: f32 = std::f32::consts::PI * 2.0;
const ALPHA: f32 = 0.16;
const HAMMING_COEFF: f32 = 25.0 / 46.0;
const HANNING_COEFF: f32 = 0.5;

#[derive(Debug)]
pub enum WindowError
{
    WindowLengthExceeded
}

#[derive(Debug)]
pub enum QWindow
{
    Rect,
    Hamming,
    Hanning,
    Blackman
}

impl QWindow
{
    pub fn get_window(&self, length: usize) -> Vec<f32> {
        let mut win = vec![0.0; length];
        for (i, value) in win.iter_mut().enumerate() {
            let coeff = (TWOPI * i as f32 / length as f32).cos();
            *value = match self {
                Self::Rect => 1.0,
                Self::Hamming => HAMMING_COEFF * (1.0 - coeff),
                Self::Hanning => HANNING_COEFF * (1.0 - coeff),
                Self::Blackman => {
                    let a0 = (1.0 - ALPHA) / 2.0;
                    let a1 = 1.0 / 2.0;
                    let a2 = ALPHA / 2.0;
                    let sec_coeff = (2.0 * TWOPI * i as f32 / length as f32).cos();
                    a0 - a1 * coeff + a2 * sec_coeff
                }
            }
        }
        win
    }
}

#[derive(Debug, Default)]
pub struct ProceduralWindow
{
    index: usize,
    last_sample: f32
}

impl ProceduralWindow
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_window(&mut self, mode: QWindow, length: usize, length_exceeded: bool) -> Result<f32, WindowError> {
        let coeff = (TWOPI * self.index as f32 / length as f32).cos();
        if self.index < length {
            let sample = match mode {
                QWindow::Rect => 1.0,
                QWindow::Hamming => HAMMING_COEFF * (1.0 - coeff),
                QWindow::Hanning => HANNING_COEFF * (1.0 - coeff),
                QWindow::Blackman => {
                    let a0 = (1.0 - ALPHA) / 2.0;
                    let a1 = 1.0 / 2.0;
                    let a2 = ALPHA / 2.0;
                    let sec_coeff = (2.0 * TWOPI * self.index as f32 / length as f32).cos();
                    a0 - a1 * coeff + a2 * sec_coeff
                }
            };
            self.last_sample = sample;
            self.index += 1;
            Ok(sample)
        } else {
            if length_exceeded { return Err(WindowError::WindowLengthExceeded) }
            Ok(self.last_sample)
        }
    }

    pub fn reset_index(&mut self) {
        self.index = 0
    }
}