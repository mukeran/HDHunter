use libafl_bolts::ownedref::OwnedSlice;
use libafl_bolts::{AsSlice, Error, ErrorBacktrace};
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Formatter};
use std::ptr::null_mut;

macro_rules! constraint {
    ($cond: expr) => {
        if !$cond {
            return Err(Error::IllegalArgument(
                format!("constraint not match: {}", stringify!($cond)),
                ErrorBacktrace::new(),
            ));
        }
    };
}

#[derive(PartialEq)]
pub enum NodeLabel {
    String,
    Number,
    Symbol,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum NodeType {
    HttpMessage,    /* HTTP-message = start-line *( field-line CRLF ) CRLF [message-body]*/
    StartLine,      /* start-line = request-line / status-line */
    RequestLine,    /* request-line = method SP request-target SP HTTP-version CRLF */
    StatusLine,     /* status-line = HTTP-version SP status-code SP [reason-phrase] CRLF */
    FieldLines,     /* (wrapper) field-lines = *field-line */
    FieldLine,      /* field-line = field-name ":" OWS field-value OWS CRLF */
    MessageBody,    /* message-body = chunked-body / *OCTET */
    ChunkedBody,    /* chunked-body = *chunk last-chunk trailer-section CRLF */
    Chunks,         /* (wrapper) chunks = *chunk */
    Chunk,          /* chunk = chunk-size [ chunk-ext ] CRLF chunk-data CRLF */
    ChunkSize,      /* chunk-size = 1*HEXDIG */
    LastChunk,      /* last-chunk = 1*("0") [ chunk-ext ] CRLF */
    TrailerSection, /* trailer-section   = *field-line */
    SP,             /* SP = %x20 */
    HTAB,           /* HTAB = %x09 */
    OWSBWS,         /* OWS = *( SP / HTAB ) */
    CRLF,           /* CRLF = %x0D %x0A */
    HEXDIG,         /* HEXDIG = DIGIT / "A" / "B" / "C" / "D" / "E" / "F" */
    COLON,          /* COLON = %x3A */
    RawBytes,       /* RawBytes = *OCTET */
}

// Map NodeType to NodeLabel
impl NodeType {
    pub fn label(&self) -> NodeLabel {
        match self {
            NodeType::HttpMessage => NodeLabel::String,
            NodeType::StartLine => NodeLabel::String,
            NodeType::RequestLine => NodeLabel::String,
            NodeType::StatusLine => NodeLabel::String,
            NodeType::FieldLines => NodeLabel::String,
            NodeType::FieldLine => NodeLabel::String,
            NodeType::MessageBody => NodeLabel::String,
            NodeType::ChunkedBody => NodeLabel::String,
            NodeType::Chunks => NodeLabel::String,
            NodeType::Chunk => NodeLabel::String,
            NodeType::ChunkSize => NodeLabel::Number,
            NodeType::LastChunk => NodeLabel::String,
            NodeType::TrailerSection => NodeLabel::String,
            NodeType::SP => NodeLabel::Symbol,
            NodeType::HTAB => NodeLabel::Symbol,
            NodeType::OWSBWS => NodeLabel::Symbol,
            NodeType::CRLF => NodeLabel::Symbol,
            NodeType::HEXDIG => NodeLabel::Number,
            NodeType::COLON => NodeLabel::Symbol,
            NodeType::RawBytes => NodeLabel::String,
        }
    }

