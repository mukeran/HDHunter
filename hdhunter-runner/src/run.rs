use std::path::{Path, PathBuf};
use hdhunter::executors::STNyxExecutor;
use hdhunter::mode::Mode;
use hdhunter::mutators::message::TokenMetadata;
use libafl::corpus::{Corpus, CachedOnDiskCorpus, OnDiskCorpus};
use libafl::events::SimpleEventManager;
use libafl::executors::DiffExecutor;
use libafl::feedbacks::{MaxMapFeedback, TimeFeedback};
use libafl::monitors::SimpleMonitor;
use libafl::mutators::StdScheduledMutator;
use libafl::observers::{CanTrack, HitcountsIterableMapObserver, MultiMapObserver, TimeObserver};
use libafl::schedulers::powersched::PowerSchedule;
use libafl::stages::{CalibrationStage, StdPowerMutationalStage};
use libafl::{feedback_or, Fuzzer, HasMetadata, StdFuzzer};
use libafl::schedulers::{IndexesLenTimeMinimizerScheduler, StdWeightedScheduler};
use libafl::state::{HasCorpus, StdState};
use libafl_bolts::tuples::tuple_list;
use libafl_bolts::current_nanos;
use libafl_bolts::ownedref::OwnedMutSlice;
use libafl_bolts::rands::StdRand;
use libafl_nyx::executor::NyxExecutorBuilder;
use libafl_nyx::helper::NyxHelper;
use libafl_nyx::settings::NyxSettings;
use log::info;
use hdhunter::feedbacks::HttpParamFeedback;
use hdhunter::input::{set_mode, HttpSequenceInput};
use hdhunter::mutators::http_mutations;
use hdhunter::observers::HttpParamObserver;

pub(crate) fn run(mode: &Mode, first_path: &str, second_path: &str, seeds: &[String], corpus: &str, solutions: &str, tokens: &str) {
    set_mode(mode);

    let helper = (
        NyxHelper::new(
            Path::new(first_path), NyxSettings::builder().cpu_id(0).parent_cpu_id(None).build()).unwrap(),
        NyxHelper::new(
            Path::new(second_path), NyxSettings::builder().cpu_id(0).parent_cpu_id(None).build()).unwrap(),
    );
    let map_observer = HitcountsIterableMapObserver::new(
        MultiMapObserver::differential(
            "combined-edges",
            vec![
                unsafe { OwnedMutSlice::from_raw_parts_mut(helper.0.bitmap_buffer, helper.0.bitmap_size) },
                unsafe { OwnedMutSlice::from_raw_parts_mut(helper.1.bitmap_buffer, helper.1.bitmap_size) },
            ],
        )
    ).track_indices();

    let http_param_observer = (
        HttpParamObserver::new("state_tuple_first", helper.0.http_param_buffer),
        HttpParamObserver::new("state_tuple_second", helper.1.http_param_buffer),
    );

    let time_observer = TimeObserver::new("time");

    let mut objective = HttpParamFeedback::new("http_param", &http_param_observer);

    let map_feedback = MaxMapFeedback::new(&map_observer);
    let calibration = CalibrationStage::new(&map_feedback);

    let mut feedback = feedback_or!(
        map_feedback,
        TimeFeedback::new(&time_observer)
    );

    let mut state = StdState::new(
        StdRand::with_seed(current_nanos()),
        CachedOnDiskCorpus::<HttpSequenceInput>::new(PathBuf::from(corpus), 64).unwrap(),
        OnDiskCorpus::new(PathBuf::from(solutions)).unwrap(),
        &mut feedback,
        &mut objective,
    ).unwrap();

    let tokens: TokenMetadata = serde_json::from_reader(std::fs::File::open(tokens).unwrap()).unwrap();
    state.add_metadata(tokens);

    let scheduler = IndexesLenTimeMinimizerScheduler::new(&map_observer, StdWeightedScheduler::with_schedule(&mut state, &map_observer, Some(PowerSchedule::EXPLORE)));
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    let monitor = SimpleMonitor::with_user_monitor(|s| println!("{}", s));

    let mut mgr = SimpleEventManager::new(monitor);
    let mutator = StdScheduledMutator::new(http_mutations());

    let mut stages = tuple_list!(
        calibration,
        StdPowerMutationalStage::new(mutator)
    );
    let mut executor = DiffExecutor::new(
        STNyxExecutor::new(NyxExecutorBuilder::new().build(helper.0, tuple_list!(http_param_observer.0))),
        STNyxExecutor::new(NyxExecutorBuilder::new().build(helper.1, tuple_list!(http_param_observer.1))),
        tuple_list!(map_observer, time_observer)
    );

    let corpus_dirs: Vec<PathBuf> = seeds.iter().map(|s| PathBuf::from(s)).collect();
    state.load_initial_inputs(&mut fuzzer, &mut executor, &mut mgr, &corpus_dirs)
        .unwrap_or_else(|e| {
            panic!("Failed to load initial inputs: {}", e);
        });
    info!("Loaded {} inputs", state.corpus().count());

    fuzzer
        .fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)
        .expect("Error in the fuzzing loop");
}
