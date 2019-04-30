use serde_json::Value;
use std::string::String;

#[derive(Debug, PartialEq)]
pub enum Command {
    FindImage {
        path: String,
    },
    ResizeImage {
        source: String,
        sizes: Vec<(String, u32, u32)>,
    },
    WrongCommand,
}

pub fn parse_json_string(json: String) -> Command {
    match serde_json::from_str(&json) {
        Ok(value) => create_command_from_parsed_json_value(value),
        _ => Command::WrongCommand
    }
}

fn create_command_from_parsed_json_value(json: Value) -> Command {
    match json["type"].as_str() {
        Some("find") => match &json["path"] {
            Value::String(path) => Command::FindImage {
                path: path.to_string()
            },
            _ => Command::WrongCommand
        },
        Some("resize") => Command::ResizeImage {
            source: String::from(json["source"].as_str().unwrap()),
            sizes: collect_image_sizes_from_json(json)
        },
        _ => Command::WrongCommand
    }
}

fn collect_image_sizes_from_json(json: Value) -> Vec<(String, u32, u32)> {
    match &json["sizes"] {
        Value::Array(list_sizes) => list_sizes
            .into_iter()
            .filter_map(parse_size_item)
            .collect(),
        _ => vec![]
    }
}

fn parse_size_item(size: &Value) -> Option<(String, u32, u32)> {
    match size.as_array() {
        Some(sizes) => match &sizes[..] {
            [target, width, height] if is_size_tuple(target, width, height) => Some((
                    String::from(target.as_str().unwrap()),
                    width.as_u64().unwrap() as u32,
                    height.as_u64().unwrap() as u32)),
            _ => None
        }
        _ => None
    }
}

fn is_size_tuple(target: &Value, width: &Value, height: &Value) -> bool {
    return target.is_string() && width.is_number() && height.is_number();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_find_image_command_from_json() {
        assert_eq!(
            Command::FindImage {
                path: String::from("/path/to/file")
            },
            parse_json_string(
                String::from("{\"type\":\"find\",\"path\":\"/path/to/file\"}")
            )
        )
    }

    #[test]
    fn when_type_is_wrong_creates_wrong_command() {
        assert_eq!(
            Command::WrongCommand,
            parse_json_string(
                String::from("{\"type\":\"something\"}")
            )
        )
    }

    #[test]
    fn when_type_is_not_provided_creates_wrong_command() {
        assert_eq!(Command::WrongCommand, parse_json_string(
            String::from("{}")
        ));
    }

    #[test]
    fn when_path_is_missing_for_find_command_creates_wrong_command() {
        assert_eq!(Command::WrongCommand, parse_json_string(
            String::from("{\"type\": \"find\"}")
        ));
    }

    #[test]
    fn when_resize_image_type_is_provided_creates_resize_image_command() {
        assert_eq!(
            Command::ResizeImage {
                source: String::from("path/to/image.jpg"),
                sizes: vec![
                    (String::from("path/to/320x400/image.jpg"), 320, 400)
                ],
            },
            parse_json_string(
                String::from(
                    "{\
                    \"type\":\"resize\", \
                    \"source\":\"path/to/image.jpg\", \
                    \"sizes\": [[\"path/to/320x400/image.jpg\", 320, 400]]\
                }"
                )
            )
        );
    }

    #[test]
    fn when_one_of_the_resize_items_is_invalid_it_ignores_it() {
        assert_eq!(
            Command::ResizeImage {
                source: String::from("path/to/image.jpg"),
                sizes: vec![
                    (String::from("path/to/320x400/image.jpg"), 320, 400),
                    (String::from("path/to/400x500/image.jpg"), 400, 500),
                ],
            },
            parse_json_string(
                String::from(
                    "{\
                    \"type\":\"resize\", \
                    \"source\":\"path/to/image.jpg\", \
                    \"sizes\": [\
                        [\"path/to/320x400/image.jpg\", 320, 400],\
                        [\"path/to/320x400/image.jpg\", 320],\
                        [\"path/to/400x500/image.jpg\", 400, 500]\
                    ]\
                }"
                )
            )
        );
    }

    #[test]
    fn when_empty_resize_list_creates_empty_resize_list_command () {
        assert_eq!(
            Command::ResizeImage {
                source: String::from("path/to/image.jpg"),
                sizes: vec![]
            },
            parse_json_string(
                String::from(
                    "{\
                    \"type\":\"resize\", \
                    \"source\":\"path/to/image.jpg\", \
                    \"sizes\": []\
                }"
                )
            )
        );
    }

    #[test]
    fn when_invalid_json_provided_creates_wrong_command_instance() {
        assert_eq!(Command::WrongCommand, parse_json_string(String::from("{daasdas}")));
    }
}