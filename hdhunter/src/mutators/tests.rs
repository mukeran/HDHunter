use std::vec;

use crate::input::{HttpInput, HttpSequenceInput, Node, NodeType};
use crate::{mutators, new_node};
use libafl::corpus::{Corpus, InMemoryCorpus};
use libafl::feedbacks::ConstFeedback;
use libafl::inputs::HasTargetBytes;
use libafl::mutators::{MutationResult, Mutator, MutatorsTuple};
use libafl::state::{HasCorpus, HasMaxSize, HasRand, StdState};
use libafl::HasMetadata;
use libafl_bolts::{AsSlice, HasLen};
use libafl_bolts::ownedref::OwnedSlice;
use libafl_bolts::rands::StdRand;
use log::debug;
use similar::{Algorithm, capture_diff_slices, DiffOp, DiffTag};
use crate::mutators::byte::{ByteDuplicateMutator, ByteInsertMutator, ByteRemoveMutator, ByteSpliceMutator};
use crate::mutators::message::{MessageFieldLineDuplicateMutator, MessageFieldLineRemoveMutator, MessageFieldLineSpliceMutator, MessageNodeTypedSwapMutator, MessageTrailerSectionReplaceMutator};
use crate::mutators::sequence::{SequenceRemoveMutator, SequenceSpliceMutator};

use super::message::{MessageNodeTokenReplaceMutator, TokenMetadata};

fn test_state() -> impl HasCorpus<Input = HttpSequenceInput> + HasRand + HasMaxSize + HasMetadata {
    let rand = StdRand::with_seed(1337);
    let mut corpus = InMemoryCorpus::new();
    let mut feedback = ConstFeedback::new(false);
    let mut objective = ConstFeedback::new(false);

    corpus
        .add(
            HttpSequenceInput::new(vec![
                HttpInput::new(new_node!(
                    NodeType::HttpMessage,
                    new_node!(
                        NodeType::StartLine,
                        new_node!(
                            NodeType::RequestLine,
                            new_node!(NodeType::RawBytes; b"GET".into()),
                            new_node!(NodeType::SP; b" ".into()),
                            new_node!(NodeType::RawBytes; b"/first".into()),
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
                            new_node!(NodeType::RawBytes; b"a.com".into()),
                            new_node!(NodeType::OWSBWS; b"".into()),
                            new_node!(NodeType::CRLF; b"\r\n".into()),
                        ),
                    ),
                    new_node!(NodeType::CRLF; b"\r\n".into()),
                    new_node!(
                        NodeType::MessageBody,
                        new_node!(NodeType::RawBytes; b"".into()),
                    ),
                ))
                .unwrap(),
                HttpInput::new(new_node!(
                    NodeType::HttpMessage,
                    new_node!(
                        NodeType::StartLine,
                        new_node!(
                            NodeType::RequestLine,
                            new_node!(NodeType::RawBytes; b"POST".into()),
                            new_node!(NodeType::SP; b" ".into()),
                            new_node!(NodeType::RawBytes; b"/second".into()),
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
                            new_node!(NodeType::RawBytes; b"b.com".into()),
                            new_node!(NodeType::OWSBWS; b"".into()),
                            new_node!(NodeType::CRLF; b"\r\n".into()),
                        ),
                        new_node!(
                            NodeType::FieldLine,
                            new_node!(NodeType::RawBytes; b"Transfer-Encoding".into()),
                            new_node!(NodeType::COLON; b":".into()),
                            new_node!(NodeType::OWSBWS; b" ".into()),
                            new_node!(NodeType::RawBytes; b"chunked".into()),
                            new_node!(NodeType::OWSBWS; b"".into()),
                            new_node!(NodeType::CRLF; b"\r\n".into()),
                        ),
                    ),
                    new_node!(NodeType::CRLF; b"\r\n".into()),
                    new_node!(
                        NodeType::MessageBody,
                        new_node!(
                            NodeType::ChunkedBody,
                            new_node!(
                                NodeType::Chunks,
                                new_node!(
                                    NodeType::Chunk,
                                    new_node!(
                                        NodeType::ChunkSize,
                                        new_node!(NodeType::HEXDIG; b"1".into()),
                                        new_node!(NodeType::HEXDIG; b"0".into()),
                                    ),
                                    new_node!(NodeType::RawBytes; b"".into()),
                                    new_node!(NodeType::CRLF; b"\r\n".into()),
                                    new_node!(NodeType::RawBytes; b"1234567890abcdef".into()),
                                    new_node!(NodeType::CRLF; b"\r\n".into()),
                                ),
                            ),
                            new_node!(
                                NodeType::LastChunk,
                                new_node!(
                                    NodeType::ChunkSize,
                                    new_node!(NodeType::HEXDIG; b"0".into()),
                                ),
                                new_node!(NodeType::RawBytes; b"".into()),
                                new_node!(NodeType::CRLF; b"\r\n".into()),
                            ),
                            new_node!(NodeType::TrailerSection,),
                            new_node!(NodeType::CRLF; b"\r\n".into()),
                        ),
                    ),
                ))
                .unwrap(),
            ])
            .into(),
        )
        .unwrap();

    let mut state = StdState::new(
        rand,
        corpus,
        InMemoryCorpus::new(),
        &mut feedback,
        &mut objective,
    )
    .unwrap();

    state.add_metadata(TokenMetadata {
        string: vec!["Content-Encoding".into(), "Random".into()],
        number: vec!["1".into(), "2".into()],
        symbol: vec!["!".into(), "?".into()],
    });

    state
}

