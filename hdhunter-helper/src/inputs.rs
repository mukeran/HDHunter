use hdhunter::input::{HttpInput, HttpSequenceInput, NodeType, Node};
use hdhunter::new_node;

#[derive(Debug)]
enum HttpInputBody {
    Identity {
        body: Vec<u8>
    },
    Chunked {
        chunks: Vec<(String, Option<String>, Vec<u8>)>,
        trailers: Vec<(String, String)>,
    },
    None
}

enum HttpInputFirstLine {
    RequestLine {
        method: String,
        path: String,
        version: String,
    },
    StatusLine {
        version: String,
        status_code: String,
        reason_phrase: String,
    }
}

pub struct HttpInputBuilder {
    first_line: Option<HttpInputFirstLine>,
    headers: Vec<(String, String)>,
    body: HttpInputBody,
}

fn split_bytes_on<'a>(bytes: &'a[u8], delimiter: &'a[u8]) -> Vec<&'a[u8]> {
    let mut result = Vec::new();
    let mut start = 0;

    while let Some(index) = bytes[start..]
        .windows(delimiter.len())
        .position(|window| window == delimiter)
    {
        result.push(&bytes[start..start + index]);
        start += index + delimiter.len();
    }

    result.push(&bytes[start..]);
    result
}

fn split_bytes_on_first<'a>(bytes: &'a[u8], delimiter: &'a[u8]) -> (&'a[u8], &'a[u8]) {
    let index = bytes.windows(delimiter.len()).position(|window| window == delimiter);
    if index.is_none() {
        return (bytes, &[]);
    }
    let index = index.unwrap();
    (&bytes[0..index], &bytes[index + delimiter.len()..])
}

fn trim_bytes(bytes: &[u8], byte_to_trim: u8) -> &[u8] {
    let start = bytes.iter().position(|&x| x != byte_to_trim).unwrap_or(bytes.len());
    let end = bytes.iter().rposition(|&x| x != byte_to_trim).map_or(start, |pos| pos + 1);
    &bytes[start..end]
}

impl HttpInputBuilder {
    pub fn new() -> Self {
        Self {
            first_line: None,
            headers: Vec::new(),
            body: HttpInputBody::None,
        }
    }

    pub fn request_line(mut self, method: &str, path: &str, version: &str) -> Self {
        self.first_line = Some(HttpInputFirstLine::RequestLine {
            method: method.to_string(),
            path: path.to_string(),
            version: version.to_string(),
        });
        self
    }

