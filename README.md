# bevy_capture_media
Event based image &amp; video capture for Bevy

[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue?style=for-the-badge)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![Crates.io](https://img.shields.io/crates/v/bevy_capture_media?style=for-the-badge)](https://crates.io/crates/bevy_capture_media)
[![docs.rs](https://img.shields.io/docsrs/bevy_capture_media?style=for-the-badge)](https://docs.rs/bevy_capture_media)

## Features
- Track any number of cameras for recording
- Dispatch events to control the recording lifecycle
- Keep a frame buffer of the past X frames for each recorder (Where X is any user supplied `Duration`)
- Pick and choose the formats you want to record with features
- `wasm` support

## Supported Formats
- PNG screenshots
- GIF recordings
    - _GIF Recordings are functional but require work_

## Roadmap
- Perspective camera support
- Support for viewports
- Support for resizing cameras
- More formats
- More control over frame smuggling
- Screenshot watermarks
- Improved web performance

## A Simple Example

```rust
use std::time::Duration;
use bevy::prelude::*;
use bevy_capture_media::{MediaCapture, BevyCapturePlugin};

pub fn spawn_cameras(
    mut commands: Commands,
    mut capture: MediaCapture,
) {
    let camera_entity = commands
        .spawn_bundle(Camera2dBundle::default())
        .id();
        
    // The tracking ID (1357) is arbitrary, but uniquely identifies this tracker
    capture.start_tracking_camera(1357, camera_entity, Duration::from_secs(5));
}

pub fn take_screenshot(
    input: Res<Input<KeyCode>>,
    mut capture: MediaCapture,
) {
    if input.just_released(KeyCode::RShift) {
        // If you have many cameras, consider storing their IDs
        // in a resource
        capture.capture_png(1357);
    }
}

fn main() {
    App::new()
        .add_plugin(DefaultPlugins)
        .add_plugin(bevy_capture_media::BevyCapturePlugin)
        .add_startup_system(spawn_cameras)
        .add_system(take_screenshot)
        .run();
}
```

https://user-images.githubusercontent.com/2522620/184448446-8cd5214b-81fa-41a3-bdbe-156412cc99cc.mp4

## Contributing

All suggestions, issues, and pull requests are welcome. Any contributions must
be licensed compatibly with this repository.

### I want to add a new format!

Yes! This is great! There is a small checklist that any new format will need to meet to be included:

- Encoding must happen in pure rust. No compiled C libraries, no bindings to other languages.
- The format must perform well for a web target - if there is a very good reason to circumvent this, the format may still be accepted.
- By default, the format and any dependencies must be optional and opt-in. 
- No patent-encumbered or closed formats will be accepted unless there is a free and permissive license grant for this project and any end users.

## Asset Licenses

The git repository contains some assets for the examples. They are licensed from their original creators with the
following licenses:

- **KenneyBlocks.tts**: [[CC0]](http://creativecommons.org/publicdomain/zero/1.0/), [Created/distributed by Kenney](www.kenney.nl) 
- **Tree.gltf**: [[CC0]](http://creativecommons.org/publicdomain/zero/1.0/), [Created/distributed by Kenney](www.kenney.nl) 