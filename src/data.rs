use std::collections::{HashMap, VecDeque};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bevy_asset::Handle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventWriter;
use bevy_ecs::system::SystemParam;
use bevy_render::camera::OrthographicProjection;
use bevy_render::texture::{BevyDefault, Image};
use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

pub type RecorderID = usize;

/// Stores the user defined ID given to this recorder when it was created
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Default, Component)]
pub struct Recorder(pub RecorderID);
/// Stores a reference to an entity. Indicates that the entity this component is attached to
/// should track the referenced entity
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Component)]
pub struct Track(pub Entity);

/// Align one item (the target) based on a relative position to some point on another item
/// (the background)
///
/// The canonical example of this would be overlaying a watermark on top of an image. The
/// watermark would be the target (of the alignment), while the image would be the background
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Alignment {
	/// An offset between the top left corner of the target and the top left corner of the background
	TopLeft { top: u32, left: u32 },
	/// Offset the top edge of the target from the top edge of the background. Horizontal alignment
	/// will align the mid point of the target to the mid point of the background
	TopCentre { top: u32 },
	/// An offset between the top right corner of the target and the top right corner of the background
	TopRight { top: u32, right: u32 },
	/// Offset the left edge of the target from the left edge of the background. Vertical alignment
	/// will align the mid point of the target to the mid point of the background
	CentreLeft { left: u32 },
	/// Align the vertical and horizontal centres of the target and the background
	CentreCentre,
	/// Offset the right edge of the target from the left edge of the background. Vertical alignment
	/// will align the mid point of the target to the mid point of the background
	CentreRight { right: u32 },
	/// An offset between the bottom left corner of the target and the bottom right corner of the background
	BottomLeft { bottom: u32, left: u32 },
	/// Offset the bottom edge of the target from the top edge of the background. Horizontal alignment
	/// will align the mid point of the target to the mid point of the background
	BottomCentre { bottom: u32 },
	/// An offset between the bottom right corner of the target and the top right corner of the background
	BottomRight { bottom: u32, right: u32 },
}

/// Holds a single frame buffer's worth of pixel data and the amount of time this frame took to render
#[derive(Clone, Debug)]
pub struct TextureFrame {
	/// All of the bytes for one frame
	pub texture: Vec<u8>,
	/// The amount of time it took to render the frame
	pub frame_time: Duration,
}

impl TextureFrame {
	/// Create a new frame with a duration of 0
	pub fn zeroed(frame: Vec<u8>) -> Self {
		Self {
			texture: frame,
			frame_time: Duration::ZERO,
		}
	}
	/// Create a new frame with a specified duration
	pub fn with_duration(frame: Vec<u8>, delta: Duration) -> Self {
		Self {
			texture: frame,
			frame_time: delta,
		}
	}
	/// Create a new frame with a duration specified in delta seconds. 1.0 = 1 second.
	pub fn with_seconds(frame: Vec<u8>, delta: f32) -> Self {
		Self {
			texture: frame,
			frame_time: Duration::from_secs_f32(delta),
		}
	}
}

/// What action should a recorder take when it has completed a given command
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub enum PostCaptureAction {
	/// Continue to capture frames into a new frame buffer
	#[default]
	Continue,
	/// TODO: Implement stop-once-recorded
	Stop,
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

// -- TRAITS --

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

/// Request that a single frame is captured, with the given type
/// information. This will be the most recent frame already stored when
/// the event is processed, rather than the next frame to be stored after
/// the event is processed.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub struct CaptureFrame<CaptureType> {
	/// The identifier for the camera tracker that should capture a frame
	pub tracking_id: RecorderID,
	/// The file path that the frame should be saved to, including file name
	/// and extension. *N.B.*: The extension does not affect the format to be
	/// captured
	///
	/// ## wasm
	///
	/// When capturing a frame on the web target, the path information is stripped
	/// down to the filename + extension. This will be used as the name of the file
	/// to download in the browser
	///
	/// ## `None`
	///
	/// When you don't provide a path, a timestamped filename will be generated, and the
	/// file will be saved to the current directory. The format may differ depending on
	/// the platform - if you rely on a consistent format across platforms, you must provide
	/// a `path`.
	pub path: Option<PathBuf>,
	/// Determines what the camera tracker should do after recording this frame
	pub and_then: PostCaptureAction,
	/// Define the type of capture to use (e.g. PNG). Some capture types may provide more
	/// information or alter the capture behaviour
	pub capture_type: CaptureType,
}

