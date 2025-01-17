use std::borrow::Cow;

use crate::input::{IsHttpSequenceInput, NodeLabel};
use crate::input::{Node, NodeType};
use crate::mutators::{random_input_from_corpus, random_input_from_sequence, swap_node};
use libafl::corpus::Corpus;
use libafl::inputs::Input;
use libafl::mutators::{rand_range, MutationResult, Mutator};
use libafl::{random_corpus_id, HasMetadata, SerdeAny};
use libafl::state::{HasCorpus, HasRand};
use libafl_bolts::rands::Rand;
use libafl_bolts::{Error, Named};
use serde::{Deserialize, Serialize};

pub struct MessageFieldLineDuplicateMutator;

impl MessageFieldLineDuplicateMutator {
    pub fn new() -> Self {
        Self
    }
}

impl Named for MessageFieldLineDuplicateMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("HTTPMessageFieldLineDuplicateMutator");
        &NAME
    }
}

impl<I, S> Mutator<I, S> for MessageFieldLineDuplicateMutator
where
    S: HasRand,
    I: IsHttpSequenceInput,
{
    fn mutate(
        &mut self,
        state: &mut S,
        input: &mut I,
    ) -> Result<MutationResult, Error> {
        let input = random_input_from_sequence!(state.rand_mut(), input.http_sequence_input_mut());
        let field_lines = input.field_lines_mut();
        if field_lines.children.is_empty() || field_lines.children.len() >= 20 {
            return Ok(MutationResult::Skipped);
        }
        let idx = state.rand_mut().below(field_lines.children.len());
        let node = Node::clone(field_lines.children[idx], field_lines);
        field_lines.children.insert(idx, node);
        field_lines.update_metadata_up(idx);
        Ok(MutationResult::Mutated)
    }
}

pub struct MessageFieldLineRemoveMutator;

impl MessageFieldLineRemoveMutator {
    pub fn new() -> Self {
        Self
    }
}

impl Named for MessageFieldLineRemoveMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("HTTPMessageFieldLineRemoveMutator");
        &NAME
    }
}

impl<I, S> Mutator<I, S> for MessageFieldLineRemoveMutator
where
    S: HasRand,
    I: IsHttpSequenceInput,
{
    fn mutate(
        &mut self,
        state: &mut S,
        input: &mut I,
    ) -> Result<MutationResult, Error> {
        let input = random_input_from_sequence!(state.rand_mut(), input.http_sequence_input_mut());
        let field_lines = input.field_lines_mut();
        if field_lines.children.is_empty() {
            return Ok(MutationResult::Skipped);
        }
        let idx = state.rand_mut().below(field_lines.children.len());
        let node = field_lines.children.remove(idx);
        field_lines.update_metadata_up(idx);
        Node::free(node);
        Ok(MutationResult::Mutated)
    }
}

pub struct MessageFieldLineSpliceMutator;

impl MessageFieldLineSpliceMutator {
    pub fn new() -> Self {
        Self
    }
}

impl Named for MessageFieldLineSpliceMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("HTTPMessageFieldLineSpliceMutator");
        &NAME
    }
}

impl<I, S> Mutator<I, S> for MessageFieldLineSpliceMutator
where
    S: HasRand + HasCorpus<Input = I>,
    I: Input + IsHttpSequenceInput,
{
    fn mutate(
        &mut self,
        state: &mut S,
        input: &mut I,
    ) -> Result<MutationResult, Error> {
        let input = random_input_from_sequence!(state.rand_mut(), input.http_sequence_input_mut());
        let count = input.field_lines().children.len();

        if count <= 0 || count >= 20 {
            return Ok(MutationResult::Skipped);
        }

        let other_input = random_input_from_corpus!(state);
        let other_count = other_input.field_lines().children.len();
        if other_count == 0 {
            return Ok(MutationResult::Skipped);
        }

        let range = rand_range(state, other_count, other_count);
        let pos = state.rand_mut().below(count);
        let field_lines = input.field_lines_mut();

        let other_nodes = other_input
            .field_lines()
            .children
            .iter()
            .skip(range.start)
            .take(range.len())
            .map(|x| Node::clone(*x, field_lines))
            .collect::<Vec<_>>();

        field_lines.children.splice(pos..pos, other_nodes);
        field_lines.update_metadata_up(pos);

        Ok(MutationResult::Mutated)
    }
}

pub struct MessageNodeTypedSwapMutator;

impl MessageNodeTypedSwapMutator {
    pub fn new() -> Self {
        Self
    }
}

impl Named for MessageNodeTypedSwapMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("HTTPMessageNodeTypedSwapMutator");
        &NAME
    }
}