    pub fn validate(&self, children: &[*mut Node]) -> Result<(), Error> {
        unsafe {
            match self {
                NodeType::HttpMessage => {
                    constraint!(children.len() == 4);
                    constraint!((*children[0]).node_type == NodeType::StartLine);
                    constraint!((*children[1]).node_type == NodeType::FieldLines);
                    constraint!((*children[2]).node_type == NodeType::CRLF);
                    constraint!((*children[3]).node_type == NodeType::MessageBody);
                }
                NodeType::StartLine => {
                    constraint!(children.len() == 1);
                    constraint!(
                        (*children[0]).node_type == NodeType::RequestLine
                            || (*children[0]).node_type == NodeType::StatusLine
                    );
                }
                NodeType::RequestLine | NodeType::StatusLine => {
                    constraint!(children.len() == 6);
                    constraint!((*children[0]).node_type == NodeType::RawBytes);
                    constraint!((*children[1]).node_type == NodeType::SP);
                    constraint!((*children[2]).node_type == NodeType::RawBytes);
                    constraint!((*children[3]).node_type == NodeType::SP);
                    constraint!((*children[4]).node_type == NodeType::RawBytes);
                    constraint!((*children[5]).node_type == NodeType::CRLF);
                }
                NodeType::FieldLines => {
                    constraint!(children
                        .iter()
                        .all(|x| { (**x).node_type == NodeType::FieldLine }));
                }
                NodeType::FieldLine => {
                    constraint!(children.len() == 6);
                    constraint!((*children[0]).node_type == NodeType::RawBytes);
                    constraint!((*children[1]).node_type == NodeType::COLON);
                    constraint!((*children[2]).node_type == NodeType::OWSBWS);
                    constraint!((*children[3]).node_type == NodeType::RawBytes);
                    constraint!((*children[4]).node_type == NodeType::OWSBWS);
                    constraint!((*children[5]).node_type == NodeType::CRLF);
                }
                NodeType::MessageBody => {
                    constraint!(children.len() == 1);
                    constraint!(
                        (*children[0]).node_type == NodeType::ChunkedBody
                            || (*children[0]).node_type == NodeType::RawBytes
                    );
                }
                NodeType::ChunkedBody => {
                    constraint!(children.len() == 4);
                    constraint!((*children[0]).node_type == NodeType::Chunks);
                    constraint!((*children[1]).node_type == NodeType::LastChunk);
                    constraint!((*children[2]).node_type == NodeType::TrailerSection);
                    constraint!((*children[3]).node_type == NodeType::CRLF);
                }
                NodeType::Chunks => {
                    constraint!(children
                        .iter()
                        .all(|x| { (**x).node_type == NodeType::Chunk }));
                }
                NodeType::Chunk => {
                    constraint!(children.len() == 5);
                    constraint!((*children[0]).node_type == NodeType::ChunkSize);
                    constraint!((*children[1]).node_type == NodeType::RawBytes);
                    constraint!((*children[2]).node_type == NodeType::CRLF);
                    constraint!((*children[3]).node_type == NodeType::RawBytes);
                    constraint!((*children[4]).node_type == NodeType::CRLF);
                }
                NodeType::ChunkSize => {
                    constraint!(children
                        .iter()
                        .all(|x| { (**x).node_type == NodeType::HEXDIG }));
                }
                NodeType::LastChunk => {
                    constraint!(children.len() == 3);
                    constraint!((*children[0]).node_type == NodeType::ChunkSize);
                    constraint!((*children[1]).node_type == NodeType::RawBytes);
                    constraint!((*children[2]).node_type == NodeType::CRLF);
                }
                NodeType::TrailerSection => {
                    constraint!(children
                        .iter()
                        .all(|x| { (**x).node_type == NodeType::FieldLine }));
                }
                NodeType::SP
                | NodeType::HTAB
                | NodeType::OWSBWS
                | NodeType::CRLF
                | NodeType::HEXDIG
                | NodeType::COLON
                | NodeType::RawBytes => {
                    constraint!(children.len() == 0)
                }
            }
        }
        Ok(())
    }
}

pub struct Node {
    pub node_type: NodeType,
    pub value: Vec<u8>,

    value_sum: usize,
    leaf_sum: usize,
    node_sum: usize,

