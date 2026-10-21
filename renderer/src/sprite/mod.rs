use std::pin::Pin;

use fileformat::mdx;
use web_sys::WebGl2RenderingContext;

use crate::{Error, sprite::body::Body};

pub mod body;
pub mod bound;
pub mod render_params;

pub struct Sprite<'s> {
    model: Pin<Box<mdx::Model<'s>>>,

    body: Body<'s>,
}

impl<'s> Sprite<'s> {
    pub fn new(
        gl: &WebGl2RenderingContext,
        model: Pin<Box<mdx::Model<'s>>>,
    ) -> Result<Self, Error> {
        let model_ref: &'s mdx::Model<'s> = unsafe { &*(model.as_ref().get_ref() as *const _) };
        let body = Body::new(gl, model_ref)?;

        Ok(Self { model, body })
    }
}
