use crate::input::IsHttpSequenceInput;
use crate::mutators::{random_input_from_corpus, random_input_from_sequence};
use libafl::corpus::Corpus;
use libafl::inputs::{HasTargetBytes, Input};
use libafl::mutators::{rand_range, MutationResult, Mutator};
use libafl::random_corpus_id;
use libafl::state::{HasCorpus, HasMaxSize, HasRand};
use libafl_bolts::bolts_prelude::Rand;
use libafl_bolts::{Error, HasLen, Named};
use std::borrow::Cow;
use std::cmp::{max, min};

pub struct ByteInsertMutator;

impl ByteInsertMutator {
    pub fn new() -> Self {
        Self
    }
}

impl Named for ByteInsertMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("HTTPInputByteInsertMutator");
        &NAME
    }
}

impl<I, S> Mutator<I, S> for ByteInsertMutator
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
        let len = input.len();
        if len == 0 {
            Ok(MutationResult::Skipped)
        } else {
            let idx = state.rand_mut().below(len);
            let byte = state.rand_mut().below(256) as u8;
            let (node, idx) = unsafe { (*input.node).locate_value_mut(idx).unwrap() };
            node.value.insert(idx, byte);
            node.update_metadata_up(0);
            Ok(MutationResult::Mutated)
        }
    }
}

pub struct ByteRemoveMutator;

impl ByteRemoveMutator {
    pub fn new() -> Self {
        Self
    }
}

impl Named for ByteRemoveMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("HTTPInputByteRemoveMutator");
        &NAME
    }
}

impl<I, S> Mutator<I, S> for ByteRemoveMutator
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
        let len = input.len();
        if len == 0 {
            Ok(MutationResult::Skipped)
        } else {
            let idx = state.rand_mut().below(len);
            let (node, idx) = unsafe { (*input.node).locate_value_mut(idx).unwrap() };
            node.value.remove(idx);
            node.update_metadata_up(0);
            Ok(MutationResult::Mutated)
        }
    }
}

pub struct ByteDuplicateMutator;

impl ByteDuplicateMutator {
    pub fn new() -> Self {
        Self
    }
}

impl Named for ByteDuplicateMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("HTTPInputByteDuplicateMutator");
        &NAME
    }
}

impl<I, S> Mutator<I, S> for ByteDuplicateMutator
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
        let len = input.len();
        if len == 0 {
            Ok(MutationResult::Skipped)
        } else {
            let idx = state.rand_mut().below(len);
            let (node, idx) = unsafe { (*input.node).locate_value_mut(idx).unwrap() };
            node.value.insert(idx, node.value[idx]);
            node.update_metadata_up(0);
            Ok(MutationResult::Mutated)
        }
    }
}

pub struct ByteSpliceMutator;

impl ByteSpliceMutator {
    pub fn new() -> Self {
        Self
    }
}

impl Named for ByteSpliceMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("HTTPInputByteSpliceMutator");
        &NAME
    }
}

impl<I, S> Mutator<I, S> for ByteSpliceMutator
where
    S: HasRand + HasCorpus<Input = I> + HasMaxSize,
    I: Input + IsHttpSequenceInput,
{
    fn mutate(
        &mut self,
        state: &mut S,
        input: &mut I,
    ) -> Result<MutationResult, Error> {
        let input = random_input_from_sequence!(state.rand_mut(), input.http_sequence_input_mut());
        let mut max_size = state.max_size();
        let len = input.len();
        max_size = max(max_size, len);
        if len == 0 {
            Ok(MutationResult::Skipped)
        } else {
            let other_input = random_input_from_corpus!(state);
            let other_len = other_input.len();

            if other_len == 0 {
                return Ok(MutationResult::Skipped);
            }

            let range = rand_range(state, other_len, min(other_len, max_size - len));
            let pos = state.rand_mut().below(len);

            let (node, pos) = unsafe { (*input.node).locate_value_mut(pos).unwrap() };
            node.value.splice(
                pos..pos,
                other_input
                    .target_bytes()
                    .iter()
                    .skip(range.start)
                    .take(range.len())
                    .map(|x| *x),
            );
            node.update_metadata_up(0);

            Ok(MutationResult::Mutated)
        }
    }
}
