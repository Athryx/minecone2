// This module contains functions for manipulating the textures

use image::{DynamicImage, RgbaImage, imageops::tile, GenericImage};
pub use image::imageops::{overlay, rotate90, rotate180, rotate270};
use anyhow::Result;

// length of textures in pixels
const TEXTURE_SIZE: u32 = 32;

pub fn tile_from_side(side: &DynamicImage) -> DynamicImage {
	let mut out = RgbaImage::new(4 * TEXTURE_SIZE, 3 * TEXTURE_SIZE);
	tile(&mut out, side);
	DynamicImage::ImageRgba8(out)
}

// describes the orientation of a texture face
pub enum TextureFace<'a> {
	// the texture face will be aligned with the direction of the stiched texture
	TextureAligned(&'a DynamicImage),
	// the texture will be aligned with the face of the block
	BlockAligned(&'a DynamicImage),
	Empty,
}

pub struct TextureStitchFaces<'a> {
	// z positive face
	pub front: TextureFace<'a>,
	// z negative face
	pub back: TextureFace<'a>,
	// x negative face
	pub left: TextureFace<'a>,
	// x positive face
	pub right: TextureFace<'a>,
	// y positive face
	pub top: TextureFace<'a>,
	// y negative face
	pub bottom: TextureFace<'a>,
}

pub fn stitch_texture(faces: TextureStitchFaces) -> Result<DynamicImage> {
	let mut out = DynamicImage::new_rgba8(4 * TEXTURE_SIZE, 3 * TEXTURE_SIZE);

	let mut copy_side = |face, x, y, rotate_func: Option<fn(&DynamicImage) -> RgbaImage>| {
		let temp;
		let rotated_image = match face {
			TextureFace::TextureAligned(image) => image,
			TextureFace::BlockAligned(image) => {
				if let Some(rotate_func) = rotate_func {
					temp = DynamicImage::ImageRgba8(rotate_func(image));
					&temp
				} else {
					image
				}
			},
			// the texture should already be transparant because new_rgba8 starts with 0 alpha
			TextureFace::Empty => return Ok(()),
		};

		out.copy_from(rotated_image, x * TEXTURE_SIZE, y * TEXTURE_SIZE)
	};

	copy_side(faces.front, 3, 1, Some(rotate90))?;
	copy_side(faces.back, 1, 1, Some(rotate270))?;
	copy_side(faces.left, 2, 0, None)?;
	copy_side(faces.right, 2, 2, Some(rotate180))?;
	copy_side(faces.top, 0, 1, Some(rotate270))?;
	copy_side(faces.bottom, 2, 1, Some(rotate270))?;

	Ok(out)
}
