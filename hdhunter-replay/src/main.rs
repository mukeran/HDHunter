mod args;

use std::env;
use std::path::Path;
use clap::Parser;
use hdhunter::executors::STNyxExecutor;
use hdhunter::mode::Mode;
use libafl::corpus::NopCorpus;
use libafl::events::NopEventManager;
use libafl::feedbacks::ConstFeedback;
use libafl::inputs::Input;
use libafl::observers::{MapObserver, Observer, StdMapObserver};
use libafl::schedulers::{RandScheduler, StdScheduler};
use libafl::state::StdState;
use libafl::StdFuzzer;
use libafl::executors::{Executor, ExitKind, HasObservers};
use libafl_bolts::rands::{RomuDuoJrRand, StdRand};
use libafl_bolts::tuples::{tuple_list, Handled, MatchNameRef};
use libafl_nyx::executor::NyxExecutorBuilder;
use libafl_nyx::helper::NyxHelper;
use libafl_nyx::settings::NyxSettings;
use log::info;
use hdhunter::input::{set_mode, HttpSequenceInput};
use hdhunter::observers::{HttpParam, HttpParamObserver};
use crate::args::Args;

fn run_single(workspace: &str, input: &HttpSequenceInput, mode: &Mode) -> (ExitKind, Vec<u8>, HttpParam) {
    set_mode(mode);
    let helper = NyxHelper::new(Path::new(workspace), NyxSettings::builder().cpu_id(0).parent_cpu_id(None).build()).unwrap();
    let map_observer =
        unsafe { StdMapObserver::from_mut_ptr("trace", helper.bitmap_buffer, helper.bitmap_size) };
    let http_param_observer = HttpParamObserver::new("http_param", helper.http_param_buffer);

    let map_observer_handle = map_observer.handle();
    let http_param_observer_handle = http_param_observer.handle();

    let mut const_feedback = ConstFeedback::new(false);
    let mut const_objective = ConstFeedback::new(false);
    let mut state = StdState::new(
        StdRand::with_seed(0),
        NopCorpus::new(),
        NopCorpus::new(),
        &mut const_feedback,
        &mut const_objective
    ).unwrap();
    let scheduler = StdScheduler::new();
    let mut mgr = NopEventManager::new();
    let mut fuzzer: StdFuzzer<
        RandScheduler<
            StdState<
                HttpSequenceInput,
                NopCorpus<HttpSequenceInput>,
                RomuDuoJrRand,
                NopCorpus<HttpSequenceInput>
            >
        >,
        ConstFeedback, ConstFeedback, ()
    > = StdFuzzer::new(scheduler, const_feedback, const_objective);
    let mut executor = STNyxExecutor::new(NyxExecutorBuilder::new().build(helper,
                                        tuple_list!(map_observer, http_param_observer)));
    let mut observers = executor.observers_mut();
    let map_observer: &mut StdMapObserver<u8, false> = observers.get_mut(&map_observer_handle).unwrap();
    map_observer.pre_exec(&mut state, input).unwrap();
    let http_param_observer: &mut HttpParamObserver = observers.get_mut(&http_param_observer_handle).unwrap();
    http_param_observer.pre_exec(&mut state, input).unwrap();

    let exit_kind = executor.run_target(&mut fuzzer, &mut state, &mut mgr, input).unwrap();
    let observers = executor.observers_mut();
    let map_observer: &StdMapObserver<u8, false> = observers.get(&map_observer_handle).unwrap();
    let http_param_observer: &HttpParamObserver = observers.get(&http_param_observer_handle).unwrap();

    let edgemap = map_observer.to_vec();
    let http_param = HttpParam::clone(http_param_observer.http_param());
    executor.parent.helper.nyx_process.shutdown();

    (exit_kind, edgemap, http_param)
}

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init_timed();
    let args = Args::parse();

    let input = HttpSequenceInput::from_file(&args.input).unwrap();
    let (exit, edgemap, http_param) = run_single(&args.workspace, &input, &args.mode);
    info!("ExitKind: {:?}", exit);
    info!("HttpParam: {:?}", http_param);
    if args.print_edgemap {
        info!("EdgeMap: {:?}", edgemap);
    }
}
