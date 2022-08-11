use std::path::{Path, PathBuf};

use bevy_asset::Assets;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::Events;
use bevy_ecs::system::{Commands, Res, ResMut};
use bevy_render::texture::Image;
use bevy_render::texture::TextureFormatPixelInfo;
use bevy_tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use image::{ImageBuffer, ImageFormat};
use wgpu::TextureFormat;

use crate::data::{ActiveRecorders, CaptureFrame, HasTaskStatus};
use crate::image_utils::frame_data_to_rgba_image;

#[derive(Component)]
pub struct SaveFrameTask(pub Task<()>);

impl HasTaskStatus for SaveFrameTask {
	fn is_done(&mut self) -> bool {
		let result = future::block_on(future::poll_once(&mut self.0));
		result.is_some()
	}
}

pub fn save_single_frame(
	mut commands: Commands,
	mut events: ResMut<Events<CaptureFrame>>,
	recorders: ResMut<ActiveRecorders>,
	images: Res<Assets<Image>>,
) {
	let thread_pool = AsyncComputeTaskPool::get();
	'event_drain: for event in events.drain() {
		if let Some(recorder) = recorders.get(&event.tracking_id) {
			let data = match recorder.frames.back() {
				Some(data) => data.texture.clone(),
				None => continue 'event_drain,
			};

			let (width, height, target_format) = match images.get(&recorder.target_handle) {
				Some(image) => (
					image.size().x as u32,
					image.size().y as u32,
					image.texture_descriptor.format,
				),
				None => continue 'event_drain,
			};

			let task = thread_pool.spawn(async move {
				let data = data;
				let format = target_format;

				let expected_size = width * height * format.pixel_size() as u32;
				if expected_size != data.len() as u32 {
					log::error!("Failed to assert that the data frame is correctly formatted");
					return;
				}

				let image = frame_data_to_rgba_image(width, height, data, format);
				if let Err(e) = image.save_with_format(
					event.path.unwrap_or_else(|| {
						PathBuf::from(format!(
							"{}.png",
							std::time::UNIX_EPOCH.elapsed().unwrap().as_secs()
						))
					}),
					ImageFormat::Png,
				) {
					log::error!("Failed to write screenshot: {}", e);
				}
			});

			if cfg!(target_arch = "wasm32") {
				task.detach();
			} else {
				commands.spawn().insert(SaveFrameTask(task));
			}
		}
	}
}