    pub children: Vec<*mut Node>,
    pub parent: *mut Node,
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let children = self
            .children
            .iter()
            .map(|x| unsafe { &**x })
            .collect::<Vec<_>>();
        f.debug_struct("Node")
            .field("node_type", &self.node_type)
            .field("value", &self.value)
            .field("children", &children)
            .finish()
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.node_type == other.node_type
            && self.value == other.value
            && self
                .children
                .iter()
                .zip(&other.children)
                .all(|x| unsafe { (**x.0).eq(&**x.1) })
    }
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Node", 3)?;
        state.serialize_field("node_type", &self.node_type)?;
        state.serialize_field("value", &self.value)?;
        let children = self
            .children
            .iter()
            .map(|x| unsafe { &**x })
            .collect::<Vec<_>>();
        state.serialize_field("children", &children)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NodeVisitor;

        impl<'de> Visitor<'de> for NodeVisitor {
            type Value = Node;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct Node")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut node_type = None;
                let mut value = None;
                let mut children: Option<Vec<Node>> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "node_type" => {
                            node_type = Some(map.next_value()?);
                        }
                        "value" => {
                            value = Some(map.next_value()?);
                        }
                        "children" => {
                            children = Some(map.next_value()?);
                        }
                        _ => {}
                    }
                }
                let node_type =
                    node_type.ok_or_else(|| serde::de::Error::missing_field("node_type"))?;
                let value = value.ok_or_else(|| serde::de::Error::missing_field("value"))?;
                let children =
                    children.ok_or_else(|| serde::de::Error::missing_field("children"))?;
                let mut boxed_children: Vec<*mut Node> = vec![];
                for child in children {
                    boxed_children.push(Box::into_raw(Box::new(child)));
                }
                Ok(Node {
                    node_type,
                    value,
                    value_sum: 0,
                    leaf_sum: 0,
                    node_sum: 0,
                    children: boxed_children,
                    parent: null_mut(),
                })
            }
        }

        deserializer.deserialize_map(NodeVisitor)
    }
}

#[macro_export]
macro_rules! new_node {
    ($node_type: expr, $($child: expr,)*) => {
        Node::new_with_children($node_type, vec![$($child,)*])
    };
    ($node_type: expr; $value: expr) => {
        Node::new_with_value($node_type, $value)
    };
}

pub use new_node;

impl Node {
    pub fn new_with_children(node_type: NodeType, children: Vec<*mut Node>) -> *mut Self {
        if children.len() == 0 {
            return Self::new_with_value(node_type, vec![]);
        }
        let node = Box::into_raw(Box::new(Self {
            node_type,
            children,
            value: vec![],
            value_sum: 0,
            leaf_sum: 0,
            node_sum: 0,
            parent: null_mut(),
        }));
        unsafe {
            (*node)
                .children
                .iter_mut()
                .for_each(|x| (**x).parent = node);
            (*node).update_metadata(0);
        }
        node
    }

    pub fn new_with_value(node_type: NodeType, value: Vec<u8>) -> *mut Self {
        Box::into_raw(Box::new(Self {
            node_type,
            children: vec![],
            value_sum: value.len(),
            value,
            leaf_sum: 0,
            node_sum: 0,
            parent: null_mut(),
        }))
    }

    pub fn free(node: *mut Node) {
        unsafe {
            for child in (*node).children.iter() {
                Self::free(*child);
            }
            drop(Box::from_raw(node));
        }
    }

    pub fn clone(node: *const Node, parent: *mut Node) -> *mut Node {
        let node = unsafe { &*node };
        let new_node = Box::into_raw(Box::new(Node {
            node_type: node.node_type.clone(),
            value: node.value.clone(),
            value_sum: node.value_sum,
            leaf_sum: node.leaf_sum,
            node_sum: node.node_sum,
            children: vec![],
            parent,
        }));
        unsafe {
            (*new_node).children = node
                .children
                .iter()
                .map(|x| Self::clone(*x, new_node))
                .collect();
        }
        new_node
    }
}

impl Node {
    pub fn validate(&self) -> Result<(), Error> {
        self.node_type.validate(&self.children)?;
        for child in self.children.iter() {
            unsafe { (**child).validate()? };
        }
        Ok(())
    }

