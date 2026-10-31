#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum BlendMode {
    #[default]
    None,
    AlphaKey,
    Blend,
    Add,
    AddAlpha,
    Modulate,
    Modulate2X,
}