fn test_inputs() -> Vec<HttpSequenceInput> { vec![
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
                new_node!(NodeType::RawBytes; b"random message".into()),
            ),
        )).unwrap(),
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
                new_node!(NodeType::RawBytes; b"some message".into()),
            ),
        )).unwrap(),
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
                    new_node!(NodeType::RawBytes; b"any.com".into()),
                    new_node!(NodeType::OWSBWS; b"".into()),
                    new_node!(NodeType::CRLF; b"\r\n".into()),
                ),
                new_node!(
                    NodeType::FieldLine,
                    new_node!(NodeType::RawBytes; b"Transfer-Encoding".into()),
                    new_node!(NodeType::COLON; b":".into()),
                    new_node!(NodeType::OWSBWS; b" ".into()),
                    new_node!(NodeType::RawBytes; b"chunked".into()),
                    new_node!(NodeType::OWSBWS; b"".into()),
                    new_node!(NodeType::CRLF; b"\r\n".into()),
                ),
            ),
            new_node!(NodeType::CRLF; b"\r\n".into()),
            new_node!(
                NodeType::MessageBody,
                new_node!(
                    NodeType::ChunkedBody,
                    new_node!(
                        NodeType::Chunks,
                        new_node!(
                            NodeType::Chunk,
                            new_node!(
                                NodeType::ChunkSize,
                                new_node!(NodeType::HEXDIG; b"1".into()),
                                new_node!(NodeType::HEXDIG; b"0".into()),
                            ),
                            new_node!(NodeType::RawBytes; b"".into()),
                            new_node!(NodeType::CRLF; b"\r\n".into()),
                            new_node!(NodeType::RawBytes; b"1234567890abcdef".into()),
                            new_node!(NodeType::CRLF; b"\r\n".into()),
                        ),
                    ),
                    new_node!(
                        NodeType::LastChunk,
                        new_node!(
                            NodeType::ChunkSize,
                            new_node!(NodeType::HEXDIG; b"0".into()),
                        ),
                        new_node!(NodeType::RawBytes; b"".into()),
                        new_node!(NodeType::CRLF; b"\r\n".into()),
                    ),
                    new_node!(NodeType::TrailerSection,),
                    new_node!(NodeType::CRLF; b"\r\n".into()),
                ),
            ),
        )).unwrap(),
    ])
] }

#[test]
fn test_mutators() {
    let mut inputs = test_inputs();
    let mut state = test_state();
    let mut mutations = mutators::http_mutations();

    for _ in 0..2 {
        let mut new_testcases = vec![];
        for idx in 0..mutations.len() {
            for input in inputs.iter() {
                let mut mutant = input.clone();
                match mutations
                    .get_and_mutate(idx.into(), &mut state, &mut mutant)
                    .unwrap()
                {
                    MutationResult::Mutated => new_testcases.push(mutant),
                    MutationResult::Skipped => (),
                }
            }
        }
        inputs.append(&mut new_testcases);
    }
}

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

