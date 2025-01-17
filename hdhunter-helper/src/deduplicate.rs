use std::path::{Path, PathBuf};

use hdhunter::input::HttpSequenceInput;
use libafl::{corpus::{Corpus, InMemoryCorpus, InMemoryOnDiskCorpus}, events::SimpleEventManager, executors::DiffExecutor, feedbacks::MaxMapFeedback, inputs::Input, monitors::SimpleMonitor, observers::{HitcountsIterableMapObserver, MultiMapObserver}, schedulers::QueueScheduler, state::{HasCorpus, StdState}, Evaluator, StdFuzzer};
use libafl_bolts::{current_nanos, ownedref::OwnedMutSlice, rands::StdRand, tuples::tuple_list};
use libafl_nyx::{executor::NyxExecutorBuilder, helper::NyxHelper, settings::NyxSettings};
use log::info;

pub(crate) fn deduplicate(first_path: &str, second_path: &str, solutions_path: &str, output_path: &str) {
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
            ]
        )
    );
    let mut map_feedback = MaxMapFeedback::new(&map_observer);

    let mut state = StdState::new(
        StdRand::with_seed(current_nanos()),
        InMemoryOnDiskCorpus::<HttpSequenceInput>::new(PathBuf::from(output_path)).unwrap(),
        InMemoryCorpus::new(),
        &mut map_feedback,
        &mut (),
    ).unwrap();

    let scheduler = QueueScheduler::new();
    let mut fuzzer = StdFuzzer::new(scheduler, map_feedback, ());

    let monitor = SimpleMonitor::with_user_monitor(|s| println!("{}", s));

    let mut mgr = SimpleEventManager::new(monitor);
    let mut executor = DiffExecutor::new(
        NyxExecutorBuilder::new().build(helper.0, ()),
        NyxExecutorBuilder::new().build(helper.1, ()),
        tuple_list!(map_observer),
    );

    for entry in std::fs::read_dir(solutions_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() || path.file_name().unwrap().to_str().unwrap().starts_with(".") {
            continue;
        }
        let input = HttpSequenceInput::from_file(&path).unwrap();
        let _ = fuzzer.evaluate_input(&mut state, &mut executor, &mut mgr, input);
    }
    
    info!("Loaded {} inputs", state.corpus().count());
}