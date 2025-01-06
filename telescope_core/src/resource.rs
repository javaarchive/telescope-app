use std::io::Cursor;

use http_body_util::{BodyExt, BodyStream, Collected};
use hudsucker::{rustls::version, tokio_tungstenite::tungstenite::http::request};
use hyper::HeaderMap;
use log::warn;
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
                return std::fs::read_to_string(self.data_dir.join(file_resource.path.clone())).expect(format!("File resource read failed: {}", file_resource.path.clone()).as_str());
            },
            Resource::String(string_resource) => {
                return string_resource.string.clone();
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMeta {
    pub url: String,
    pub method: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub status: u32,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestOrResponseMeta {
    Request(RequestMeta),
    Response(ResponseMeta)
}

#[derive(Debug, Clone)]
pub struct RequestOrResponse {
    pub body: Resource,
    pub headers: HeaderMap,
    pub is_response: bool,
    pub meta: RequestOrResponseMeta
    // pub reply: Option<Box<RequestOrResponse>>
}

impl RequestOrResponse {
    pub fn new_request(body: Resource, headers: HeaderMap, meta: RequestMeta) -> Self {
        Self {
            body,
            headers,
            is_response: false,
            meta: RequestOrResponseMeta::Request(meta)
        }
    }

    pub fn new_response(body: Resource, headers: HeaderMap, meta: ResponseMeta) -> Self {
        Self {
            body,
            headers,
            is_response: true,
            meta: RequestOrResponseMeta::Response(meta)
        }
    }
}


pub fn version_to_string(version: hyper::Version) -> String {
    match version {
        hyper::Version::HTTP_09 => "HTTP/0.9".to_string(),
        hyper::Version::HTTP_10 => "HTTP/1.0".to_string(),
        hyper::Version::HTTP_11 => "HTTP/1.1".to_string(),
        hyper::Version::HTTP_2 => "HTTP/2".to_string(),
        hyper::Version::HTTP_3 => "HTTP/3".to_string(),
        _ => {
            {
                warn!("Unsupported HTTP version {:?}", version);
                #[cfg(feature = "strict")]
                panic!("Unsupported HTTP version {:?}", version);
            }

            "HTTP/1.0".to_string()
        }
    }
}

impl RequestOrResponse {
    /*pub fn has_reply(&self) -> bool {
        self.reply.is_some()
    }*/

    // code from https://github.com/sinKettu/cruster/blob/0238047e713624b17942ad18fb9a9a9d136ab8f2/src/cruster_proxy/request_response.rs#L84
    // note hyper reexports a lot of http stuff so the types are essentially the same
    // TODO: maybe better to use hyper only version to prevent version conflicts
    pub async fn copy_request(request: hyper::Request<hudsucker::Body>) -> (RequestOrResponse, hyper::Request<hudsucker::Body>) {
        let (parts, body) = request.into_parts();
        let url = parts.uri.clone().to_string();
        let method = parts.method.clone().to_string();
        let headers = parts.headers.clone();
        let version_str = version_to_string(parts.version);

        let body_collected =  body.collect().await.unwrap_or_default();
        let body_bytes = body_collected.to_bytes();
        // let cursor = Cursor::new(body_bytes.clone().to_vec());
        let body_cloned: hudsucker::Body = hudsucker::Body::from(http_body_util::Full::new(body_bytes.clone()));

        let duplicated_request = hyper::Request::from_parts(parts, body_cloned);
        let request_to_save = RequestOrResponse::new_request(Resource::Memory(MemoryResource::new(body_bytes.to_vec())), headers, RequestMeta { url, method, version: version_str });

        (request_to_save, duplicated_request)
    }

    pub async fn copy_response(response: hyper::Response<hudsucker::Body>) -> (RequestOrResponse, hyper::Response<hudsucker::Body>) {
        let (parts, body) = response.into_parts();
        let body_collected = body.collect().await.unwrap_or_default();
        let body_bytes = body_collected.to_bytes();
        let body_cloned: hudsucker::Body = hudsucker::Body::from(http_body_util::Full::new(body_bytes.clone()));
        let status = parts.status.as_u16() as u32;
        let version_str = version_to_string(parts.version.clone());
        let headers = parts.headers.clone();

        let duplicated_response = hyper::Response::from_parts(parts, body_cloned);
        let response_to_save = RequestOrResponse::new_response(Resource::Memory(MemoryResource::new(body_bytes.to_vec())), headers, ResponseMeta { status: status, version: version_str });

        (response_to_save, duplicated_response)
    }
}


#[derive(Debug, Clone)]
pub struct HTTPPair {
    pub request: RequestOrResponse,
    pub response: Option<RequestOrResponse>,
}

impl HTTPPair {
    pub fn has_response(&self) -> bool {
        self.response.is_some()
    }

    pub fn new_request(request: RequestOrResponse) -> Self {
        Self {
            request,
            response: None
        }
    }

    pub fn add_response(&mut self, response: RequestOrResponse) {
        self.response = Some(response);
    }
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