#[test]
fn test_byte_insert() {
    let base = test_inputs()[0].clone();
    let mut state = test_state();
    let mut mutator = ByteInsertMutator::new();
    let rounds = 100;

    for _ in 0..rounds {
        let mut mutated = base.clone();
        if mutator.mutate(&mut state, &mut mutated).unwrap() == MutationResult::Skipped {
            continue;
        }
        debug!("{}", escaped!(mutated.target_bytes().as_slice()));
        let ops = capture_diff_slices(Algorithm::Myers, base.target_bytes().as_slice(), mutated.target_bytes().as_slice());
        assert_eq!(ops.iter().filter(|x| { x.tag() == DiffTag::Insert }).count(), 1);
    }
}

#[test]
fn test_byte_remove() {
    let base = test_inputs()[0].clone();
    let mut state = test_state();
    let mut mutator = ByteRemoveMutator::new();
    let rounds = 100;

    for _ in 0..rounds {
        let mut mutated = base.clone();
        if mutator.mutate(&mut state, &mut mutated).unwrap() == MutationResult::Skipped {
            continue;
        }
        debug!("{}", escaped!(mutated.target_bytes().as_slice()));
        let ops = capture_diff_slices(Algorithm::Myers, base.target_bytes().as_slice(), mutated.target_bytes().as_slice());
        assert_eq!(ops.iter().filter(|x| { x.tag() == DiffTag::Delete }).count(), 1);
    }
}

#[test]
fn test_byte_duplicate() {
    let base = test_inputs()[0].clone();
    let mut state = test_state();
    let mut mutator = ByteDuplicateMutator::new();
    let rounds = 100;

    for _ in 0..rounds {
        let mut mutated = base.clone();
        if mutator.mutate(&mut state, &mut mutated).unwrap() == MutationResult::Skipped {
            continue;
        }
        debug!("{}", escaped!(mutated.target_bytes().as_slice()));
        let ops = capture_diff_slices(Algorithm::Myers, base.target_bytes().as_slice(), mutated.target_bytes().as_slice());
        let insert = ops.iter().filter(|x| { x.tag() == DiffTag::Insert }).collect::<Vec<&DiffOp>>();
        assert_eq!(insert.len(), 1);
        assert_eq!(
            mutated.target_bytes().as_slice()[insert[0].as_tag_tuple().1.start - 1],
            mutated.target_bytes().as_slice()[insert[0].as_tag_tuple().2.start]
        )
    }
}

#[test]
fn test_byte_splice() {
    let base = test_inputs()[0].clone();
    let mut state = test_state();
    let mut mutator = ByteSpliceMutator::new();
    let rounds = 100;

    for _ in 0..rounds {
        let mut mutated = base.clone();
        if mutator.mutate(&mut state, &mut mutated).unwrap() == MutationResult::Skipped {
            continue;
        }
        debug!("{}", escaped!(mutated.target_bytes().as_slice()));
        let ops = capture_diff_slices(Algorithm::Myers, base.target_bytes().as_slice(), mutated.target_bytes().as_slice());
        debug!("{:?}", ops);
        assert_eq!(ops.iter().filter(|x| { x.tag() == DiffTag::Insert }).count(), 1);
    }
}

#[test]
fn test_message_field_line_duplicate() {
    let base = test_inputs()[0].clone();
    let mut state = test_state();
    let mut mutator = MessageFieldLineDuplicateMutator::new();
    let rounds = 100;

    for _ in 0..rounds {
        let mut mutated = base.clone();
        if mutator.mutate(&mut state, &mut mutated).unwrap() == MutationResult::Skipped {
            continue;
        }
        debug!("{}", escaped!(mutated.target_bytes().as_slice()));
        let mut ops: Vec<DiffOp> = vec![];
        for (idx, input) in base.inputs.iter().enumerate() {
            let old: Vec<OwnedSlice<u8>> = input.field_lines().children.iter()
                .map(|x| unsafe { (**x).bytes() }).collect();
            let new: Vec<OwnedSlice<u8>> = mutated.inputs[idx].field_lines().children.iter()
                .map(|x| unsafe { (**x).bytes() }).collect();
            let old: Vec<&[u8]> = old.iter().map(|x| x.as_slice()).collect();
            let new: Vec<&[u8]> = new.iter().map(|x| x.as_slice()).collect();
            ops.extend(capture_diff_slices(
                Algorithm::Myers,
                &old,
                &new,
            ));
        }
        debug!("{:?}", ops);
        assert_eq!(ops.iter().filter(|x| { x.tag() == DiffTag::Insert }).count(), 1);
    }
}

