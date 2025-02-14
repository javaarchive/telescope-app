use std::{io::Cursor, time::{Instant, SystemTime, UNIX_EPOCH}};

use http_body_util::{BodyExt, BodyStream, Collected};
use hudsucker::{rustls::version, tokio_tungstenite::tungstenite::http::request};
use hyper::HeaderMap;
use log::warn;
use serde::{de, Deserialize, Serialize, Serializer};

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

// https://stackoverflow.com/questions/39383809/how-to-transform-fields-during-serialization-using-serde
fn url_serialize<S>(url: &reqwest::Url, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let url_str = url.as_str();
    s.serialize_str(url_str)
}

// https://users.rust-lang.org/t/need-help-with-serde-deserialize-with/18374/3
fn deserialize_json_string<'de, D>(deserializer: D) -> Result<reqwest::Url, D::Error>
where
    D: de::Deserializer<'de>,
{
    let url_str: &str = de::Deserialize::deserialize(deserializer)?;
    reqwest::Url::parse(url_str).map_err(de::Error::custom)
}

pub fn get_current_time() -> u128 {
    // TODO: fix non-monotonic func use?
    SystemTime::now().duration_since(UNIX_EPOCH).expect("time keeping failure").as_millis()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMeta {
    #[serde(serialize_with = "url_serialize", deserialize_with = "deserialize_json_string")]
    pub url: reqwest::Url,
    pub method: String,
    pub version: String,
    pub created_at: u128,
}

impl RequestMeta {
    pub fn new(url: &str, method: &str, version: &str) -> Self {
        Self {
            url: reqwest::Url::parse(url).unwrap(), // TODO: error handling
            method: String::from(method),
            version: String::from(version),
            created_at: get_current_time()
        }
    }

    pub fn url_str(&self) -> &str {
        self.url.as_str()
    }

    pub fn is_proxy_client_connection(&self) -> bool {
        self.method == "CONNECT"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub status: u32,
    pub version: String,
    pub created_at: u128
}

impl ResponseMeta {
    pub fn new(status: u32, version: &str) -> Self {
        Self {
            status,
            version: String::from(version),
            created_at: get_current_time()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestOrResponseMeta {
    Request(RequestMeta),
    Response(ResponseMeta)
}

impl RequestOrResponseMeta {
    pub fn unwrap_request_ref(&self) -> &RequestMeta {
        match self {
            RequestOrResponseMeta::Request(request_meta) => request_meta,
            RequestOrResponseMeta::Response(response_meta) => panic!("ResponseMeta cannot be unwrapped as RequestMeta")
        }
    }

    pub fn unwrap_response_ref(&self) -> &ResponseMeta {
        match self {
            RequestOrResponseMeta::Request(request_meta) => panic!("RequestMeta cannot be unwrapped as ResponseMeta"),
            RequestOrResponseMeta::Response(response_meta) => response_meta
        }
    }
}

impl Into<RequestMeta> for RequestOrResponseMeta {
    fn into(self) -> RequestMeta {
        match self {
            RequestOrResponseMeta::Request(request_meta) => request_meta,
            RequestOrResponseMeta::Response(response_meta) => panic!("ResponseMeta cannot be converted to RequestMeta")
        }
    }
}

impl Into<ResponseMeta> for RequestOrResponseMeta {
    fn into(self) -> ResponseMeta {
        match self {
            RequestOrResponseMeta::Request(request_meta) => panic!("RequestMeta cannot be converted to ResponseMeta"),
            RequestOrResponseMeta::Response(response_meta) => response_meta
        }
    }
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
        let request_to_save = RequestOrResponse::new_request(Resource::Memory(MemoryResource::new(body_bytes.to_vec())), headers, RequestMeta::new(&url, &method, &version_str));

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
        let response_to_save = RequestOrResponse::new_response(Resource::Memory(MemoryResource::new(body_bytes.to_vec())), headers, ResponseMeta::new(status, &version_str));

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

    pub fn get_time_taken(&self) -> Option<u128> {
        if let Some(response) = &self.response {
            Some(response.meta.unwrap_response_ref().created_at - self.request.meta.unwrap_request_ref().created_at)
        } else {
            None
        }
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
