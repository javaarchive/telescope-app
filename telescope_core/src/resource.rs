use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MemoryResource {
    pub buffer: Vec<u8>
}

impl MemoryResource {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self {
            buffer
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileResource {
    pub path: String
}

impl FileResource {
    pub fn new(path: &str) -> Self {
        Self {
            path: String::from(path)
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StringResource {
    pub string: String
}

impl StringResource {
    pub fn new(string: &str) -> Self {
        Self {
            string: String::from(string)
        }
    }

    pub fn new_from_string(string: String) -> Self {
        Self {
            string
        }
    }
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

    pub fn empty() -> Self {
        Self::Memory(MemoryResource::new(Vec::new()))
    }
}

pub trait ResolveString {
    fn resolve_string(&self, resource: &Resource) -> String;
}

impl ResolveString for Config {
    fn resolve_string(&self, resource: &Resource) -> String {
        match resource {
            Resource::Memory(memory_resource) => {
                if memory_resource.buffer.is_empty() {
                    return "".to_string();
                }
                return String::from_utf8(memory_resource.buffer.clone()).unwrap();
            },
            Resource::File(file_resource) => {
                return std::fs::read_to_string(self.data_dir.join(file_resource.path.clone())).expect("File resource read failed");
            },
            Resource::String(string_resource) => {
                return string_resource.string.clone();
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestOrResponse {
    pub body: Resource,
    pub headers: Vec<(String, String)>,
    pub is_response: bool,
    pub reply: Option<Box<RequestOrResponse>>
}

impl RequestOrResponse {
    pub fn get_header(&self, header: &str) -> Option<String> {
        // supermaven wrote this, pretty nice rust
        self.headers.iter().find_map(|(k, v)| {
            if k == header {
                Some(v.clone())
            } else {
                None
            }
        })
    }

    pub fn has_reply(&self) -> bool {
        self.reply.is_some()
    }
}


#[derive(Debug, Clone)]
pub struct HTTPPair {
    pub request: RequestOrResponse,
    pub response: Option<RequestOrResponse>,
}

#[derive(Debug, Clone)]
pub enum FlowContent {
    RequestResponse(HTTPPair)
    // TODO: websocket?
}

#[derive(Debug, Clone)]
pub struct Flow {
    pub id: String,
    pub content: FlowContent,
    pub is_active: bool,
}

impl Flow {
    pub fn new(content: FlowContent) -> Self {
        Self {
            id: nanoid::nanoid!(), // TODO: restrict charset for id
            content,
            is_active: true
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}
