
#[derive(Debug, Clone)]
pub struct TwoPoleCoeffs {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a0: f32,
    pub a1: f32,
    pub a2: f32
}

impl Default for TwoPoleCoeffs
{
    fn default() -> Self {
        Self { b0: 0.0, b1: 0.0, b2: 0.0, a0: 0.0, a1: 0.0, a2: 0.0 }
    }
}

impl TwoPoleCoeffs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_coeffs(&mut self, coeffs: (f32, f32, f32, f32, f32, f32)) {
        self.b0 = coeffs.0;
        self.b1 = coeffs.1;
        self.b2 = coeffs.2;
        self.a0 = coeffs.3;
        self.a1 = coeffs.4;
        self.a2 = coeffs.5;
    }
}

#[derive(Debug, Clone)]
pub struct OnePoleCoeffs {
    pub b0: f32,
    pub b1: f32,
    pub a1: f32
}

impl Default for OnePoleCoeffs
{
    fn default() -> Self {
        Self { b0: 0.0, b1: 0.0, a1: 0.0 }
    }
}

impl OnePoleCoeffs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_coeffs(&mut self, coeffs: (f32, f32, f32)) {
        self.b0 = coeffs.0;
        self.b1 = coeffs.1;
        self.a1 = coeffs.2;
    }
}

pub struct ButterCoeffs
{
    pub lp: OnePoleCoeffs,
    pub hp: OnePoleCoeffs 
}

impl Default for ButterCoeffs
{
    fn default() -> Self {
        Self { lp: OnePoleCoeffs::new(), hp: OnePoleCoeffs::new() }
    }
}

impl ButterCoeffs
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_coeffs(&mut self, c_lp: (f32, f32, f32), c_hp: (f32, f32, f32)) {
        self.lp.b0 = c_lp.0;
        self.lp.b1 = c_lp.1;
        self.lp.a1 = c_lp.2;
        self.hp.b0 = c_hp.0;
        self.hp.b1 = c_hp.1;
        self.hp.a1 = c_hp.2;
    }    
}