    pub fn status_line(mut self, version: &str, status_code: &str, reason_phrase: &str) -> Self {
        self.first_line = Some(HttpInputFirstLine::StatusLine {
            version: version.to_string(),
            status_code: status_code.to_string(),
            reason_phrase: reason_phrase.to_string(),
        });
        self
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    pub fn identity_body(mut self, body: &[u8]) -> Self {
        self.body = HttpInputBody::Identity { body: body.to_vec() };
        self
    }

    pub fn chunk(mut self, data: &[u8], extension: Option<&str>) -> Self {
        let entry = (
            format!("{:x}", data.len()),
            extension.map(|s| s.to_string()),
            data.to_vec(),
        );
        match &mut self.body {
            HttpInputBody::Identity { body: _body } => {
                self.body = HttpInputBody::Chunked {
                    chunks: vec![entry],
                    trailers: vec![],
                }
            }
            HttpInputBody::Chunked { ref mut chunks, trailers: _trailers } => {
                chunks.push(entry);
            }
            HttpInputBody::None => {
                self.body = HttpInputBody::Chunked {
                    chunks: vec![entry],
                    trailers: vec![],
                }
            }
        }
        self
    }

    pub fn trailer(mut self, name: &str, value: &str) -> Self {
        match &mut self.body {
            HttpInputBody::Identity { body: _body } => {
                self.body = HttpInputBody::Chunked {
                    chunks: vec![],
                    trailers: vec![(name.to_string(), value.to_string())],
                }
            }
            HttpInputBody::Chunked { chunks: _chunks, ref mut trailers } => {
                trailers.push((name.to_string(), value.to_string()))
            }
            HttpInputBody::None => {
                self.body = HttpInputBody::Chunked {
                    chunks: vec![],
                    trailers: vec![(name.to_string(), value.to_string())],
                }
            }
        }
        self
    }

    pub fn build(self) -> HttpInput {
        let mut nodes = vec![];
        nodes.push(new_node!(
            NodeType::StartLine,
            match self.first_line.unwrap() {
                HttpInputFirstLine::RequestLine { method, path, version } => {
                    new_node!(
                        NodeType::RequestLine,
                        new_node!(NodeType::RawBytes; method.into()),
                        new_node!(NodeType::SP; b" ".into()),
                        new_node!(NodeType::RawBytes; path.into()),
                        new_node!(NodeType::SP; b" ".into()),
                        new_node!(NodeType::RawBytes; version.into()),
                        new_node!(NodeType::CRLF; b"\r\n".into()),
                    )
                }
                HttpInputFirstLine::StatusLine { version, status_code, reason_phrase } => {
                    new_node!(
                        NodeType::StatusLine,
                        new_node!(NodeType::RawBytes; version.into()),
                        new_node!(NodeType::SP; b" ".into()),
                        new_node!(NodeType::RawBytes; status_code.into()),
                        new_node!(NodeType::SP; b" ".into()),
                        new_node!(NodeType::RawBytes; reason_phrase.into()),
                        new_node!(NodeType::CRLF; b"\r\n".into()),
                    )
                }
            },
        ));

        let mut field_lines = Vec::new();
        for (name, value) in self.headers {
            field_lines.push(new_node!(
                NodeType::FieldLine,
                new_node!(NodeType::RawBytes; name.into()),
                new_node!(NodeType::COLON; b":".into()),
                new_node!(NodeType::OWSBWS; b" ".into()),
                new_node!(NodeType::RawBytes; value.into()),
                new_node!(NodeType::OWSBWS; b"".into()),
                new_node!(NodeType::CRLF; b"\r\n".into()),
            ));
        }
        nodes.push(Node::new_with_children(
            NodeType::FieldLines,
            field_lines,
        ));

        nodes.push(new_node!(NodeType::CRLF; b"\r\n".into()));

        match self.body {
            HttpInputBody::Identity { body } => {
                nodes.push(new_node!(
                    NodeType::MessageBody,
                    new_node!(NodeType::RawBytes; body.into()),
                ));
            }
            HttpInputBody::Chunked { chunks, trailers } => {
                let mut chunk_nodes = Vec::new();
                let mut last_chunk: Option<*mut Node> = None;
                for (size, extension, data) in chunks {
                    let mut hexdigs = Vec::new();
                    for c in size.bytes() {
                        hexdigs.push(new_node!(NodeType::HEXDIG; vec![c]));
                    }
                    if size != "0" {
                        chunk_nodes.push(new_node!(
                            NodeType::Chunk,
                            Node::new_with_children(NodeType::ChunkSize, hexdigs),
                            new_node!(NodeType::RawBytes; extension.unwrap_or("".to_string()).into()),
                            new_node!(NodeType::CRLF; b"\r\n".into()),
                            new_node!(NodeType::RawBytes; data.into()),
                            new_node!(NodeType::CRLF; b"\r\n".into()),
                        ));
                    } else {
                        last_chunk = Some(new_node!(
                            NodeType::LastChunk,
                            Node::new_with_children(NodeType::ChunkSize, hexdigs),
                            new_node!(NodeType::RawBytes; extension.unwrap_or("".to_string()).into()),
                            new_node!(NodeType::CRLF; b"\r\n".into()),
                        ));
                    }
                }
                let mut extension_nodes = Vec::new();
                for (name, value) in trailers {
                    extension_nodes.push(new_node!(
                    NodeType::FieldLine,
                        new_node!(NodeType::RawBytes; name.into()),
                        new_node!(NodeType::COLON; b":".into()),
                        new_node!(NodeType::OWSBWS; b" ".into()),
                        new_node!(NodeType::RawBytes; value.into()),
                        new_node!(NodeType::OWSBWS; b"".into()),
                        new_node!(NodeType::CRLF; b"\r\n".into()),
                    ));
                }
                nodes.push(new_node!(
                    NodeType::MessageBody,
                    new_node!(NodeType::ChunkedBody,
                        Node::new_with_children(NodeType::Chunks, chunk_nodes),
                        last_chunk.unwrap(),
                        Node::new_with_children(NodeType::TrailerSection, extension_nodes),
                        new_node!(NodeType::CRLF; b"\r\n".into()),
                    ),
                ));
            }
            HttpInputBody::None => {
                nodes.push(new_node!(
                    NodeType::MessageBody,
                    new_node!(NodeType::RawBytes; b"".into()),
                ));
            }
        }

        HttpInput::new(Node::new_with_children(NodeType::HttpMessage, nodes)).unwrap()
    }

    pub fn from_raw(raw: &[u8], response: bool) -> Self {
        let mut builder = Self::new();
        let mut pos = raw.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4).unwrap();
        let lines = split_bytes_on(&raw[0..pos - 4], b"\r\n");

        let first_line = lines.first().unwrap();
        let mut parts = first_line.split(|&c| c == b' ');
        if response {
            builder = builder.status_line(
                std::str::from_utf8(parts.next().unwrap_or(b"")).unwrap(),
                std::str::from_utf8(parts.next().unwrap_or(b"")).unwrap(),
                std::str::from_utf8(&parts.collect::<Vec<&[u8]>>().join(&b' ')).unwrap(),
            );
        } else {
            builder = builder.request_line(
                std::str::from_utf8(parts.next().unwrap_or(b"")).unwrap(),
                std::str::from_utf8(parts.next().unwrap_or(b"")).unwrap(),
                std::str::from_utf8(&parts.collect::<Vec<&[u8]>>().join(&b' ')).unwrap(),
            );
        }

        let mut chunked = false;
        for line in &lines[1..] {
            let (name, value) = split_bytes_on_first(line, b":");
            let value = trim_bytes(value, b' ');
            builder = builder.header(
                std::str::from_utf8(name).unwrap(),
                std::str::from_utf8(value).unwrap(),
            );

            if name.to_ascii_lowercase() == b"transfer-encoding" && value.to_ascii_lowercase() == b"chunked" {
                chunked = true;
            }
        }

        if chunked {
            assert!(raw.ends_with(b"\r\n\r\n"));

            while pos < raw.len() {
                let data_start = raw[pos..].windows(2).position(|w| w == b"\r\n").map(|i| i + 2).unwrap();

                let extension_start = raw[pos..pos + data_start - 2].iter()
                    .position(|c| !((*c >= b'0' && *c <= b'9') || (*c >= b'a' && *c <= b'f') || (*c >= b'A' && *c <= b'F')));
                let extension = extension_start.map(|idx| std::str::from_utf8(&raw[pos + idx..pos + data_start - 2]).unwrap());

                let size = usize::from_str_radix(if extension_start.is_some() {
                    std::str::from_utf8(&raw[pos..pos + extension_start.unwrap()]).unwrap()
                } else {
                    std::str::from_utf8(&raw[pos..pos + data_start - 2]).unwrap()
                }, 16).unwrap();

                pos += data_start;
                let end = pos + size;
                let data = &raw[pos..end];

                builder = builder.chunk(data, extension);

                if size == 0 {
                    break
                } else {
                    assert!(raw[end..].starts_with(b"\r\n"));
                    pos = end + 2;
                }
            }

            if pos < raw.len()-4 {
                let lines = split_bytes_on(&raw[pos..raw.len()-4], b"\r\n");
                for line in lines {
                    let (name, value) = split_bytes_on_first(line, b":");
                    let value = trim_bytes(value, b' ');
                    builder = builder.trailer(
                        std::str::from_utf8(name).unwrap(),
                        std::str::from_utf8(value).unwrap(),
                    );
                }
            }
        } else {
            builder = builder.identity_body(&raw[pos..]);
        }

        builder
    }
}

