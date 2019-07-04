use futures::Future;
use std::fmt::Debug;
use serde_json::{json, Value, to_string as json_to_string};

pub trait Command {
    fn process_handler(&self) -> Box<Future<Item=Event, Error=std::io::Error>>;
}

pub trait Event {
    fn as_json(&self) -> String;
}

type ImagePath = String;
type WrongInputDescription = String;
type ImageResizeFailureReason = String;
type Width = u32;
type Height = u32;

struct ImageSize(Width, Height);

struct ImageTarget(ImagePath, ImageSize);

enum Commands {
    InvalidInput(WrongInputDescription),
    FindImage(ImagePath),
    ResizeImage(ImagePath, Vec<ImageTarget>),
}

enum Events {
    InvalidInputProvided(WrongInputDescription),
    ImageFound(ImagePath),
    ImageNotFound(ImagePath),
    ImageResizeCompleted(ImageTarget),
    ImageResizeFailed(ImageTarget, ImageResizeFailureReason),
}

impl Command for Commands
{
    fn process_handler(&self) -> Box<Future<Item=Event, Error=std::io::Error>> {
        unimplemented!()
    }
}

impl Event for Events {
    fn as_json(&self) -> String {
        match self {
            Events::InvalidInputProvided(reason) => json_event_invalid_input(reason),
            Events::ImageFound(path) => json_event_image_found(path),
            Events::ImageNotFound(path) => json_event_image_not_found(path),
            Events::ImageResizeCompleted(target)
                => json_event_image_resize_completed(target),
            Events::ImageResizeFailed(target, reason)
                => json_event_image_resize_failed(target, reason),
        }
    }
}

fn json_event_invalid_input(reason: &WrongInputDescription) -> String {
    json_to_string(&json!({
        "event": "invalid_input",
        "reason": reason
    })).unwrap()
}

fn json_event_image_found(path: &ImagePath) -> String {
    json_to_string(&json!({
        "event": "image_found",
        "path": path
    })).unwrap()
}

fn json_event_image_not_found(path: &ImagePath) -> String {
    json_to_string(&json!({
        "event": "image_not_found",
        "path": path
    })).unwrap()
}

fn json_event_image_resize_completed(target: &ImageTarget) -> String {
    let ImageTarget(path, ImageSize(width, height)) = target;
    json_to_string(&json!({
        "event": "image_resize_completed",
        "path": path,
        "size": [width, height]
    })).unwrap()
}

fn json_event_image_resize_failed(target: &ImageTarget, reason: &ImageResizeFailureReason) -> String {
    let ImageTarget(path, ImageSize(width, height)) = target;
    json_to_string(&json!({
        "event": "image_resize_failed",
        "path": path,
        "size": [width, height],
        "reason": reason
    })).unwrap()
}

#[cfg(test)]
mod event_serialization {
    use super::*;

    #[test]
    fn serializes_invalid_input_event_as_json() {
        assert_eq!(
            r#"{"event":"invalid_input","reason":"Can't parse input line as json"}"#,
            Events::InvalidInputProvided("Can't parse input line as json".into()).as_json()
        );
    }

    #[test]
    fn serializes_image_found_event_as_json() {
        assert_eq!(
            r#"{"event":"image_found","path":"/path/to/image.jpg"}"#,
            Events::ImageFound("/path/to/image.jpg".into()).as_json()
        )
    }

    #[test]
    fn serializes_image_not_found_event_as_json() {
        assert_eq!(
            r#"{"event":"image_not_found","path":"/path/to/image.jpg"}"#,
            Events::ImageNotFound("/path/to/image.jpg".into()).as_json()
        )
    }

    #[test]
    fn serializes_image_resize_completed_as_json() {
        assert_eq!(
            r#"{"event":"image_resize_completed","path":"/path/to/image1.jpg","size":[300,200]}"#,
            Events::ImageResizeCompleted(
                ImageTarget(
                    "/path/to/image1.jpg".into(),
                    ImageSize(300, 200),
                )
            ).as_json()
        );
    }

    #[test]
    fn serializes_image_resize_failed_as_json() {
        assert_eq!(
            r#"{"event":"image_resize_failed","path":"/path/to/image1.jpg","reason":"Failed!","size":[300,200]}"#,
            Events::ImageResizeFailed(
                ImageTarget(
                    "/path/to/image1.jpg".into(),
                    ImageSize(300, 200),
                ),
                "Failed!".into()
            ).as_json()
        );
    }
}