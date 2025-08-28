use slotmap::{SlotMap, new_key_type};

use crate::render::{shaders::wgsl_draw, texture::TextureBundle};

new_key_type! {
    pub struct TextureKey;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFilter {
    Linear,
    Nearest,
}

pub struct LoadedTexture {
    pub(crate) texture: TextureBundle,
    pub(crate) bind_group: wgsl_draw::globals::BindGroup1,
}

pub type TextureMap = SlotMap<TextureKey, LoadedTexture>;
