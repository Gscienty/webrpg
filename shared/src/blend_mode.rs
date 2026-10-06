#[derive(Clone, Copy, Debug)]
pub enum BlendMode {
    None,
    AlphaKey,
    Blend,
    Add,
    AddAlpha,
    Modulate,
    Modulate2X,
}
