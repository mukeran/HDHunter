pub mod byte;
pub mod message;
pub mod sequence;
#[cfg(test)]
mod tests;

macro_rules! random_input_from_sequence {
    ($rand:expr,$seq:expr) => {
        $rand.choose(&mut $seq.inputs).unwrap()
    };
}

macro_rules! random_input_from_corpus {
    ($state:expr) => {{
        let idx = random_corpus_id!($state.corpus(), $state.rand_mut());
        if let Some(cur) = $state.corpus().current() {
            if idx == *cur {
                return Ok(MutationResult::Skipped);
            }
        }

        let other_count = {
            let mut other_testcase = $state.corpus().get(idx).unwrap().borrow_mut();
            let other_input = other_testcase
                .load_input($state.corpus())
                .unwrap()
                .http_sequence_input();
            other_input.inputs.len()
        };
        if other_count == 0 {
            return Ok(MutationResult::Skipped);
        }

        let input_idx = $state.rand_mut().below(other_count);
        let mut other_testcase = $state.corpus().get(idx).unwrap().borrow_mut();
        other_testcase
            .load_input($state.corpus())
            .unwrap()
            .http_sequence_input()
            .inputs[input_idx]
            .clone()
    }};
}

macro_rules! swap_node {
    ($node_a:expr,$node_b:expr) => {
        unsafe {
            let parent_a = (*$node_a).parent;
            let parent_b = (*$node_b).parent;
            let idx_a = (*parent_a)
                .children
                .iter()
                .position(|x| *x == $node_a)
                .unwrap();
            let idx_b = (*parent_b)
                .children
                .iter()
                .position(|x| *x == $node_b)
                .unwrap();
            (*$node_a).parent = parent_b;
            (*$node_b).parent = parent_a;
            (*parent_a).children[idx_a] = $node_b;
            (*parent_b).children[idx_b] = $node_a;
            (*parent_a).update_metadata_up(idx_a);
            (*parent_b).update_metadata_up(idx_b);
        }
    };
}

use libafl_bolts::bolts_prelude::{tuple_list, tuple_list_type};
pub(crate) use random_input_from_corpus;
pub(crate) use random_input_from_sequence;
pub(crate) use swap_node;
use crate::mutators::byte::{ByteDuplicateMutator, ByteInsertMutator, ByteRemoveMutator, ByteSpliceMutator};
use crate::mutators::message::{MessageFieldLineDuplicateMutator, MessageFieldLineRemoveMutator, MessageFieldLineSpliceMutator, MessageNodeTokenReplaceMutator, MessageNodeTypedSwapMutator, MessageTrailerSectionReplaceMutator};
use crate::mutators::sequence::{SequenceRemoveMutator, SequenceSpliceMutator};

pub type HttpMutatorsTupleType = tuple_list_type!(
    // Byte-level mutators
    ByteInsertMutator,
    ByteRemoveMutator,
    ByteDuplicateMutator,
    ByteSpliceMutator,
    // Message-level mutators
    MessageFieldLineDuplicateMutator,
    MessageFieldLineRemoveMutator,
    MessageFieldLineSpliceMutator,
    MessageNodeTypedSwapMutator,
    MessageTrailerSectionReplaceMutator,
    MessageNodeTokenReplaceMutator,
    // Sequence-level mutators
    SequenceSpliceMutator,
    SequenceRemoveMutator,
);

pub fn http_mutations() -> HttpMutatorsTupleType {
    tuple_list!(
        ByteInsertMutator::new(),
        ByteRemoveMutator::new(),
        ByteDuplicateMutator::new(),
        ByteSpliceMutator::new(),
        MessageFieldLineDuplicateMutator::new(),
        MessageFieldLineRemoveMutator::new(),
        MessageFieldLineSpliceMutator::new(),
        MessageNodeTypedSwapMutator::new(),
        MessageTrailerSectionReplaceMutator::new(),
        MessageNodeTokenReplaceMutator::new(),
        SequenceSpliceMutator::new(),
        SequenceRemoveMutator::new(),
    )
}

pub type HttpRemoveMutatorsTupleType = tuple_list_type!(
    ByteRemoveMutator,
    MessageFieldLineRemoveMutator,
    SequenceRemoveMutator,
);

pub fn http_remove_mutations() -> HttpRemoveMutatorsTupleType {
    tuple_list!(
        ByteRemoveMutator::new(),
        MessageFieldLineRemoveMutator::new(),
        SequenceRemoveMutator::new()
    )
}
