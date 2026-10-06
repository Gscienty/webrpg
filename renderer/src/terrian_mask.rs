pub struct TerrainMask {
    pub width: u32,
    pub height: u32,
    pub origin: shared::math3::Vector2,
    pub cell_size: f32,
    pub cells: Vec<u8>,
    pub generation: u32,
}