    pub fn bytes(&self) -> OwnedSlice<u8> {
        if self.children.is_empty() {
            OwnedSlice::from(&self.value)
        } else {
            let mut bytes = Vec::new();
            for child in self.children.iter() {
                unsafe {
                    bytes.extend_from_slice((**child).bytes().as_slice());
                }
            }
            OwnedSlice::from(bytes)
        }
    }

    pub fn value_size(&self) -> usize {
        if self.children.is_empty() {
            self.value.len()
        } else {
            unsafe { &**self.children.last().unwrap() }.value_sum
        }
    }

    pub fn leaf_size(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            unsafe { &**self.children.last().unwrap() }.leaf_sum
        }
    }

    pub fn node_size(&self) -> usize {
        if self.children.is_empty() {
            0
        } else {
            unsafe { &**self.children.last().unwrap() }.node_sum + self.children.len()
        }
    }

    pub fn update_metadata(&mut self, idx: usize) {
        if self.children.is_empty() {
            return;
        }
        for i in idx..self.children.len() {
            let current = unsafe { &mut *self.children[i] };
            if i == 0 {
                current.value_sum = current.value_size();
                current.leaf_sum = current.leaf_size();
                current.node_sum = current.node_size();
            } else {
                let prev = unsafe { &*self.children[i - 1] };
                current.value_sum = prev.value_sum + current.value_size();
                current.leaf_sum = prev.leaf_sum + current.leaf_size();
                current.node_sum = prev.node_sum + current.node_size();
            }
        }
    }

    pub fn update_metadata_up(&mut self, idx: usize) {
        self.update_metadata(idx);
        if !self.parent.is_null() {
            let parent = unsafe { &mut *self.parent };
            parent.update_metadata_up(
                parent
                    .children
                    .iter()
                    .position(|x| *x == self as *mut Node)
                    .unwrap(),
            );
        }
    }

    pub fn update_metadata_down(&mut self) {
        if self.children.is_empty() {
            self.value_sum = self.value.len();
            self.leaf_sum = 1;
            self.node_sum = 0;
            return;
        }
        let ptr = self as *mut Node;
        for child in self.children.iter() {
            unsafe {
                (**child).parent = ptr;
                (**child).update_metadata_down();
            }
        }
        self.update_metadata(0);
    }

    pub fn insert_child(&mut self, child: *mut Node, idx: usize) {
        if self.children.is_empty() {
            self.children = vec![child];
            self.value.clear();
        } else {
            self.children.insert(idx, child);
            self.update_metadata_up(idx);
        }
    }

    pub fn add_child(&mut self, child: *mut Node) {
        self.insert_child(
            child,
            if self.children.is_empty() {
                0
            } else {
                self.children.len()
            },
        );
    }

    pub fn remove_child(&mut self, idx: usize) {
        if self.children.is_empty() || idx >= self.children.len() {
            return;
        }
        let child = self.children.remove(idx);
        self.update_metadata_up(idx);
        Self::free(child);
    }

    pub fn iter_node(&self) -> NodeIterator {
        NodeIterator {
            0: NodeBaseIterator::new(self),
        }
    }

    pub fn iter_node_mut(&mut self) -> NodeMutIterator {
        NodeMutIterator {
            0: NodeBaseIterator::new_mut(self),
        }
    }

    pub fn iter_value(&self) -> NodeValueIterator {
        NodeValueIterator::new(self)
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    pub fn is_parent_of(&self, mut other: *const Node) -> bool {
        while other != null_mut() {
            if other == self {
                return true;
            }
            other = unsafe { &*other }.parent;
        }
        false
    }

    pub fn locate_value_mut(&mut self, idx: usize) -> Option<(&mut Node, usize)> {
        if self.children.is_empty() {
            return Some((self, idx));
        }
        let mut idx = idx;
        for child in self.children.iter_mut() {
            let child = unsafe { &mut **child };
            if idx < child.value_size() {
                return child.locate_value_mut(idx);
            }
            idx -= child.value_size();
        }
        None
    }

    pub fn child(&self, idx: usize) -> Option<&Node> {
        if idx >= self.children.len() {
            return None;
        }
        Some(unsafe { &*self.children[idx] })
    }

    pub fn child_mut(&mut self, idx: usize) -> Option<&mut Node> {
        if idx >= self.children.len() {
            return None;
        }
        Some(unsafe { &mut *self.children[idx] })
    }

    pub unsafe fn child_unchecked(&self, idx: usize) -> &Node {
        &*self.children[idx]
    }

    pub unsafe fn child_mut_unchecked(&mut self, idx: usize) -> &mut Node {
        &mut *self.children[idx]
    }

    pub fn children(&self) -> Vec<&Node> {
        self.children.iter().map(|x| unsafe { &**x }).collect()
    }
}

