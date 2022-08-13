#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#[allow(clippy::type_complexity)]
pub mod data;
pub mod formats;
#[cfg(any(feature = "gif", feature = "png"))]
mod image_utils;
mod management;
mod render;
#[cfg(target_arch = "wasm32")]
mod web_utils;

mod plugin {
	use bevy_app::{App, CoreStage, Plugin};
	use bevy_render::{RenderApp, RenderStage};

	use super::*;

	pub struct BevyCapturePlugin;
	impl Plugin for BevyCapturePlugin {
		fn build(&self, app: &mut App) {
			let tracking_tracker = data::ActiveRecorders::default();
			let data_smuggler = data::SharedDataSmuggler::default();

			app.add_event::<data::StartTrackingCamera>()
				.add_event::<data::StopTrackingCamera>()
				.insert_resource(tracking_tracker)
				.insert_resource(data_smuggler.clone())
				.add_system_to_stage(CoreStage::First, management::clean_cameras)
				.add_system_to_stage(CoreStage::First, management::move_camera_buffers)
				.add_system_to_stage(CoreStage::PostUpdate, management::sync_tracking_cameras)
				.add_system_to_stage(
					CoreStage::PostUpdate,
					management::start_tracking_orthographic_camera,
				);

			#[cfg(feature = "gif")]
			{
				app.add_event::<formats::gif::CaptureGifRecording>()
					.add_system_to_stage(
						CoreStage::PostUpdate,
						formats::gif::capture_gif_recording,
					);

				#[cfg(not(target_arch = "wasm32"))]
				app.add_system_to_stage(
					CoreStage::Last,
					management::clean_unmonitored_tasks::<formats::gif::SaveGifRecording>,
				);
			}
			#[cfg(feature = "png")]
			{
				app.add_event::<formats::png::SavePngFile>()
					.add_system_to_stage(CoreStage::PostUpdate, formats::png::save_single_frame);

				#[cfg(not(target_arch = "wasm32"))]
				app.add_system_to_stage(
					CoreStage::Last,
					management::clean_unmonitored_tasks::<formats::png::SaveFrameTask>,
				);
			}

			let render_app = app.get_sub_app_mut(RenderApp)
				.expect("bevy_capture_media will not work without the render app. Either enable this sub app, or disable bevy_capture_media");

			render_app
				.insert_resource(data_smuggler)
				.add_system_to_stage(RenderStage::Render, render::smuggle_frame);
		}
	}
}

pub use data::MediaCapture;
pub use plugin::BevyCapturePlugin;
