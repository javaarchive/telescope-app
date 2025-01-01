use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MemoryResource {
    pub buffer: Vec<u8>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileResource {
    pub path: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StringResource {
    pub string: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Resource {
    Memory(MemoryResource),
    File(FileResource),
    String(StringResource)
}

impl Resource {
    pub fn as_string(&self) -> String {
        match self {
            Resource::Memory(m) => String::from_utf8(m.buffer.clone()).unwrap(),
            Resource::File(f) => std::fs::read_to_string(f.path.clone()).expect("File resource read failed"),
            Resource::String(s) => s.string.clone()
        }
    }
}

pub struct Request {
    body: Resource
}

pub struct Response {
    body: Resource
}

pub struct HTTPPair {
    pub request: Request,
    pub response: Option<Response>,
}

pub enum Flow {
    RequestResponse(HTTPPair)
    // TODO: websocket?
}