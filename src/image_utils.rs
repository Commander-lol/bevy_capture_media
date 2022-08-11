use bevy::math::UVec2;
use bevy::render::texture::TextureFormatPixelInfo;
use image::RgbaImage;
use wgpu::TextureFormat;

pub fn frame_data_to_rgba_image(size: UVec2, buffer: Vec<u8>, format: TextureFormat) -> RgbaImage {
	let pixels = buffer.chunks(format.pixel_size()).collect::<Vec<&[u8]>>();
	let mut image = RgbaImage::from_fn(size.x, size.y, |x, y| {
		let index = ((y * size.x) + x) as usize;
		let pixel = pixels[index];

		match format {
			TextureFormat::Rgba8UnormSrgb
			| TextureFormat::Rgba8Uint
			| TextureFormat::Rgba8Sint
			| TextureFormat::Rgba8Snorm
			| TextureFormat::Rgba8Unorm => image::Rgba([pixel[0], pixel[1], pixel[2], pixel[3]]),
			TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => {
				image::Rgba([pixel[2], pixel[1], pixel[0], pixel[3]])
			}
			_ => {
				panic!("Unhandled texture format {:?}", format);
			}
		}
	});

	image
}

pub fn to_rgba(buffer: Vec<u8>, format: TextureFormat) -> Vec<u8> {
	match format {
		TextureFormat::Rgba8UnormSrgb
		| TextureFormat::Rgba8Uint
		| TextureFormat::Rgba8Sint
		| TextureFormat::Rgba8Snorm
		| TextureFormat::Rgba8Unorm => buffer,
		TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => buffer
			.chunks_exact(4)
			.flat_map(|pixel| match pixel {
				[b, g, r, a] => [*r, *g, *b, *a],
				_ => panic!("Chunks didn't give us a 4 chunk"),
			})
			.collect(),
		_ => {
			panic!("Unhandled texture format {:?}", format);
		}
	}
}
