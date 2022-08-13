// ----------------------
//     NOTICE
//
//   PAY ATTENTION
//
//  This example does not currently work
//
// ----------------------

use std::f32::consts::{FRAC_PI_4, TAU};
use std::time::Duration;

use bevy::prelude::*;
use bevy_capture_media::{BevyCapturePlugin, MediaCapture};

// Tracking IDs are just arbitrary numbers that you use to interact with
// a specific tracker. You can generate and store them in whatever way
// makes sense for your usecase
const TRACKING_ID: usize = 736298;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_plugin(BevyCapturePlugin)
		.add_startup_system(spawn_camera)
		.add_startup_system(spawn_scene)
		.add_system(rotate_cam)
		.add_system(take_screenshot)
		.insert_resource(WindowDescriptor {
			width: 800.0,
			height: 600.0,
			title: String::from("Multiple Cameras Example"),
			..Default::default()
		})
		.run();
}

/// Handle keyboard input to capture a screenshot
pub fn take_screenshot(input: Res<Input<KeyCode>>, mut capture: MediaCapture) {
	if input.just_released(KeyCode::RShift) {
		// The identifier that we pass to `capture_png` is something that we
		// create ourselves. If you only ever have one tracker, you could store
		// it in a CONST. If you're dynamically tracking cameras, you'll need
		// to have your own logic for generating and storing identifiers
		capture.capture_png(TRACKING_ID);
	}
}

pub fn spawn_camera(mut commands: Commands, mut capture: MediaCapture) {
	// You need to get the `Entity` of the camera you want to track. This could also come from a
	// query at a later point in time
	let camera_entity = commands
		.spawn_bundle(Camera3dBundle {
			transform: Transform::from_xyz(5., 2., 6.)
				.looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
			..default()
		})
		.id();

	capture.start_tracking_camera(TRACKING_ID, camera_entity, Duration::from_secs(5));
}

pub fn rotate_cam(time: Res<Time>, mut query: Query<&mut Transform, With<DirectionalLight>>) {
	for mut transform in query.iter_mut() {
		transform.rotation = Quat::from_euler(
			EulerRot::ZYX,
			0.0,
			time.seconds_since_startup() as f32 * TAU / 10.0,
			-FRAC_PI_4,
		);
	}
}

pub fn spawn_scene(
	mut commands: Commands,
	assets: Res<AssetServer>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut capture: MediaCapture,
) {
	const HALF_SIZE: f32 = 1.0;
	commands.spawn_bundle(DirectionalLightBundle {
		directional_light: DirectionalLight {
			shadow_projection: OrthographicProjection {
				left: -HALF_SIZE,
				right: HALF_SIZE,
				bottom: -HALF_SIZE,
				top: HALF_SIZE,
				near: -10.0 * HALF_SIZE,
				far: 10.0 * HALF_SIZE,
				..default()
			},
			shadows_enabled: true,
			..default()
		},
		..default()
	});

	let tree_handle = assets.load("Tree.gltf#Scene0");

	let scene = commands.spawn_bundle(SceneBundle {
		scene: tree_handle,
		..Default::default()
	});

	commands
		.spawn_bundle(NodeBundle {
			style: Style {
				size: Size::new(Val::Percent(100.), Val::Percent(100.)),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::FlexStart,
				padding: UiRect::new(Val::Auto, Val::Auto, Val::Auto, Val::Px(50.)),
				..Default::default()
			},
			color: UiColor(Color::rgba(0., 0., 0., 0.)),
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
