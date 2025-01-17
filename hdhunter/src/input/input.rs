use crate::input::{Node, NodeType};
use crate::mode::Mode;
use ahash::RandomState;
use libafl::inputs::{HasTargetBytes, Input};
use libafl_bolts::bolts_prelude::{write_file_atomic, OwnedSlice};
use libafl_bolts::{AsSlice, Error, ErrorBacktrace, HasLen};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::hash::{BuildHasher, Hash, Hasher};
use std::io::BufReader;
use std::path::Path;
use std::ptr::null_mut;

use super::{to_ajp, to_fastcgi, to_scgi, to_uwsgi};

static mut MODE: Mode = Mode::Request;

pub fn set_mode(mode: &Mode) {
    unsafe {
        MODE = *mode;
    }
}

fn to_json_file<J, P>(input: &J, path: P) -> Result<(), Error>
where
    J: Serialize,
    P: AsRef<Path>,
{
    let data = serde_json::to_vec(input)?;
    write_file_atomic(path, &data)
}

fn from_json_file<J, P>(path: P) -> Result<J, Error>
where
    J: for<'de> Deserialize<'de>,
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let input = serde_json::from_reader(reader)?;
    Ok(input)
}

pub trait IsHttpInput {
    fn http_input(&self) -> &HttpInput;
    fn http_input_mut(&mut self) -> &mut HttpInput;
}

pub struct HttpInput {
    pub(crate) node: *mut Node,
}

impl Debug for HttpInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "{:?}", &*self.node) }
    }
}

impl PartialEq for HttpInput {
    fn eq(&self, other: &Self) -> bool {
        unsafe { (*self.node).eq(&*other.node) }
    }
}

impl Drop for HttpInput {
    fn drop(&mut self) {
        Node::free(self.node);
    }
}

impl Clone for HttpInput {
    fn clone(&self) -> Self {
        HttpInput {
            node: Node::clone(self.node, null_mut()),
        }
    }
}

impl Serialize for HttpInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        unsafe { (*self.node).serialize(serializer) }
    }
}

impl<'de> Deserialize<'de> for HttpInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let node = Box::into_raw(Box::new(Node::deserialize(deserializer)?));
        unsafe {
            (*node).update_metadata_down();
        }
        Ok(HttpInput { node })
    }
}

impl Hash for HttpInput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.target_bytes().as_slice().hash(state)
    }
}

impl HttpInput {
    pub fn new(node: *mut Node) -> Result<Self, Error> {
        if node.is_null() {
            return Err(Error::IllegalArgument(
                "node must not be null".to_string(),
                ErrorBacktrace::new(),
            ));
        }
        let input = HttpInput { node };
        let node = unsafe { &*input.node };
        if (*node).node_type != NodeType::HttpMessage {
            return Err(Error::IllegalArgument(
                "node must be an HttpMessage".to_string(),
                ErrorBacktrace::new(),
            ));
        }
        node.validate()?;
        Ok(input)
    }

    pub fn first_line(&self) -> &Node {
        unsafe { &*(*self.node).children[0] }
    }

    pub fn first_line_mut(&mut self) -> &mut Node {
        unsafe { &mut *(*self.node).children[0] }
    }

    pub fn field_lines(&self) -> &Node {
        unsafe { &*(*self.node).children[1] }
    }

    pub fn field_lines_mut(&mut self) -> &mut Node {
        unsafe { &mut *(*self.node).children[1] }
    }

    pub fn message_body(&self) -> &Node {
        unsafe { &*(*self.node).children[3] }
    }

    pub fn message_body_mut(&mut self) -> &mut Node {
        unsafe { &mut *(*self.node).children[3] }
    }

    pub fn node(&self) -> &Node {
        unsafe { &*self.node }
    }

    pub fn node_mut(&mut self) -> &mut Node {
        unsafe { &mut *self.node }
    }
}

impl<'a> IsHttpInput for HttpInput {
    fn http_input(&self) -> &HttpInput {
        self
    }

    fn http_input_mut(&mut self) -> &mut HttpInput {
        self
    }
}

impl<'a> Input for HttpInput {
    fn to_file<P>(&self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        to_json_file(self, path)
    }

    fn from_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        from_json_file(path)
    }

    fn generate_name(&self, _idx: usize) -> String {
        let mut hasher = RandomState::with_seeds(0, 0, 0, 0).build_hasher();
        hasher.write(self.target_bytes().as_slice());
        format!("{:016x}", hasher.finish())
    }
}

