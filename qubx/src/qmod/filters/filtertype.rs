// TODO: Chebisev

#[derive(Debug, Clone, Copy)]
pub enum BiquadFilter
{
    Lp,
    Hp,
    Bp0dB,
    Bpsg,
    Notch,
    Ap,
    Peq,
    LpShelf,
    HpShelf
}

#[derive(Debug, Clone, Copy)]
pub enum TwoZeroTwoPoleFilter 
{
    Notch,
    Bp,
}

#[derive(Debug, Clone, Copy)]
pub enum HarmonicFilter 
{
    CombFIR,
    CombFreeverbFIR,
    CombIIR,
    LPFBCombFilter,
    Allpass,
    AllpassFreeverb,
    LPFBAllpassFilter
}

#[derive(Debug, Clone, Copy)]
pub enum OnePoleFilter 
{
    LowPass,
    HighPass
}

// #[derive(Debug, Clone, Copy)]
// pub enum DcBlockFilter
// {
//     DcBlockJulius
// }

#[derive(Debug, Clone, Copy)]
pub enum NarrowFilter 
{
    Bp,
    Notch
}

#[derive(Debug, Clone, Copy)]
pub enum ZavalishinFilter
{
    OnePoleZeroDelay,
    NaiveOnePole,
    TrapIntOnePole,
    StateVariable
}

#[derive(Debug, Clone, Copy)]
pub enum ButterFilter
{
    Lp,
    Hp,
    Bp,
    Notch
}

#[derive(Debug)]
pub enum FilterError
{
    FilterModeNotAllowed,
    FilterGenericError,
    BandWidthNotDefined,
    FcSpreadNotSpecified,
    FilterCoeffsErrorNotCompatibleMode
}

pub struct SvfSamples {
    pub lp: f32,
    pub hp: f32,
    pub ap: f32,
    pub bp: f32,
    pub br: f32,
}

impl Default for SvfSamples
{
    fn default() -> Self {
        Self { lp: 0.0, hp: 0.0, ap: 0.0, bp: 0.0, br: 0.0 }
    }
}

impl SvfSamples {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_samples(&mut self, coeffs: (f32, f32, f32, f32, f32)) {
        self.lp = coeffs.0;
        self.hp = coeffs.1;
        self.ap = coeffs.2;
        self.bp = coeffs.3;
        self.br = coeffs.4;
    }
}