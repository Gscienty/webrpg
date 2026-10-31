pub struct Vertex {
    pub accurate_height: u16,
    pub water_level: u16,
    pub map_edge: u8,
    pub ground: u8,
    pub ramp: u8,
    pub blight: u8,
    pub water: u8,
    pub boundary: u8,
    pub ground_variation: u8,
    pub cliff_variation: u8,
    pub level: u8,
    pub cliff: u8,
}

impl From<[u8; 7]> for Vertex {
    fn from(bytes: [u8; 7]) -> Self {
        let water_and_edge = u16::from_le_bytes([bytes[2], bytes[3]]);
        let flags = bytes[4];
        let variation = bytes[5];
        let cliff_and_layer = bytes[6];

        Self {
            accurate_height: u16::from_le_bytes([bytes[0], bytes[1]]),
            water_level: water_and_edge & 0x3fff,
            map_edge: u8::from(water_and_edge & 0x4000 != 0),
            ground: flags & 0x0f,
            ramp: u8::from(flags & 0x10 != 0),
            blight: u8::from(flags & 0x20 != 0),
            water: u8::from(flags & 0x40 != 0),
            boundary: u8::from(flags & 0x80 != 0),
            ground_variation: variation & 0x1f,
            cliff_variation: (variation >> 5) & 0x07,
            cliff: (cliff_and_layer >> 4) & 0x0f,
            level: cliff_and_layer & 0x0f,
        }
    }
}
