pub enum BLPEncoding {
    Uncompressed,
    Dxt,
    UncompressedRawBGRA,
}

pub enum BLPAlphaDepth {
    Depth0,
    Depth1,
    Depth4,
    Depth8,
}

pub enum BLPAlphaEncoding {
    Dxt1,
    Dxt3,
    Dxt5,
}

pub enum BLPFormat {
    Jpeg,

    PalettedNoAlpha,
    PalettedAlpha1,
    PalettedAlpha4,
    PalettedAlpha8,

    Dxt1NoAlpha,
    Dxt1Alpha1,
    Dxt3NoAlpha,
    Dxt3Alpha4,
    Dxt3Alpha8,
    Dxt5Alpha8,
}
