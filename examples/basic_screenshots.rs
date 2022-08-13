use std::time::Duration;

use bevy::prelude::*;
use bevy_capture_media::{BevyCapturePlugin, MediaCapture};

pub fn spawn_cameras(assets: Res<AssetServer>, mut commands: Commands, mut capture: MediaCapture) {
	let camera_entity = commands.spawn_bundle(Camera2dBundle::default()).id();
	capture.start_tracking_camera(1357, camera_entity, Duration::from_secs(5));
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

pub fn take_screenshot(input: Res<Input<KeyCode>>, mut capture: MediaCapture) {
	if input.just_released(KeyCode::RShift) {
		capture.capture_png(1357);
	}
}

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugin(BevyCapturePlugin)
		.add_startup_system(spawn_cameras)
		.add_system(take_screenshot)
		.run();
}