#[test]
fn test_message_field_line_remove() {
    let base = test_inputs()[0].clone();
    let mut state = test_state();
    let mut mutator = MessageFieldLineRemoveMutator::new();
    let rounds = 100;

    for _ in 0..rounds {
        let mut mutated = base.clone();
        if mutator.mutate(&mut state, &mut mutated).unwrap() == MutationResult::Skipped {
            continue;
        }
        debug!("{}", escaped!(mutated.target_bytes().as_slice()));
        let mut ops: Vec<DiffOp> = vec![];
        for (idx, input) in base.inputs.iter().enumerate() {
            let old: Vec<OwnedSlice<u8>> = input.field_lines().children.iter()
                .map(|x| unsafe { (**x).bytes() }).collect();
            let new: Vec<OwnedSlice<u8>> = mutated.inputs[idx].field_lines().children.iter()
                .map(|x| unsafe { (**x).bytes() }).collect();
            let old: Vec<&[u8]> = old.iter().map(|x| x.as_slice()).collect();
            let new: Vec<&[u8]> = new.iter().map(|x| x.as_slice()).collect();
            ops.extend(capture_diff_slices(
                Algorithm::Myers,
                &old,
                &new,
            ));
        }
        debug!("{:?}", ops);
        assert_eq!(ops.iter().filter(|x| { x.tag() == DiffTag::Delete }).count(), 1);
    }
}

#[test]
fn test_message_field_line_splice() {
    let base = test_inputs()[0].clone();
    let mut state = test_state();
    let mut mutator = MessageFieldLineSpliceMutator::new();
    let rounds = 100;

    for _ in 0..rounds {
        let mut mutated = base.clone();
        if mutator.mutate(&mut state, &mut mutated).unwrap() == MutationResult::Skipped {
            continue;
        }
        debug!("{}", escaped!(mutated.target_bytes().as_slice()));
        let mut ops: Vec<DiffOp> = vec![];
        for (idx, input) in base.inputs.iter().enumerate() {
            let old: Vec<OwnedSlice<u8>> = input.field_lines().children.iter()
                .map(|x| unsafe { (**x).bytes() }).collect();
            let new: Vec<OwnedSlice<u8>> = mutated.inputs[idx].field_lines().children.iter()
                .map(|x| unsafe { (**x).bytes() }).collect();
            let old: Vec<&[u8]> = old.iter().map(|x| x.as_slice()).collect();
            let new: Vec<&[u8]> = new.iter().map(|x| x.as_slice()).collect();
            ops.extend(capture_diff_slices(
                Algorithm::Myers,
                &old,
                &new,
            ));
        }
        debug!("{:?}", ops);
        assert_eq!(ops.iter().filter(|x| { x.tag() == DiffTag::Insert }).count(), 1);
    }
}

#[test]
fn test_message_node_typed_swap() {
    let mut base = test_inputs()[0].clone();
    let mut state = test_state();
    let mut mutator = MessageNodeTypedSwapMutator::new();
    let rounds = 100;

    debug!("base: {}", escaped!(base.target_bytes().as_slice()));

    for _ in 0..rounds {
        let mut mutated = base.clone();
        if mutator.mutate(&mut state, &mut mutated).unwrap() == MutationResult::Skipped {
            continue;
        }
        debug!("{}", escaped!(mutated.target_bytes().as_slice()));
        let ops = capture_diff_slices(Algorithm::Myers, base.target_bytes().as_slice(), mutated.target_bytes().as_slice());
        debug!("{:?}", ops);
        // TODO: check that the node types have been swapped
        base = mutated;
    }
}

