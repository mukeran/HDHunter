// use std::{io::Write, process::exit};

use libafl::{executors::{Executor, ExitKind, HasObservers}, inputs::{HasTargetBytes, UsesInput}, observers::{ObserversTuple, UsesObservers}, state::{HasExecutions, State, UsesState}};
use libafl_bolts::tuples::RefIndexable;
use libafl_nyx::executor::NyxExecutor;

use crate::{input::{HttpSequenceInput, IsHttpSequenceInput, NodeType, Node}, new_node};

pub struct STNyxExecutor<S, OT>
where
    S: State,
    S::Input: IsHttpSequenceInput
{
    pub parent: NyxExecutor<S, OT>
}

impl<S, OT> STNyxExecutor<S, OT>
where
    S: State,
    S::Input: IsHttpSequenceInput
{
    pub fn new(parent: NyxExecutor<S, OT>) -> Self {
        Self {
            parent
        }
    }
}

impl<S, OT> UsesState for STNyxExecutor<S, OT>
where
    S: State + UsesInput<Input = HttpSequenceInput>,
{
    type State = S;
}

impl<S, OT> UsesObservers for STNyxExecutor<S, OT>
where
    OT: ObserversTuple<S>,
    S: State + UsesInput<Input = HttpSequenceInput>,
{
    type Observers = OT;
}

impl<S, OT> HasObservers for STNyxExecutor<S, OT>
where
    S: State + UsesInput<Input = HttpSequenceInput>,
    OT: ObserversTuple<S>,
{
    fn observers(&self) -> RefIndexable<&Self::Observers, Self::Observers> {
        self.parent.observers()
    }

    fn observers_mut(&mut self) -> RefIndexable<&mut Self::Observers, Self::Observers> {
        self.parent.observers_mut()
    }
}

impl<EM, Z, S, OT> Executor<EM, Z> for STNyxExecutor<S, OT>
where 
    EM: UsesState<State = S>,
    S: State + HasExecutions + UsesInput<Input = HttpSequenceInput>,
    S::Input: HasTargetBytes + IsHttpSequenceInput,
    Z: UsesState<State = S>,
    OT: ObserversTuple<S> {
    fn run_target(
        &mut self,
        fuzzer: &mut Z,
        state: &mut Self::State,
        mgr: &mut EM,
        input: &Self::Input,
    ) -> Result<ExitKind, libafl::Error> {
        let mut input = input.http_sequence_input().clone();
        for (idx, msg) in input.inputs.iter_mut().enumerate() {
            msg.field_lines_mut().add_child(
                new_node!(
                    NodeType::FieldLine,
                    new_node!(NodeType::RawBytes; b"X-Desync-Id".into()),
                    new_node!(NodeType::COLON; b":".into()),
                    new_node!(NodeType::OWSBWS; b" ".into()),
                    new_node!(NodeType::RawBytes; (idx + 1).to_string().into()),
                    new_node!(NodeType::OWSBWS; b"".into()),
                    new_node!(NodeType::CRLF; b"\r\n".into()),
                )
            );
        }
        self.parent.run_target(fuzzer, state, mgr, &input)
    }
}