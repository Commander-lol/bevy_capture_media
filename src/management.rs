use std::time::Duration;

use bevy_asset::Assets;
use bevy_core_pipeline::core_2d::Camera2dBundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Events;
use bevy_ecs::query::{With, Without};
use bevy_ecs::system::{Commands, Query, Res, ResMut};
use bevy_render::camera::{Camera, OrthographicProjection, RenderTarget};
use bevy_render::texture::Image;
use bevy_time::Time;
use bevy_transform::components::Transform;

use crate::data::ProjectToImage;
use crate::data::{
	ActiveRecorder, ActiveRecorders, HasTaskStatus, Recorder, RenderData, SharedDataSmuggler,
	StartTrackingCamera, TextureFrame, Track,
};

pub fn sync_tracking_cameras(
	mut trackers: Query<(&mut Transform, &mut OrthographicProjection, &Track), With<Recorder>>,
	tracked: Query<(&Transform, &OrthographicProjection), (With<Camera>, Without<Recorder>)>,
) {
	for (mut transform, mut ortho, Track(camera)) in &mut trackers {
		if let Ok((target_transform, target_ortho)) = tracked.get(*camera) {
			*transform = *target_transform;
			*ortho = target_ortho.clone();
		}
	}
}

pub fn clean_cameras(
	mut commands: Commands,
	mut smugglers: ResMut<SharedDataSmuggler>,
	mut recorders: ResMut<ActiveRecorders>,
	trackers: Query<(Entity, &Recorder, &Track)>,
	tracked: Query<(), With<Camera>>,
) {
	for (entity, Recorder(id), Track(target)) in &trackers {
		if tracked.get(*target).is_err() {
			commands.entity(entity).despawn();
			smugglers.lock().unwrap().remove(id);
			recorders.remove(id);
		}
	}
}

pub fn clean_unmonitored_tasks<T: HasTaskStatus>(
	mut commands: Commands,
	mut tasks: Query<(Entity, &mut T)>,
) {
	for (entity, mut task) in &mut tasks {
		if task.is_done() {
			commands.entity(entity).despawn();
		}
	}
}

pub fn move_camera_buffers(
	time: Res<Time>,
	mut smugglers: ResMut<SharedDataSmuggler>,
	mut recorders: ResMut<ActiveRecorders>,
) {
	let dt = time.delta();
	let mut smugglers = smugglers.lock().unwrap();
	for (id, mut data) in smugglers.iter_mut() {
		if data.last_frame.is_none() {
			continue;
		}

		recorders.entry(*id).and_modify(|mut recorder| {
			let current_duration = recorder
				.frames
				.iter()
				.fold(Duration::ZERO, |total, frame| total + frame.frame_time);

			let mut next_duration = current_duration + dt;

			// If we're over budget, drop frames until we're under our target
			while next_duration > recorder.target_duration {
				if let Some(frame) = recorder.frames.pop_front() {
					next_duration -= frame.frame_time;
					drop(frame);
				} else {
					log::warn!(
						"Tried to discard excess frames from recorder {}, but there were no frames",
						id
					);
					break;
				}
			}

			recorder.frames.push_back(TextureFrame::with_duration(
				std::mem::replace(&mut data.last_frame, None)
					.expect("A frame has disappeared in Lego City"),
				dt,
			));
		});
	}
}

pub fn start_tracking_orthographic_camera(
	mut commands: Commands,
	mut events: ResMut<Events<StartTrackingCamera>>,
	mut images: ResMut<Assets<Image>>,
	mut smugglers: ResMut<SharedDataSmuggler>,
	mut recorders: ResMut<ActiveRecorders>,
	query: Query<(&Camera, &Transform, &OrthographicProjection)>,
) {
	for event in events.drain() {
		if let Ok((camera, transform, ortho)) = query.get(event.cam_entity) {
			let target_image = ortho.project_to_image();
			let target_handle = images.add(target_image);
			let new_id = event.tracking_id;

			let tracker_entity = commands
				.spawn_bundle(Camera2dBundle {
					transform: transform.clone(),
					projection: ortho.clone(),
					camera: Camera {
						target: RenderTarget::Image(target_handle.clone()),
						..camera.clone()
					},
					..Default::default()
				})
				.insert(Recorder(event.tracking_id))
				.insert(Track(event.cam_entity))
				.id();

			let mut smuggle = smugglers
				.lock()
				.expect("Smugglers have gone; Poisoned Mutex");

			smuggle.insert(
				new_id,
				RenderData {
					target_handle: target_handle.clone(),
					last_frame: None,
				},
			);

			recorders.insert(
				new_id,
				ActiveRecorder {
					target_handle,
					target_duration: event.length,
					frames: Default::default(),
					tracker: tracker_entity,
				},
			);
		}
	}
}
