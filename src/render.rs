use std::num::NonZeroU32;
use std::ops::Deref;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::TextureFormat;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::{ImageFormat, TextureFormatPixelInfo};
use wgpu::{
	BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d, ImageCopyBuffer,
	ImageDataLayout, Maintain, TextureDescriptor,
};

use crate::data::SharedDataSmuggler;

pub fn layout_data(size: Vec2, format: TextureFormat) -> ImageDataLayout {
	ImageDataLayout {
		bytes_per_row: if size.y > 1.0 {
			NonZeroU32::new((size.x as usize * (format.pixel_size())) as u32)
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
	for (_id, mut recorder) in smugglers.lock().unwrap().iter_mut() {
		if let Some(image) = images.get(&recorder.target_handle) {
			let width = image.size.x as u32;
			let height = image.size.y as u32;

			let device = render_device.wgpu_device();
			let destination = device.create_buffer(&BufferDescriptor {
				label: None,
				size: (width * height * image.texture_format.pixel_size() as u32) as u64,
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
					layout: layout_data(image.size, image.texture_format),
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

			recorder.last_frame = Some(result);
		}
	}
}
