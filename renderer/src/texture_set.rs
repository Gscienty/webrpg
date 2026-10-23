use std::{collections::BTreeMap, sync};

use fileformat::image::Image;
use shared::color;
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

pub struct TextureSet {
    pub white: Texture,
    pub black: Texture,
    pub placeholder: Texture,
    pub shadow: Texture,
    pub loading: Texture,

    pub team_glow: Vec<Texture>,
    pub team_color: Vec<Texture>,
    pub water: Option<Texture>,
    pub selection_circles: Vec<Texture>,

    pub textures: BTreeMap<usize, Texture>,
    texture_ids: BTreeMap<(String, String), usize>,
    pub texture_id_generator: sync::atomic::AtomicUsize,
}

impl TextureSet {
    fn load_image<T>(
        mpq: &mut webrpg_mpq::Archive,
        path: &str,
        convert: impl FnOnce(Image<'_>) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let data = mpq.read_file(path).map_err(Error::Mpq)?;
        convert(Image::load(path, &data)?)
    }

    fn load_mpq_texture(
        gl: &WebGl2RenderingContext,
        mpq: &mut webrpg_mpq::Archive,
        path: &str,
    ) -> Result<Texture, Error> {
        Self::load_image(mpq, path, |image| Texture::new(gl, image))
    }

    pub fn init(
        gl: &WebGl2RenderingContext,
        war3_mpq: Option<&mut webrpg_mpq::Archive>,
    ) -> Result<Self, Error> {
        let mut textures = Self {
            white: Texture::color(gl, color::Color32::white())?,
            black: Texture::color(gl, color::Color32::black())?,
            placeholder: Texture::placeholder(gl)?,
            shadow: Texture::shadow(gl)?,
            loading: Texture::loading(gl)?,

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
        textures.water = Some(Self::load_mpq_texture(gl, war3_mpq, WATER_FILENAME)?);
        textures
            .selection_circles
            .reserve_exact(SELECTION_CIRCLE_FILENAMES.len());
        for filename in SELECTION_CIRCLE_FILENAMES {
            textures
                .selection_circles
                .push(Self::load_mpq_texture(gl, war3_mpq, filename)?);
        }
        textures.team_glow.reserve_exact(MAX_TEAMS);
        textures.team_color.reserve_exact(MAX_TEAMS);

        for team in 0..MAX_TEAMS {
            let (glow_filename, color_filename) = team_filenames(team);

            textures
                .team_glow
                .push(Self::load_mpq_texture(gl, war3_mpq, &glow_filename)?);
            textures
                .team_color
                .push(Self::load_mpq_texture(gl, war3_mpq, &color_filename)?);
        }

        Ok(textures)
    }

    pub fn load_texture(
        &mut self,
        gl: &WebGl2RenderingContext,
        mpq: &mut webrpg_mpq::Archive,
        path: &str,
    ) -> Result<usize, Error> {
        let key = (mpq.id().to_owned(), path.to_owned());
        if let Some(texture_id) = self.texture_ids.get(&key) {
            return Ok(*texture_id);
        }

        let texture = Self::load_mpq_texture(gl, mpq, path)?;
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
