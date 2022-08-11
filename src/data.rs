use std::collections::{HashMap, VecDeque};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bevy_asset::Handle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_render::camera::OrthographicProjection;
use bevy_render::texture::{BevyDefault, Image};
use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

pub type RecorderID = usize;

#[derive(Clone, Debug)]
pub struct TextureFrame {
	pub texture: Vec<u8>,
	pub frame_time: Duration,
}

impl TextureFrame {
	pub fn zeroed(frame: Vec<u8>) -> Self {
		Self {
			texture: frame,
			frame_time: Duration::ZERO,
		}
	}
	pub fn with_duration(frame: Vec<u8>, delta: Duration) -> Self {
		Self {
			texture: frame,
			frame_time: delta,
		}
	}
	pub fn with_seconds(frame: Vec<u8>, delta: f32) -> Self {
		Self {
			texture: frame,
			frame_time: Duration::from_secs_f32(delta),
		}
	}
}

#[derive(Debug)]
pub struct ActiveRecorder {
	pub tracker: Entity,
	pub target_handle: Handle<Image>,
	pub target_duration: Duration,
	pub frames: VecDeque<TextureFrame>,
}

#[derive(Default, Debug)]
pub struct ActiveRecorders(pub HashMap<RecorderID, ActiveRecorder>);
impl Deref for ActiveRecorders {
	type Target = HashMap<RecorderID, ActiveRecorder>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for ActiveRecorders {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Debug)]
pub struct RenderData {
	pub target_handle: Handle<Image>,
	pub last_frame: Option<Vec<u8>>,
}
#[derive(Default, Debug)]
pub struct DataSmuggler(pub HashMap<RecorderID, RenderData>);
impl Deref for DataSmuggler {
	type Target = HashMap<RecorderID, RenderData>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for DataSmuggler {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

pub type SharedDataSmuggler = Arc<Mutex<DataSmuggler>>;

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Default, Component)]
pub struct Recorder(pub RecorderID);
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Component)]
pub struct Track(pub Entity);

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub enum PostCaptureAction {
	#[default]
	Continue,
	/// TODO: Implement stop-once-recorded
	Stop,
}

// -- EVENTS --

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct StartTrackingCamera {
	pub cam_entity: Entity,
	pub tracking_id: RecorderID,
	pub length: Duration,
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct StopTrackingCamera {
	pub tracking_id: RecorderID,
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub struct CaptureFrame {
	pub tracking_id: RecorderID,
	pub path: Option<PathBuf>,
	pub and_then: PostCaptureAction,
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub struct CaptureRecording<CaptureType> {
	pub tracking_id: RecorderID,
	pub and_then: PostCaptureAction,
	pub capture_type: CaptureType,
}

pub struct CaptureRecordingType;
impl CaptureRecordingType {
	#[cfg(feature = "gif")]
	pub fn capture_gif(
		tracking_id: RecorderID,
		and_then: PostCaptureAction,
	) -> CaptureRecording<crate::formats::gif::RecordGif> {
		CaptureRecording {
			and_then,
			tracking_id,
			capture_type: crate::formats::gif::RecordGif,
		}
	}
}

pub trait ProjectToImage {
	fn project_to_image(&self) -> Image;
}

impl ProjectToImage for &OrthographicProjection {
	fn project_to_image(&self) -> Image {
		let format = TextureFormat::bevy_default();
		let size = Extent3d {
			width: (self.right - self.left).max(0.0) as u32,
			height: (self.top - self.bottom).max(0.0) as u32,
			..Default::default()
		};

		let mut img = Image {
			texture_descriptor: TextureDescriptor {
				label: None,
				size,
				dimension: TextureDimension::D2,
				format,
				usage: TextureUsages::TEXTURE_BINDING
					| TextureUsages::RENDER_ATTACHMENT
					| TextureUsages::COPY_DST
					| TextureUsages::COPY_SRC,
				sample_count: 1,
				mip_level_count: 1,
			},
			..Default::default()
		};
		img.resize(size);
		img
	}
}

pub trait HasTaskStatus: Component {
	fn is_done(&mut self) -> bool;
}
