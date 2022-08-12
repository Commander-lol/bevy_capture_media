use std::io::Read;
use std::num::NonZeroU32;
use std::ops::Deref;

use bevy_ecs::system::{Res, ResMut};
use bevy_render::render_asset::RenderAssets;
use bevy_render::render_resource::TextureFormat;
use bevy_render::renderer::{RenderDevice, RenderQueue};
use bevy_render::texture::{Image, ImageFormat, TextureFormatPixelInfo};
use wgpu::{
	BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, ImageCopyBuffer,
	ImageDataLayout, Maintain, TextureDescriptor, COPY_BYTES_PER_ROW_ALIGNMENT,
};

use crate::data::SharedDataSmuggler;

pub fn align_byte_size(value: u32) -> u32 {
	value + (COPY_BYTES_PER_ROW_ALIGNMENT - (value % COPY_BYTES_PER_ROW_ALIGNMENT))
}

pub fn get_aligned_size(width: u32, height: u32, pixel_size: u32) -> u32 {
	height * align_byte_size(width * pixel_size)
}

pub fn layout_data(width: u32, height: u32, format: TextureFormat) -> ImageDataLayout {
	ImageDataLayout {
		bytes_per_row: if height > 1 {
			// 1 = 1 row
			NonZeroU32::new(get_aligned_size(width, 1, format.pixel_size() as u32))
		} else {
			None
		},
		rows_per_image: None,
		..Default::default()
	}
}

pub fn smuggle_frame(
	mut smugglers: ResMut<SharedDataSmuggler>,
	images: Res<RenderAssets<Image>>,
	render_device: Res<RenderDevice>,
	render_queue: Res<RenderQueue>,
) {
	let mut smugglers = smugglers.lock().unwrap();
	for (_id, mut recorder) in smugglers.iter_mut() {
		if let Some(image) = images.get(&recorder.target_handle) {
			let width = image.size.x as u32;
			let height = image.size.y as u32;

			let device = render_device.wgpu_device();
			let destination = device.create_buffer(&BufferDescriptor {
				label: None,
				size: get_aligned_size(width, height, image.texture_format.pixel_size() as u32)
					as u64,
				usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
				mapped_at_creation: false,
			});

			let texture = image.texture.clone();
			let mut encoder =
				render_device.create_command_encoder(&CommandEncoderDescriptor { label: None });

			encoder.copy_texture_to_buffer(
				texture.as_image_copy(),
				ImageCopyBuffer {
					buffer: &destination,
					layout: layout_data(
						image.size.x as u32,
						image.size.y as u32,
						image.texture_format,
					),
				},
				Extent3d {
					width,
					height,
					..Default::default()
				},
			);

			render_queue.submit([encoder.finish()]);
			let slice = destination.slice(..);
			slice.map_async(wgpu::MapMode::Read, move |result| {
				let err = result.err();
				if err.is_some() {
					panic!("{}", err.unwrap().to_string());
				}
			});
			device.poll(Maintain::Wait);

			let data = slice.get_mapped_range();
			let result = Vec::from(data.deref());
			drop(data);

			if result.len() == ((width * height) as usize * image.texture_format.pixel_size()) {
				recorder.last_frame = Some(result)
			} else {
				// Our buffer has been padded because we needed to align to a multiple of 256.
				// We can simplify things elsewhere by removing the padding before smuggling the
				// frame
				let pixel_size = image.texture_format.pixel_size() as u32;
				let initial_row_bytes = width * pixel_size;
				let buffered_row_bytes = align_byte_size(width * pixel_size);

				let result = result
					.chunks_exact(buffered_row_bytes as usize) // Take rows
					.flat_map(|row| row.iter().take(initial_row_bytes as usize)) // Take only the expected number of bytes
					.copied() // Darned references
					.collect();

				recorder.last_frame = Some(result);
			}
		}
	}
}
