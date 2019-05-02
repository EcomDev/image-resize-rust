# Image Resize Worker

Async image resize socket server written in Rust to offload reactive PHP image server. 



## Commands

In order to communicate between Rust and PHP it was chosen to use JSON encoded line protocol. 

Each line send via socket client is a command. Server responds back with an event that was a result of the command execution. There is no uuid for commands, there file paths serve as unique identifiers.

### Find a file
In order to make sure we have efficient lookup of existing image files socket server implements find command, where `path` is a full path to a file in a file system.
        
```json
{"command": "find", "path": "/path/to/a/file"}
```
There are two possible events produced by this command, `found` nd `not_found` with obvious meaning.

```json
{"event": "found", "path": "/path/to/a/file"}
```

```json
{"event": "not_found", "path": "/path/to/a/file"}
```  

### Resize Image

PHP application sends `resize` commands to request Rust server resize an image in various sizes. Each size variation is a tuple of target path (string), width (int) and height (int) in `sizes` array.

```json
{"command":"resize", "source": "path/to/image.jpg", "sizes": [["path/to/320x400/image.jpg", 320, 400]]}
```

Each size variation will receive back an event of `resize_completed` in case of success
```json
{"event":"resize_completed", "target": "path/to/320x400/image.jpg"}
``` 
And `resize_failed` when something went wrong
```json
{"event":"resize_failed", "target": "path/to/320x400/image.jpg", "reason": "Cannot open source file"}
```
