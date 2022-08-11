# bevy_capture_media
Event based image &amp; video capture for Bevy

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
- More formats
- More control over frame smuggling

-- A cool GIF will go here, showing the library in action. Just you wait! --

## Contributing

All suggestions, issues, and pull requests are welcome. Any contributions must
be licensed compatibly with this repository.

### I want to add a new format!

Yes! This is great! There is a small checklist that any new format will need to meet to be included:

- Encoding must happen in pure rust. No compiled C libraries, no bindings to other languages.
- The format must perform well for a web target - if there is a very good reason to circumvent this, the format may still be accepted.
- By default, the format and any dependencies must be optional and opt-in. 
- No patent-encumbered or closed formats will be accepted unless there is a free and permissive license grant for this project and any end users.