#[test]
fn test_message_trailer_section_replace() {
    let base = test_inputs()[1].clone();
    let mut state = test_state();
    let mut mutator = MessageTrailerSectionReplaceMutator::new();
    let rounds = 100;

    for _ in 0..rounds {
        let mut mutated = base.clone();
        if mutator.mutate(&mut state, &mut mutated).unwrap() == MutationResult::Skipped {
            continue;
        }
        debug!("{}", escaped!(mutated.target_bytes().as_slice()));
        let mut ops: Vec<DiffOp> = vec![];
        for (idx, input) in base.inputs.iter().enumerate() {
            let message_body = input.message_body();
            unsafe {
                if message_body.node_type != NodeType::MessageBody
                    || (*message_body.children[0]).node_type != NodeType::ChunkedBody
                    || (*(*message_body.children[0]).children[2]).node_type != NodeType::TrailerSection {
                    continue;
                }
            }

            let old = unsafe { &*(*message_body.children[0]).children[2] };
            let new = unsafe { &*(*mutated.inputs[idx].message_body().children[0]).children[2] };

            ops.extend(capture_diff_slices(
                Algorithm::Myers,
                old.bytes().as_slice(),
                new.bytes().as_slice(),
            ));
        }
        debug!("{:?}", ops);
        assert_eq!(ops.iter().filter(|x| { x.tag() == DiffTag::Insert }).count(), 1);
    }
}

#[test]
fn test_message_node_token_replace() {
    let base = test_inputs()[0].clone();
    let mut state = test_state();
    let mut mutator = MessageNodeTokenReplaceMutator::new();
    let rounds = 100;

    for _ in 0..rounds {
        let mut mutated = base.clone();
        if mutator.mutate(&mut state, &mut mutated).unwrap() == MutationResult::Skipped {
            continue;
        }
        debug!("{}", escaped!(mutated.target_bytes().as_slice()));
    }
}

#[test]
fn test_sequence_splice() {
    let base = test_inputs()[0].clone();
    let mut state = test_state();
    let mut mutator = SequenceSpliceMutator::new();
    let rounds = 100;

    for _ in 0..rounds {
        let mut mutated = base.clone();
        if mutator.mutate(&mut state, &mut mutated).unwrap() == MutationResult::Skipped {
            continue;
        }
        debug!("{}", escaped!(mutated.target_bytes().as_slice()));
        let old: Vec<OwnedSlice<u8>> = base.inputs.iter().map(|x| x.target_bytes()).collect();
        let new: Vec<OwnedSlice<u8>> = mutated.inputs.iter().map(|x| x.target_bytes()).collect();
        let old: Vec<&[u8]> = old.iter().map(|x| x.as_slice()).collect();
        let new: Vec<&[u8]> = new.iter().map(|x| x.as_slice()).collect();
        let ops = capture_diff_slices(
            Algorithm::Myers,
            &old,
            &new,
        );
        debug!("{:?}", ops);
        assert_eq!(ops.iter().filter(|x| { x.tag() == DiffTag::Insert }).count(), 1);
    }
}

#[test]
fn test_sequence_remove() {
    let base = test_inputs()[0].clone();
    let mut state = test_state();
    let mut mutator = SequenceRemoveMutator::new();
    let rounds = 100;

    for _ in 0..rounds {
        let mut mutated = base.clone();
        if mutator.mutate(&mut state, &mut mutated).unwrap() == MutationResult::Skipped {
            continue;
        }
        debug!("{}", escaped!(mutated.target_bytes().as_slice()));
        let old: Vec<OwnedSlice<u8>> = base.inputs.iter().map(|x| x.target_bytes()).collect();
        let new: Vec<OwnedSlice<u8>> = mutated.inputs.iter().map(|x| x.target_bytes()).collect();
        let old: Vec<&[u8]> = old.iter().map(|x| x.as_slice()).collect();
        let new: Vec<&[u8]> = new.iter().map(|x| x.as_slice()).collect();
        let ops = capture_diff_slices(
            Algorithm::Myers,
            &old,
            &new,
        );
        debug!("{:?}", ops);
        assert_eq!(ops.iter().filter(|x| { x.tag() == DiffTag::Delete }).count(), 1);
    }
}
