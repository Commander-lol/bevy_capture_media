// This example renders some simple text to the screen, and enables in-game screenshots by pressing
// right shift on the keyboard. The PNG will be saved with an autogenereted name to the directory that
// the `cargo run` command was issued from

use std::time::Duration;

use bevy::prelude::*;
use bevy_capture_media::{BevyCapturePlugin, MediaCapture};

/// Set up basic entities for our example
pub fn spawn_cameras(assets: Res<AssetServer>, mut commands: Commands, mut capture: MediaCapture) {
	// You need to get the `Entity` of the camera you want to track. This could also come from a
	// query at a later point in time
	let camera_entity = commands.spawn_bundle(Camera2dBundle::default()).id();

	// Tell 'MediaCapture' to track the camera that we just spawned. Setup happens in the PostUpdate
	// stage, so if you're tracking a camera that is spawned in the same frame, it must be spawned
	// in First, PreUpdate or Update
	capture.start_tracking_camera(1357, camera_entity, Duration::from_secs(5));

	// This just sets up something to appear in the screenshots
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

/// Handle keyboard input to capture a screenshot
pub fn take_screenshot(input: Res<Input<KeyCode>>, mut capture: MediaCapture) {
	if input.just_released(KeyCode::RShift) {
		// The identifier that we pass to `capture_png` is something that we
		// create ourselves. If you only ever have one tracker, you could store
		// it in a CONST. If you're dynamically tracking cameras, you'll need
		// to have your own logic for generating and storing identifiers
		capture.capture_png(1357);
	}
}

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugin(BevyCapturePlugin)
		.add_startup_system(spawn_cameras)
		.add_system(take_screenshot)
		.insert_resource(WindowDescriptor {
			width: 800.0,
			height: 600.0,
			title: String::from("Multiple Cameras Example"),
			..Default::default()
		})
		.run();
}
