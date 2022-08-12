use std::io::Cursor;
use std::path::{Path, PathBuf};

use bevy_asset::{Assets, Handle};
use bevy_ecs::component::Component;
use bevy_ecs::prelude::Events;
use bevy_ecs::system::{Commands, Res, ResMut};
use bevy_render::texture::Image;
use bevy_render::texture::TextureFormatPixelInfo;
use bevy_tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use image::{EncodableLayout, ImageBuffer, ImageFormat};
use wgpu::TextureFormat;

use crate::data::{ActiveRecorders, Alignment, CaptureFrame, HasTaskStatus};
use crate::image_utils::frame_data_to_rgba_image;
#[cfg(target_arch = "wasm32")]
use crate::web_utils;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub enum SavePng {
	#[default]
	Basic,
	Watermarked {
		watermark: Handle<Image>,
		alignment: Alignment,
	},
}

pub type SavePngFile = CaptureFrame<SavePng>;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Component)]
pub struct SaveFrameTask(pub Task<()>);

#[cfg(not(target_arch = "wasm32"))]
impl HasTaskStatus for SaveFrameTask {
	fn is_done(&mut self) -> bool {
		let result = future::block_on(future::poll_once(&mut self.0));
		result.is_some()
	}
}

pub fn save_single_frame(
	mut commands: Commands,
	mut events: ResMut<Events<SavePngFile>>,
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

				// if let SavePng::Watermarked { watermark } = event {
				// 	let watermark_image = watermark
				// }

				#[cfg(not(target_arch = "wasm32"))]
				{
					let file_name = event.path.unwrap_or_else(|| {
						PathBuf::from(format!(
							"{}.png",
							std::time::UNIX_EPOCH.elapsed().unwrap().as_secs()
						))
					});

					if let Err(e) = image.save_with_format(file_name, ImageFormat::Png) {
						log::error!("Failed to write screenshot: {}", e);
					}
				}
				#[cfg(target_arch = "wasm32")]
				{
					let file_name = event
						.path
						.and_then(|path| {
							path.file_name()
								.and_then(|name| name.to_str())
								.map(|name| PathBuf::from(name))
						})
						.unwrap_or_else(|| {
							PathBuf::from(format!("{}.png", crate::web_utils::get_now()))
						});

					log::info!("Image size: {}", image.len());

					let mut file_bytes = Cursor::new(vec![0; image.len()]);
					image.write_to(&mut file_bytes, image::ImageFormat::Png);

					crate::web_utils::download_bytes(file_name, file_bytes.into_inner())
				}
			});

			#[cfg(target_arch = "wasm32")]
			task.detach();
			#[cfg(not(target_arch = "wasm32"))]
			commands.spawn().insert(SaveFrameTask(task));
		}
	}
}