pub fn inputs() -> Vec<HttpSequenceInput> { vec![
    HttpSequenceInput::new(vec![
        HttpInput::new(new_node!(
                NodeType::HttpMessage,
                new_node!(
                    NodeType::StartLine,
                    new_node!(
                        NodeType::RequestLine,
                        new_node!(NodeType::RawBytes; b"GET".into()),
                        new_node!(NodeType::SP; b" ".into()),
                        new_node!(NodeType::RawBytes; b"/".into()),
                        new_node!(NodeType::SP; b" ".into()),
                        new_node!(NodeType::RawBytes; b"HTTP/1.1".into()),
                        new_node!(NodeType::CRLF; b"\r\n".into()),
                    ),
                ),
                new_node!(
                    NodeType::FieldLines,
                    new_node!(
                        NodeType::FieldLine,
                        new_node!(NodeType::RawBytes; b"Host".into()),
                        new_node!(NodeType::COLON; b":".into()),
                        new_node!(NodeType::OWSBWS; b" ".into()),
                        new_node!(NodeType::RawBytes; b"any.com".into()),
                        new_node!(NodeType::OWSBWS; b"".into()),
                        new_node!(NodeType::CRLF; b"\r\n".into()),
                    ),
                ),
                new_node!(NodeType::CRLF; b"\r\n".into()),
                new_node!(
                    NodeType::MessageBody,
                    new_node!(NodeType::RawBytes; b"".into()),
                ),
        )).unwrap()
    ]),
    HttpSequenceInput::new(vec![
        HttpInput::new(new_node!(
                NodeType::HttpMessage,
                new_node!(
                    NodeType::StartLine,
                    new_node!(
                        NodeType::RequestLine,
                        new_node!(NodeType::RawBytes; b"POST".into()),
                        new_node!(NodeType::SP; b" ".into()),
                        new_node!(NodeType::RawBytes; b"/".into()),
                        new_node!(NodeType::SP; b" ".into()),
                        new_node!(NodeType::RawBytes; b"HTTP/1.1".into()),
                        new_node!(NodeType::CRLF; b"\r\n".into()),
                    ),
                ),
                new_node!(
                    NodeType::FieldLines,
                    new_node!(
                        NodeType::FieldLine,
                        new_node!(NodeType::RawBytes; b"Host".into()),
                        new_node!(NodeType::COLON; b":".into()),
                        new_node!(NodeType::OWSBWS; b" ".into()),
                        new_node!(NodeType::RawBytes; b"one.com".into()),
                        new_node!(NodeType::OWSBWS; b"".into()),
                        new_node!(NodeType::CRLF; b"\r\n".into()),
                    ),
                    new_node!(
                        NodeType::FieldLine,
                        new_node!(NodeType::RawBytes; b"Content-Length".into()),
                        new_node!(NodeType::COLON; b":".into()),
                        new_node!(NodeType::OWSBWS; b" ".into()),
                        new_node!(NodeType::RawBytes; b"12".into()),
                        new_node!(NodeType::OWSBWS; b"".into()),
                        new_node!(NodeType::CRLF; b"\r\n".into()),
                    ),
                ),
                new_node!(NodeType::CRLF; b"\r\n".into()),
                new_node!(
                    NodeType::MessageBody,
                    new_node!(NodeType::RawBytes; b"Helloworld!!".into()),
                ),
        )).unwrap(),
    ])
] }

#[macro_export]
macro_rules! escaped {
    ($slice:expr) => {
        {
            let mut result = String::new();
            for &c in $slice.iter() {
                if c.is_ascii_graphic() || c == b' ' {
                    result.push(c as char);
                } else {
                    result.push_str(&format!("\\x{:02X}", c));
                }
            }
            result
        }
    };
}
