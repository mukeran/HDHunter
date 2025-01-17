use std::{io::{Read, Write}, net::TcpStream, process::Stdio, time::Duration};

use libafl::{executors::{Executor, ExitKind, HasObservers}, inputs::HasTargetBytes, observers::{ObserversTuple, UsesObservers}, state::{HasExecutions, State, UsesState}};
use libafl_bolts::{tuples::{MatchName, RefIndexable}, AsSlice};

/// For experiment only, please use `STNyxExecutor` in production.
pub struct NetworkRestartExecutor<OT, S> {
    start_command: String,
    stop_command: String,
    envs: Vec<(String, String)>,
    port: u16,
    timeout: Duration,
    observers: OT,
    phantom: std::marker::PhantomData<S>
}

impl<OT, S> NetworkRestartExecutor<OT, S> {
    pub fn new(start_command: &str, stop_command: &str, envs: Vec<(String, String)>, port: u16, timeout: Duration, observers: OT) -> Self {
        Self {
            start_command: start_command.to_string(),
            stop_command: stop_command.to_string(),
            envs,
            port,
            timeout,
            observers,
            phantom: std::marker::PhantomData
        }
    }
}

impl<OT, S> UsesState for NetworkRestartExecutor<OT, S>
where
    S: State {
    type State = S;
}

impl<OT, S> UsesObservers for NetworkRestartExecutor<OT, S>
where
    OT: ObserversTuple<S>,
    S: State,
{
    type Observers = OT;
}

impl<OT, S> HasObservers for NetworkRestartExecutor<OT, S>
where
    S: State,
    OT: ObserversTuple<S>,
{
    fn observers(&self) -> RefIndexable<&Self::Observers, Self::Observers> {
        RefIndexable::from(&self.observers)
    }

    fn observers_mut(&mut self) -> RefIndexable<&mut Self::Observers, Self::Observers> {
        RefIndexable::from(&mut self.observers)
    }
}

// fn connect_with_retry<A: ToSocketAddrs>(addr: A, max_attempts: usize, delay: Duration) -> Result<TcpStream, std::io::Error> {
//     let mut attempts = 0;
//     loop {
//         match TcpStream::connect(&addr) {
//             Ok(stream) => return Ok(stream),
//             Err(_) if attempts < max_attempts => {
//                 // println!("Attempt {} failed, retrying in {:?}...", attempts, delay);
//                 sleep(delay);
//                 attempts += 1;
//             },
//             Err(e) => return Err(e),
//         }
//     }
// }

impl<EM, OT, S, Z> Executor<EM, Z> for NetworkRestartExecutor<OT, S>
where
    EM: UsesState<State = S>,
    S: State + HasExecutions,
    S::Input: HasTargetBytes,
    OT: MatchName + ObserversTuple<S>,
    Z: UsesState<State = S> {
    fn run_target(
        &mut self,
        _fuzzer: &mut Z,
        state: &mut Self::State,
        _mgr: &mut EM,
        input: &Self::Input,
    ) -> Result<libafl::prelude::ExitKind, libafl::prelude::Error> {
        *state.executions_mut() += 1;
        for (key, value) in &self.envs {
            std::env::set_var(key, value);
        }
        std::process::Command::new("sh").arg("-c").arg(&self.start_command)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();

        let mut socket = TcpStream::connect(format!("127.0.0.1:{}", self.port)).unwrap();
        socket.set_read_timeout(Some(self.timeout)).unwrap();
        socket.write_all(input.target_bytes().as_slice()).unwrap();
        let mut buf = [0; 1024];
        let _ = socket.read(&mut buf);
        let _ = socket.shutdown(std::net::Shutdown::Both);

        std::process::Command::new("sh").arg("-c").arg(&self.stop_command)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();

        Ok(ExitKind::Ok)
    }
}