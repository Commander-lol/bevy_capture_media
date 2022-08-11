use bevy_render::texture::TextureFormatPixelInfo;
use image::RgbaImage;
use wgpu::TextureFormat;

pub fn frame_data_to_rgba_image(
	width: u32,
	height: u32,
	buffer: Vec<u8>,
	format: TextureFormat,
) -> RgbaImage {
	let pixels = buffer.chunks(format.pixel_size()).collect::<Vec<&[u8]>>();
	RgbaImage::from_fn(width, height, |x, y| {
		let index = ((y * width) + x) as usize;
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
	})
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

pub fn to_grouped_rgba(buffer: &[u8], format: TextureFormat) -> Vec<[u8; 4]> {
	buffer
		.chunks_exact(4)
		.map(|pixel| match pixel {
			[rb, g, br, a] => match format {
				TextureFormat::Rgba8UnormSrgb
				| TextureFormat::Rgba8Uint
				| TextureFormat::Rgba8Sint
				| TextureFormat::Rgba8Snorm
				| TextureFormat::Rgba8Unorm => [*rb, *g, *br, *a],
				TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => [*br, *g, *rb, *a],
				_ => {
					panic!("Unhandled texture format {:?}", format);
				}
			},
			_ => panic!("chunks_exact did not give an exact chunk"),
		})
		.collect()
}