pub struct NodeBaseIterator<'a> {
    root: *const Node,
    stack: Vec<(*const Node, usize)>,
    phantom: std::marker::PhantomData<&'a Node>,
}

impl<'a> NodeBaseIterator<'a> {
    pub fn new(node: &Node) -> Self {
        let mut stack = Vec::new();
        stack.push((node as *const Node, 0));
        NodeBaseIterator {
            root: node,
            stack,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn new_mut(node: &mut Node) -> Self {
        let mut stack = Vec::new();
        stack.push((node as *const Node, 0));
        NodeBaseIterator {
            root: node,
            stack,
            phantom: std::marker::PhantomData,
        }
    }
}

pub struct NodeIterator<'a>(NodeBaseIterator<'a>);

impl<'a> Iterator for NodeIterator<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, idx)) = self.0.stack.pop() {
            let node = unsafe { &*node };
            if node.children.is_empty() {
                continue;
            }
            if idx < node.children.len() {
                self.0.stack.push((node, idx + 1));
                self.0.stack.push((node.children[idx], 0));
                return Some(unsafe { &*node.children[idx] });
            }
        }
        None
    }
}

impl ExactSizeIterator for NodeIterator<'_> {
    fn len(&self) -> usize {
        unsafe { (*self.0.root).node_size() }
    }
}

pub struct NodeMutIterator<'a>(NodeBaseIterator<'a>);

impl<'a> Iterator for NodeMutIterator<'a> {
    type Item = &'a mut Node;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, idx)) = self.0.stack.pop() {
            let node = unsafe { &mut *(node as *mut Node) };
            if node.children.is_empty() {
                continue;
            }
            if idx < node.children.len() {
                self.0.stack.push((node as *const Node, idx + 1));
                self.0.stack.push((node.children[idx], 0));
                return Some(unsafe { &mut *node.children[idx] });
            }
        }
        None
    }
}

impl ExactSizeIterator for NodeMutIterator<'_> {
    fn len(&self) -> usize {
        unsafe { (*self.0.root).node_size() }
    }
}

pub struct NodeValueIterator<'a> {
    root: *const Node,
    stack: Vec<(*const Node, usize)>,
    phantom: std::marker::PhantomData<&'a u8>,
}

impl<'a> NodeValueIterator<'a> {
    pub fn new(node: *const Node) -> Self {
        let mut stack = Vec::new();
        stack.push((node, 0));
        NodeValueIterator {
            root: node,
            stack,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> Iterator for NodeValueIterator<'a> {
    type Item = &'a u8;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, idx)) = self.stack.pop() {
            let node = unsafe { &*node };
            if node.children.is_empty() {
                if idx < node.value.len() {
                    self.stack.push((node, idx + 1));
                    return Some(&node.value[idx]);
                }
                continue;
            }
            if idx < node.children.len() {
                self.stack.push((node, idx + 1));
                self.stack.push((node.children[idx], 0));
            }
        }
        None
    }
}

impl ExactSizeIterator for NodeValueIterator<'_> {
    fn len(&self) -> usize {
        unsafe { &*self.root }.value_size()
    }
}
