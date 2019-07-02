use serde_json::{json, to_string};

#[derive(Debug, PartialEq)]
pub enum Event {
    ImageFound {
        path: String
    },
    ImageNotFound {
        path: String
    },
    ImageResizeComplete {
        target: String
    },
    ImageResizeFailed {
        target: String,
        reason: String,
    },
}

impl Event {
    pub fn found(path: &str) -> Event {
        Event::ImageFound { path: String::from(path) }
    }

    pub fn not_found(path: &str) -> Event {
        Event::ImageNotFound { path: String::from(path) }
    }

    pub fn resize_complete(target: &str) -> Event {
        Event::ImageResizeComplete { target: String::from(target) }
    }

    pub fn resize_failed(target: &str, reason: &str) -> Event {
        Event::ImageResizeFailed { target: String::from(target), reason: String::from(reason) }
    }

    pub fn to_string(&self) -> String {
        let json = match self {
            Event::ImageFound { path } => json!({
                "event": "found",
                "path": path
            }),
            Event::ImageNotFound { path } => json!({
                "event": "not_found",
                "path": path
            }),
            Event::ImageResizeComplete { target } => json!({
                "event": "resize_completed",
                "target": target
            }),
            Event::ImageResizeFailed { target, reason } => json!({
                "event": "resize_failed",
                "target": target,
                "reason": reason
            }),
        };

        to_string(&json).unwrap()
    }
}

#[cfg(test)]
mod serializer {
    use super::Event;

    #[test]
    fn converts_event_for_found_image_into_json_string() {
        assert_eq!(
            String::from("{\"event\":\"found\",\"path\":\"/file/path.jpg\"}"),
            Event::found("/file/path.jpg").to_string()
        )
    }

    #[test]
    fn converts_event_for_not_found_image_into_json_string() {
        assert_eq!(
            String::from("{\"event\":\"not_found\",\"path\":\"/file/path2.jpg\"}"),
            Event::not_found("/file/path2.jpg").to_string()
        )
    }

    #[test]
    fn converts_event_completed_resize_into_json_string() {
        assert_eq!(
            String::from("{\"event\":\"resize_completed\",\"target\":\"/file/path3.jpg\"}"),
            Event::resize_complete("/file/path3.jpg").to_string()
        )
    }

    #[test]
    fn converts_event_failed_resize_into_json_string() {
        assert_eq!(
            String::from("{\"event\":\"resize_failed\",\"reason\":\"Something is wrong\",\"target\":\"/file/path4.jpg\"}"),
            Event::resize_failed("/file/path4.jpg", "Something is wrong").to_string()
        )
    }
}
