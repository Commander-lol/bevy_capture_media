// ----------------------
//     NOTICE
//
//   PAY ATTENTION
//
//  This example will crash:
//
//  - Setting a viewport on a camera causes the camera's image
//    to differ from that expected by the tracker. This is
//    not currently factored in
//
// ----------------------

use std::time::Duration;

use bevy::prelude::*;
use bevy_capture_media::{BevyCapturePlugin, MediaCapture};
use bevy_render::camera::Viewport;

// Tracking IDs are just arbitrary numbers that you use to interact with
// a specific tracker
const TRACKER_ONE: usize = 736298;
const TRACKER_TWO: usize = 193;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugin(BevyCapturePlugin)
		.insert_resource(WindowDescriptor {
			width: 800.0,
			height: 600.0,
			title: String::from("Multiple Cameras Example"),
			..Default::default()
		})
		.add_startup_system(spawn_first_camera)
		.add_startup_system(spawn_second_camera)
		.add_startup_system(spawn_scene)
		// .add_system(take_screenshot)
		.run();
}

/// Set up basic entities for our example
pub fn spawn_first_camera(mut commands: Commands, mut capture: MediaCapture) {
	let viewport = Viewport {
		physical_size: UVec2::new(200, 200),
		physical_position: UVec2::new(100, 100),
		..Default::default()
	};

	// You need to get the `Entity` of the camera you want to track. This could also come from a
	// query at a later point in time
	let camera_entity = commands
		.spawn_bundle(Camera2dBundle {
			camera: Camera {
				viewport: Some(viewport),
				..Camera::default()
			},

			..Camera2dBundle::default()
		})
		.id();

	// capture.start_tracking_camera(TRACKER_ONE, camera_entity, Duration::from_secs(2));
}
/// Set up basic entities for our example
pub fn spawn_second_camera(mut commands: Commands, mut capture: MediaCapture) {
	let viewport = Viewport {
		physical_size: UVec2::new(200, 200),
		physical_position: UVec2::new(500, 300),
		..Default::default()
	};
	// You need to get the `Entity` of the camera you want to track. This could also come from a
	// query at a later point in time
	let camera_entity = commands
		.spawn_bundle(Camera2dBundle {
			camera: Camera {
				viewport: Some(viewport),
				..Camera::default()
			},
			..Camera2dBundle::default()
		})
		.id();
	// Tracking cameras do not need to track the same amount of time
	// capture.start_tracking_camera(TRACKER_TWO, camera_entity, Duration::from_secs(5));
}

pub fn spawn_scene(mut commands: Commands, assets: Res<AssetServer>) {
	commands
		.spawn_bundle(NodeBundle {
			style: Style {
				size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				..Default::default()
			},
			..Default::default()
		})
		.with_children(|children| {
			children.spawn_bundle(TextBundle {
				text: Text::from_section(
					"Take a screenshot with right shift!",
					TextStyle {
						color: Color::BLACK,
						font_size: 48.0,
						font: assets.load("KenneyBlocks.ttf"),
					},
				),
				..Default::default()
			});
		});
}