impl<I, S> Mutator<I, S> for MessageNodeTypedSwapMutator
where
    S: HasRand,
    I: IsHttpSequenceInput,
{
    fn mutate(
        &mut self,
        state: &mut S,
        input: &mut I,
    ) -> Result<MutationResult, Error> {
        let input = random_input_from_sequence!(state.rand_mut(), input.http_sequence_input_mut());
        let node = unsafe { &mut *input.node };

        let iter = node.iter_node_mut();
        let node_a = state.rand_mut().choose(iter).unwrap() as *mut Node;
        unsafe {
            if (*node_a).parent.is_null() || (*(*node_a).parent).node_type == NodeType::MessageBody
            {
                return Ok(MutationResult::Skipped);
            }
        }

        let remaining: Vec<&mut Node> = node
            .iter_node_mut()
            .filter(|x| unsafe {
                x.node_type.label() == (*node_a).node_type.label()
                    && *x != &mut (*node_a)
                    && !x.parent.is_null()
                    && (*(*node_a).parent).node_type != NodeType::MessageBody
            })
            .collect();
        if remaining.len() == 0 {
            return Ok(MutationResult::Skipped);
        }

        let node_b = state.rand_mut().choose(remaining).unwrap() as *mut Node;
        unsafe {
            if (*node_a).is_parent_of(node_b) || (*node_b).is_parent_of(node_a) {
                return Ok(MutationResult::Skipped);
            }
        }

        swap_node!(node_a, node_b);

        Ok(MutationResult::Mutated)
    }
}

pub struct MessageTrailerSectionReplaceMutator;

impl MessageTrailerSectionReplaceMutator {
    pub fn new() -> Self {
        Self
    }
}

impl Named for MessageTrailerSectionReplaceMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("HTTPMessageTrailerSectionReplaceMutator");
        &NAME
    }
}

impl<I, S> Mutator<I, S> for MessageTrailerSectionReplaceMutator
where
    S: HasRand + HasCorpus<Input = I>,
    I: Input + IsHttpSequenceInput,
{
    fn mutate(
        &mut self,
        state: &mut S,
        input: &mut I,
    ) -> Result<MutationResult, Error> {
        let input = random_input_from_sequence!(state.rand_mut(), input.http_sequence_input_mut());
        let message_body = input.message_body_mut();
        unsafe {
            if message_body.node_type != NodeType::MessageBody
                || message_body.children.len() < 1
                || (*message_body.children[0]).node_type != NodeType::ChunkedBody
                || (*message_body.children[0]).children.len() < 3
                || (*(*message_body.children[0]).children[2]).node_type != NodeType::TrailerSection
            {
                return Ok(MutationResult::Skipped);
            }
        }

        let other_input = random_input_from_corpus!(state);
        let other_field_lines = other_input.field_lines();
        if other_field_lines.children.is_empty() {
            return Ok(MutationResult::Skipped);
        }

        unsafe {
            let trailer_section = (*message_body.children[0]).children[2];
            (*trailer_section).children = other_field_lines
                .children
                .iter()
                .map(|x| Node::clone(*x, trailer_section))
                .collect();
            (*trailer_section).update_metadata_up(0);
        }

        Ok(MutationResult::Mutated)
    }
}

#[derive(Serialize, Deserialize, Debug, SerdeAny)]
pub struct TokenMetadata {
    pub string: Vec<String>,
    pub number: Vec<String>,
    pub symbol: Vec<String>,
}

impl TokenMetadata {
    pub fn new() -> Self {
        Self {
            string: Vec::new(),
            number: Vec::new(),
            symbol: Vec::new(),
        }
    }
}

pub struct MessageNodeTokenReplaceMutator;

impl MessageNodeTokenReplaceMutator {
    pub fn new() -> Self {
        Self
    }
}

impl Named for MessageNodeTokenReplaceMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("HTTPMessageNodeTokenReplaceMutator");
        &NAME
    }
}

impl<I, S> Mutator<I, S> for MessageNodeTokenReplaceMutator
where
    S: HasRand + HasMetadata,
    I: IsHttpSequenceInput,
{
    fn mutate(
        &mut self,
        state: &mut S,
        input: &mut I,
    ) -> Result<MutationResult, Error> {
        let input = random_input_from_sequence!(state.rand_mut(), input.http_sequence_input_mut());
        let mut node = unsafe { &mut *input.node };

        while !node.is_leaf() {
            let iter = node.iter_node_mut();
            node = state.rand_mut().choose(iter).unwrap();
        }

        let tokens = state.metadata::<TokenMetadata>().unwrap();
        let length = match node.node_type.label() {
            NodeLabel::String => tokens.string.len(),
            NodeLabel::Number => tokens.number.len(),
            NodeLabel::Symbol => tokens.number.len(),
        };

        if length == 0 {
            return Ok(MutationResult::Skipped);
        }

        let idx = state.rand_mut().below(length);
        let tokens = state.metadata::<TokenMetadata>().unwrap();

        let token = match node.node_type.label() {
            NodeLabel::String => &tokens.string[idx],
            NodeLabel::Number => &tokens.number[idx],
            NodeLabel::Symbol => &tokens.symbol[idx],
        };

        node.value = Vec::from(token.as_bytes());
        node.update_metadata_up(0);

        Ok(MutationResult::Mutated)
    }
}