/// Request that the current frame buffer is converted into the specified `CaptureType` and
/// saved. Encoding the frame buffer usually takes some amount of time on most platforms, which
/// will happen asynchronously
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub struct CaptureRecording<CaptureType> {
	/// The identifier for the camera tracker that should capture a frame
	pub tracking_id: RecorderID,
	/// The file path that the frame should be saved to, including file name
	/// and extension. *N.B.*: The extension does not affect the format to be
	/// captured
	///
	/// ## wasm
	///
	/// When capturing a frame on the web target, the path information is stripped
	/// down to the filename + extension. This will be used as the name of the file
	/// to download in the browser
	///
	/// ## `None`
	///
	/// When you don't provide a path, a timestamped filename will be generated, and the
	/// file will be saved to the current directory. The format may differ depending on
	/// the platform - if you rely on a consistent format across platforms, you must provide
	/// a `path`.
	pub path: Option<PathBuf>,
	/// Determines what the camera tracker should do after recording this frame
	pub and_then: PostCaptureAction,
	/// Define the type of capture to use (e.g. PNG). Some capture types may provide more
	/// information or alter the capture behaviour
	pub capture_type: CaptureType,
}

// -- Handlers --

/// Dispatch events to control media capture. Works as a system param. The methods available
/// will depend on what formats you have enabled with feature flags
///
/// ## Example
///
/// ```rust
/// pub fn my_screenshot_system(
/// 	input: Res<Input<KeyCode>>,
/// 	mut media: MediaCapture
/// ) {
/// 	if input.just_released(KeyCode::Escape) {
/// 		// Typically you would store the recorder ID in a resource
/// 		media.capture_png_with_path(1, "screenshots/achievement_123.png");
/// 	}
/// }
/// ```
#[derive(SystemParam)]
pub struct MediaCapture<'w, 's> {
	#[cfg(feature = "gif")]
	capture_gif: EventWriter<'w, 's, crate::formats::gif::CaptureGifRecording>,
	#[cfg(feature = "png")]
	capture_png: EventWriter<'w, 's, crate::formats::png::SavePngFile>,

	start_tracking: EventWriter<'w, 's, StartTrackingCamera>,
	stop_tracking: EventWriter<'w, 's, StopTrackingCamera>,
}

impl<'w, 's> MediaCapture<'w, 's> {
	/// Start to capture frames for the given camera. The number of frames captured
	/// is determined by the given `Duration`, and will vary based on the framerate
	/// of the application
	///
	/// # Arguments
	///
	/// - **tracking_id**: A user specified identifier for the recorder, which will be used later to request capture
	/// - **target**: The entity that will be tracked. Expected to be a camera with a transform, a `Camera` component, and a projection
	/// - **length**: The duration of time that the tracker should hold frames for. E.g. specifying "10 seconds" will cause the tracker to hold the past 10 seconds worth of frames, and to create recordings 10 seconds long
	pub fn start_tracking_camera(
		&mut self,
		tracking_id: RecorderID,
		target: Entity,
		length: Duration,
	) {
		self.start_tracking.send(StartTrackingCamera {
			tracking_id,
			cam_entity: target,
			length,
		});
	}

	/// Stop capturing frames for the given camera, remove the tracking entity,
	/// discard stored frames, and ignore any in-flight requests for capture
	/// that have not started processing yet.
	///
	/// Requests for capture that _have_ started processing will continue. Capture
	/// tasks take ownership of any frames they're using, so output will not be affected
	pub fn stop_tracking_camera(&mut self, tracking_id: RecorderID) {
		self.stop_tracking.send(StopTrackingCamera { tracking_id })
	}

	/// Request that the recorder identified by `tracking_id` encodes its
	/// stored frames into a gif, and save it with a default name
	#[cfg(feature = "gif")]
	pub fn capture_gif(&mut self, tracking_id: RecorderID) {
		self.capture_gif.send(CaptureRecording {
			tracking_id,
			and_then: PostCaptureAction::Continue,
			path: None,
			capture_type: crate::formats::gif::RecordGif,
		});
	}

	/// Request that the recorder identified by `tracking_id` encodes its
	/// stored frames into a gif, and save it to a specified path
	#[cfg(feature = "gif")]
	pub fn capture_gif_with_path<P: AsRef<Path>>(&mut self, tracking_id: RecorderID, path: P) {
		self.capture_gif.send(CaptureRecording {
			tracking_id,
			and_then: PostCaptureAction::Continue,
			path: Some(path.as_ref().to_path_buf()),
			capture_type: crate::formats::gif::RecordGif,
		});
	}
	/// Request that the recorder identified by `tracking_id` encodes its
	/// most recently stored frame into a PNG image, and save it with a
	/// default name
	#[cfg(feature = "png")]
	pub fn capture_png(&mut self, tracking_id: RecorderID) {
		self.capture_png.send(CaptureFrame {
			tracking_id,
			and_then: PostCaptureAction::Continue,
			path: None,
			capture_type: crate::formats::png::SavePng::Basic,
		});
	}
	/// Request that the recorder identified by `tracking_id` encodes its
	/// most recently stored frame into a PNG image, and save it to a
	/// specified path
	#[cfg(feature = "png")]
	pub fn capture_png_with_path<P: AsRef<Path>>(&mut self, tracking_id: RecorderID, path: P) {
		self.capture_png.send(CaptureFrame {
			tracking_id,
			and_then: PostCaptureAction::Continue,
			path: Some(path.as_ref().to_path_buf()),
			capture_type: crate::formats::png::SavePng::Basic,
		})
	}
}
