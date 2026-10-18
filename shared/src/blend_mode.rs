#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum BlendMode {
    None,
    AlphaKey,
    Blend,
    Add,
    AddAlpha,
    Modulate,
    Modulate2X,
}
