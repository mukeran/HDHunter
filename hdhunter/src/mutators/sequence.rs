use std::borrow::Cow;

use crate::input::IsHttpSequenceInput;
use crate::mutators::random_input_from_corpus;
use libafl::corpus::Corpus;
use libafl::inputs::Input;
use libafl::mutators::{MutationResult, Mutator};
use libafl::random_corpus_id;
use libafl::state::{HasCorpus, HasRand};
use libafl_bolts::rands::Rand;
use libafl_bolts::{Error, Named};

pub struct SequenceSpliceMutator;

impl SequenceSpliceMutator {
    pub fn new() -> Self {
        Self
    }
}

impl Named for SequenceSpliceMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("HTTPSequenceSpliceMutator");
        &NAME
    }
}

impl<I, S> Mutator<I, S> for SequenceSpliceMutator
where
    S: HasRand + HasCorpus<Input = I>,
    I: Input + IsHttpSequenceInput,
{
    fn mutate(
        &mut self,
        state: &mut S,
        input: &mut I,
    ) -> Result<MutationResult, Error> {
        let sequence = input.http_sequence_input_mut();
        if sequence.inputs.len() >= 3 {
            return Ok(MutationResult::Skipped);
        }

        let other_input = random_input_from_corpus!(state);

        let idx = state.rand_mut().below(sequence.inputs.len());
        sequence.inputs.insert(idx, other_input);

        Ok(MutationResult::Mutated)
    }
}

pub struct SequenceRemoveMutator;

impl SequenceRemoveMutator {
    pub fn new() -> Self {
        Self
    }
}

impl Named for SequenceRemoveMutator {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("HTTPSequenceRemoveMutator");
        &NAME
    }
}

impl<I, S> Mutator<I, S> for SequenceRemoveMutator
where
    S: HasRand,
    I: IsHttpSequenceInput,
{
    fn mutate(
        &mut self,
        state: &mut S,
        input: &mut I,
    ) -> Result<MutationResult, Error> {
        let sequence = input.http_sequence_input_mut();
        if sequence.inputs.len() < 2 {
            return Ok(MutationResult::Skipped);
        }

        let idx = state.rand_mut().below(sequence.inputs.len());
        sequence.inputs.remove(idx);

        Ok(MutationResult::Mutated)
    }
}