impl<'a> HasTargetBytes for HttpInput {
    fn target_bytes(&self) -> OwnedSlice<u8> {
        match unsafe { MODE } {
            Mode::Request | Mode::Response => unsafe { (*self.node).bytes() },
            Mode::SCGI => to_scgi(self),
            Mode::FastCGI => to_fastcgi(self),
            Mode::AJP => to_ajp(self),
            Mode::UWSGI => to_uwsgi(self)
        }
    }
}

impl<'a> HasLen for HttpInput {
    fn len(&self) -> usize {
        self.target_bytes().as_slice().len()
    }
}

pub trait IsHttpSequenceInput {
    fn http_sequence_input(&self) -> &HttpSequenceInput;
    fn http_sequence_input_mut(&mut self) -> &mut HttpSequenceInput;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct HttpSequenceInput {
    pub(crate) inputs: Vec<HttpInput>,
}

impl IsHttpSequenceInput for HttpSequenceInput {
    fn http_sequence_input(&self) -> &HttpSequenceInput {
        self
    }

    fn http_sequence_input_mut(&mut self) -> &mut HttpSequenceInput {
        self
    }
}

impl HttpSequenceInput {
    pub fn new(inputs: Vec<HttpInput>) -> Self {
        Self { inputs }
    }
}

impl<'a> Input for HttpSequenceInput {
    fn to_file<P>(&self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        to_json_file(self, path)
    }

    fn from_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        from_json_file(path)
    }

    fn generate_name(&self, _idx: usize) -> String {
        let mut hasher = RandomState::with_seeds(0, 0, 0, 0).build_hasher();
        for input in &self.inputs {
            hasher.write(input.target_bytes().as_slice());
        }
        format!("{:016x}", hasher.finish())
    }
}

impl<'a> HasTargetBytes for HttpSequenceInput {
    fn target_bytes(&self) -> OwnedSlice<u8> {
        let mut bytes = Vec::new();
        for input in &self.inputs {
            bytes.extend_from_slice(input.target_bytes().as_slice());
        }
        OwnedSlice::from(bytes)
    }
}

impl<'a> HasLen for HttpSequenceInput {
    fn len(&self) -> usize {
        self.target_bytes().as_slice().len()
    }
}

#[cfg(test)]
mod tests {
    use crate::input::input::HttpInput;
    use crate::input::*;
    use libafl::inputs::Input;
    use std::path::PathBuf;

    #[test]
    fn test_http_sequence_input() {
        let start_line = new_node!(
            NodeType::StartLine,
            new_node!(
                NodeType::RequestLine,
                new_node!(NodeType::RawBytes; b"GET".into()),
                new_node!(NodeType::SP; b" ".into()),
                new_node!(NodeType::RawBytes; b"/index.html".into()),
                new_node!(NodeType::SP; b" ".into()),
                new_node!(NodeType::RawBytes; b"HTTP/1.1".into()),
                new_node!(NodeType::CRLF; b"\r\n".into()),
            ),
        );
        let field_lines = new_node!(
            NodeType::FieldLines,
            new_node!(
                NodeType::FieldLine,
                new_node!(NodeType::RawBytes; b"Host".into()),
                new_node!(NodeType::COLON; b":".into()),
                new_node!(NodeType::OWSBWS; b" ".into()),
                new_node!(NodeType::RawBytes; b"example.com".into()),
                new_node!(NodeType::OWSBWS; b"".into()),
                new_node!(NodeType::CRLF; b"\r\n".into()),
            ),
        );
        let message_body = new_node!(
            NodeType::MessageBody,
            new_node!(NodeType::RawBytes; b"".into()),
        );
        let input = HttpInput::new(new_node!(
            NodeType::HttpMessage,
            start_line,
            field_lines,
            new_node!(NodeType::CRLF; b"\r\n".into()),
            message_body,
        ))
        .unwrap();
        let sequence = HttpSequenceInput::new(vec![input.clone(), input.clone()]);
        let path = PathBuf::from("../corpus_initial/input.json");
        sequence.to_file(&path).unwrap();
        let sequence2 = HttpSequenceInput::from_file(&path).unwrap();
        assert_eq!(sequence, sequence2);
    }
}
