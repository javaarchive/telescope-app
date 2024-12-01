pub struct MemoryResource {
    buffer: Vec<u8>
}

pub struct FileResource {
    path: String
}

pub enum Resource {
    Memory(MemoryResource),
    File(FileResource)
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