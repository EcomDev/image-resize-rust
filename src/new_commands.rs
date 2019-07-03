use futures::Future;
use std::fmt::Debug;
use serde_json::Value;

trait Command: Debug {
    fn merge_with(self, other: Self) -> Box<Self>;

    fn hash_key(&self) -> String;

   // fn process_handler(&self)
   //                    -> Box<Future<Item = Event, Error = std::io::Error>>;
}

trait Event {
    fn to_string(&self) -> String;
}

#[derive(Debug, PartialEq)]
pub struct WrongCommand {
    reason: String
}

#[derive(Debug, PartialEq)]
pub struct FindImageCommand {
    path: String
}

#[derive(Debug, PartialEq)]
pub struct ImageTarget(String, u32, u32);

#[derive(Debug, PartialEq)]
pub struct ResizeImageCommand {
    path: String,
    targets: Vec<ImageTarget>
}

enum Commands {
    WrongCommand {
        reason: String
    }
}

impl Commands::WrongCommand {
    fn new(reason: String) -> Self {
        Commands::WrongCommand { reason }
    }
}

impl Command for WrongCommand {
    fn merge_with(self, _: WrongCommand) -> WrongCommand {
        self
    }

    fn hash_key(&self) -> String {
        format!("WrongCommand::{}", self.reason)
    }
}

impl FindImageCommand {
    fn new(path: String) -> Self {
        FindImageCommand { path }
    }
}

impl Command for FindImageCommand {
    fn merge_with(self, other: FindImageCommand) -> FindImageCommand {
        self
    }

    fn hash_key(&self) -> String {
        format!("FindImageCommand::{}", self.path)
    }
}

impl ResizeImageCommand {
    fn new(path: String, targets: Vec<ImageTarget>) -> Self {
        ResizeImageCommand { path, targets }
    }
}

impl Command for ResizeImageCommand {
    fn merge_with(self, other: Self) -> Self {
        ResizeImageCommand {
            targets: self.targets
                .into_iter()
                .chain(other.targets.into_iter())
                .collect(),
            ..self
        }
    }

    fn hash_key(&self) -> String {
        format!("ResizeImageCommand::{}", self.path)
    }
}

pub fn create_command_from_json_string(json: String) -> impl Command
{
    match serde_json::from_str(&json) {
        Ok(value) => create_command_from_parsed_json_value(value),
        _ => create_wrong_command("")
    }
}

fn create_command_from_parsed_json_value(json: Value) -> impl Command {
    match json["command"].as_str() {
        Some("find") => match &json["path"] {
            Value::String(path) => create_find_image_command(path.to_string()),
            _ => create_wrong_command("")
        }
        _ =>  create_wrong_command("")
    }
}

fn create_find_image_command(path: String) -> impl Command {
    FindImageCommand::new(path)
}

fn create_wrong_command(reason: &str) -> impl Command {
    WrongCommand::new(reason.into())
}

#[cfg(test)]
mod wrong_command {

    use super::*;

    #[test]
    fn does_not_merge_with_other_wrong_commands() {
        let command = WrongCommand::new("Reasons".into());

        assert_eq!(
            WrongCommand::new("Reasons".into()),
            command.merge_with(WrongCommand::new("Something Else".into()))
        );
    }

    #[test]
    fn returns_reason_as_a_hash_key () {
       assert_eq!(
           "WrongCommand::Some Reason Value",
           WrongCommand::new("Some Reason Value".into()).hash_key()
       );
    }
}

#[cfg(test)]
mod find_image
{
    use super::*;

    #[test]
    fn returns_same_command_after_merging_command_with_itself () {
        let command = FindImageCommand::new("/some/file/path.jpg".into());

        assert_eq!(
            FindImageCommand::new("/some/file/path.jpg".into()),
            command.merge_with(FindImageCommand::new("/some/file/path2.jpg".into()))
        );
    }

    #[test]
    fn returns_file_path_as_a_hash_key () {
        let command = FindImageCommand::new("/very/nice/file/path.jpg".into());

        assert_eq!(
            "FindImageCommand::/very/nice/file/path.jpg",
            command.hash_key()
        );
    }
}

#[cfg(test)]
mod image_resize
{
    use crate::new_commands::{ResizeImageCommand, ImageTarget, Command};

    #[test]
    fn merges_targets_from_another_command () {
        let command = ResizeImageCommand::new(
            "/some/path.jpg".into(),
            vec![ImageTarget("/some/path-100x200.jpg".into(), 100, 200)]
        );

       assert_eq!(
           ResizeImageCommand::new(
               "/some/path.jpg".into(),
               vec![
                   ImageTarget("/some/path-100x200.jpg".into(), 100, 200),
                   ImageTarget("/some/path-200x300.jpg".into(), 200, 300),
               ]
           ),
           command.merge_with(
               ResizeImageCommand::new(
                   "/some/path.jpg".into(),
                   vec![ImageTarget("/some/path-200x300.jpg".into(), 200, 300)]
               )
           )
       );
    }

    #[test]
    fn returns_source_image_path_as_hash_key () {
       assert_eq!(
           "ResizeImageCommand::/image/path.jpg",
           ResizeImageCommand::new("/image/path.jpg".into(), vec![]).hash_key()
       );
    }
}

#[cfg(test)]
mod factory {
    use super::*;

    fn create_command (json: &str) -> String {
        format!("{:?}", create_command_from_json_string(json.into()))
    }

    #[test]
    fn given_invalid_json_string_creates_wrong_command_with_info_on_wrong_json_structure() {
       assert_eq!(
          r#"WrongCommand { reason: "Invalid JSON structure is provided" }"#,
          create_command("{,}")
       );
    }

    #[test]
    fn creates_find_image_command_from_json() {
        assert_eq!(
            r#"FindImageCommand { path: "/path/to/file" )"#,
            create_command(
                "{\"command\":\"find\",\"path\":\"/path/to/file\"}".into()
            )
        )
    }

    #[test]
    fn when_type_is_wrong_creates_wrong_command() {
        assert_eq!(
            r#"WrongCommand { reason: "Unkown command type provided" }"#,
            create_command(
                "{\"command\":\"something\"}"
            )
        )
    }

}
