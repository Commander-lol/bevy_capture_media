use std::path::{Path, PathBuf};
use std::rc::Rc;

use js_sys::{Array, Date, Promise, Uint8Array};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, Document, FileReader, Window};

macro_rules! null_return {
	($nullable: expr, $message: literal) => {
		match $nullable {
			Some(value) => value,
			None => {
				log::error!($message);
				return;
			}
		}
	};
	($nullable: expr, $message: literal, $on_error: expr) => {
		match $nullable {
			Some(value) => value,
			None => {
				log::error!($message);
				$on_error.call1(&JsValue::NULL, &JsValue::from_str($message));
				return;
			}
		}
	};
}
macro_rules! err_return {
	($nullable: expr, $message: literal) => {
		match $nullable {
			Ok(value) => value,
			Err(e) => {
				log::error!("{}; {:?}", $message, e);
				return;
			}
		}
	};
	($nullable: expr, $message: literal, $on_error: expr) => {
		match $nullable {
			Ok(value) => value,
			Err(e) => {
				log::error!("{}; {:?}", $message, e);
				$on_error.call1(&JsValue::NULL, &e);
				return;
			}
		}
	};
}

fn read_to_data(blob: &Blob) -> Promise {
	Promise::new(&mut |resolve, reject| {
		let file_reader: FileReader =
			err_return!(FileReader::new(), "Failed to create a file reader", reject);

		let reject = Rc::new(reject);
		let file_reader = Rc::new(file_reader);

		let closure_file_reader = file_reader.clone();
		let closure_reject = reject.clone();
		let mut closure = Closure::once(move || {
			let value = err_return!(
				closure_file_reader.result(),
				"Could not get file reader result",
				closure_reject
			);
			resolve.call1(&JsValue::NULL, &value);
		});

		file_reader.set_onload(Some(closure.as_ref().unchecked_ref()));
		closure.forget();
		err_return!(
			file_reader.read_as_data_url(blob),
			"Failed to read the image data as a data url",
			reject
		);
	})
}

pub fn focus_on_first_of_type(element_type: &str) {
	let window: Window = null_return!(
		web_sys::window(),
		"Didn't find a window to attach to while saving screenshot"
	);
	let document: Document = null_return!(
		window.document(),
		"Window did not contain a document to attach to while saving screenshot"
	);
	let element = null_return!(
		document.get_elements_by_tag_name(element_type).item(0),
		"Could not find element to focus on"
	);

	err_return!(
		element.dyn_into::<web_sys::HtmlElement>(),
		"Failed to attach blob url"
	)
	.focus();
}

async fn download_bytes_inner(file_name: PathBuf, bytes: Vec<u8>) {
	let bytes = bytes.as_slice();
	let js_byte_array = Uint8Array::from(bytes);
	let js_array = Array::new();
	js_array.push(&js_byte_array.buffer());

	let blob = err_return!(
		Blob::new_with_u8_array_sequence_and_options(
			&js_array,
			BlobPropertyBag::new().type_("image/png"),
		),
		"Failed to create screenshot blob data"
	);

	let obj_url = err_return!(
		JsFuture::from(read_to_data(&blob)).await,
		"Failed to create data url for screenshot"
	);
	let obj_url_string = null_return!(
		obj_url.as_string(),
		"Couldn't convert the data url from a JsValue to a String"
	);
	let window: Window = null_return!(
		web_sys::window(),
		"Didn't find a window to attach to while saving screenshot"
	);
	let document: Document = null_return!(
		window.document(),
		"Window did not contain a document to attach to while saving screenshot"
	);
	let link = err_return!(
		document.create_element("a"),
		"Could not create download link handler"
	);
	err_return!(
		link.set_attribute("href", obj_url_string.as_str()),
		"Failed to attach blob url"
	);
	err_return!(
		link.set_attribute("download", format!("{}", file_name.display()).as_str()),
		"Failed to attach file name"
	);
	let html_link = err_return!(
		link.dyn_into::<web_sys::HtmlElement>(),
		"Could not get interactable version of download link"
	);

	html_link.click();

	log::info!("Saving image to path {}", file_name.display());
}

pub fn download_bytes(file_name: PathBuf, bytes: Vec<u8>) {
	wasm_bindgen_futures::spawn_local(download_bytes_inner(file_name, bytes));
}

pub fn get_now() -> f64 {
	Date::now()
}
