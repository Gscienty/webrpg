use std::{collections::BTreeMap, sync};

use web_sys::WebGl2RenderingContext;

use crate::{Error, texture::Texture};

pub const MAX_TEAMS: usize = 16;
const WATER_FILENAME: &str = "ReplaceableTextures\\Water\\Water12.blp";
const SELECTION_CIRCLE_FILENAMES: [&str; 3] = [
    "ReplaceableTextures\\Selection\\SelectionCircleSmall.blp",
    "ReplaceableTextures\\Selection\\SelectionCircleMed.blp",
    "ReplaceableTextures\\Selection\\SelectionCircleLarge.blp",
];

fn team_filenames(team: usize) -> (String, String) {
    (
        format!("ReplaceableTextures\\TeamGlow\\TeamGlow{team:02}.blp"),
        format!("ReplaceableTextures\\TeamColor\\TeamColor{team:02}.blp"),
    )
}

pub struct Textures {
    pub white_texture: Texture,
    pub black_texture: Texture,
    pub placeholder_texture: Texture,
    pub blob_shadow_texture: Texture,
    pub loading_indicator_texture: Texture,

    pub team_glow: Vec<Texture>,
    pub team_color: Vec<Texture>,
    pub water: Option<Texture>,
    pub selection_circles: Vec<Texture>,

    pub textures: BTreeMap<usize, Texture>,
    texture_ids: BTreeMap<(String, String), usize>,
    pub texture_id_generator: sync::atomic::AtomicUsize,
}

impl Textures {
    pub(super) fn init(
        gl: &WebGl2RenderingContext,
        war3_mpq: Option<&mut dyn webrpg_mpq::Archive>,
    ) -> Result<Self, Error> {
        let mut textures = Self {
            white_texture: Texture::white(gl)?,
            black_texture: Texture::black(gl)?,
            placeholder_texture: Texture::placeholder(gl)?,
            blob_shadow_texture: Texture::blob_shadow(gl)?,
            loading_indicator_texture: Texture::loading_indicator(gl)?,
            team_glow: Vec::new(),
            team_color: Vec::new(),
            water: None,
            selection_circles: Vec::new(),

            textures: BTreeMap::new(),
            texture_ids: BTreeMap::new(),
            texture_id_generator: sync::atomic::AtomicUsize::new(0),
        };

        let Some(war3_mpq) = war3_mpq else {
            return Ok(textures);
        };
        let water = war3_mpq.read_file(WATER_FILENAME).map_err(Error::Mpq)?;
        textures.water = Some(Texture::load(gl, WATER_FILENAME, &water)?);
        textures
            .selection_circles
            .reserve_exact(SELECTION_CIRCLE_FILENAMES.len());
        for filename in SELECTION_CIRCLE_FILENAMES {
            let data = war3_mpq.read_file(filename).map_err(Error::Mpq)?;
            textures
                .selection_circles
                .push(Texture::load(gl, filename, &data)?);
        }
        textures.team_glow.reserve_exact(MAX_TEAMS);
        textures.team_color.reserve_exact(MAX_TEAMS);

        for team in 0..MAX_TEAMS {
            let (glow_filename, color_filename) = team_filenames(team);

            let glow = war3_mpq.read_file(&glow_filename).map_err(Error::Mpq)?;
            let color = war3_mpq.read_file(&color_filename).map_err(Error::Mpq)?;
            textures
                .team_glow
                .push(Texture::load(gl, &glow_filename, &glow)?);
            textures
                .team_color
                .push(Texture::load(gl, &color_filename, &color)?);
        }

        Ok(textures)
    }

    pub fn load_texture(
        &mut self,
        gl: &WebGl2RenderingContext,
        mpq: &mut dyn webrpg_mpq::Archive,
        path: &str,
    ) -> Result<usize, Error> {
        let key = (mpq.id().to_owned(), path.to_owned());
        if let Some(texture_id) = self.texture_ids.get(&key) {
            return Ok(*texture_id);
        }

        let data = mpq.read_file(path).map_err(Error::Mpq)?;
        let texture = Texture::load(gl, path, &data)?;
        let texture_id = self
            .texture_id_generator
            .fetch_add(1, sync::atomic::Ordering::Relaxed);
        self.textures.insert(texture_id, texture);
        self.texture_ids.insert(key, texture_id);
        Ok(texture_id)
    }

    pub fn get_texture<'s>(
        &'s self,
        texture_id: usize,
        team_id: u32,
        replaceable_id: u32,
        override_texture: Option<&'s Texture>,
        override_slot: u32,
    ) -> Option<&'s Texture> {
        match replaceable_id {
            1 => self.team_color.get(team_id as usize),
            2 => self.team_glow.get(team_id as usize),
            _ => {
                if replaceable_id != 0
                    && override_texture.is_some()
                    && (override_slot == 0 || replaceable_id == override_slot)
                {
                    return override_texture;
                }
                self.textures.get(&texture_id)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_texture_paths_are_zero_padded() {
        assert_eq!(
            team_filenames(0),
            (
                "ReplaceableTextures\\TeamGlow\\TeamGlow00.blp".into(),
                "ReplaceableTextures\\TeamColor\\TeamColor00.blp".into(),
            )
        );
        assert!(team_filenames(MAX_TEAMS - 1).0.ends_with("TeamGlow15.blp"));
    }
}
