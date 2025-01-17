#![allow(static_mut_refs)]

mod java;


#[cfg(not(feature = "no_edge"))]
use std::{io::Write, fs::{File, OpenOptions}};
use libafl_bolts::{shmem::{
    unix_shmem::{UnixShMem, UnixShMemProvider},
    ShMemProvider, StdShMemProvider,
}, AsSliceMut};

use hdhunter::{mode::Mode, observers::HttpParam};
#[cfg(not(feature = "no_edge"))]
use libafl_targets::{EDGES_MAP_PTR, EDGES_MAP, MAX_EDGES_FOUND};

static mut SHMEM_PROVIDER: Option<UnixShMemProvider> = None;
#[cfg(not(feature = "no_edge"))]
static mut EDGES_MAP_SHMEM: Option<UnixShMem> = None;
static mut PARAM_SHMEM: Option<UnixShMem> = None;
#[no_mangle]
static mut __http_param: *mut HttpParam = std::ptr::null_mut();
static mut MODE: Mode = Mode::Request;

#[cfg(not(feature = "no_edge"))]
static mut TRACE: Option<File> = None;

#[cfg(not(feature = "no_edge"))]
#[no_mangle]
pub unsafe extern "C" fn __sanitizer_cov_trace_pc_guard(
    guard: *mut u32
) {
    if TRACE.is_some() {
        let _ = TRACE.as_ref().unwrap().write(&format!("{:010}\n", *guard).into_bytes());
    } else {
        libafl_targets::__sanitizer_cov_trace_pc_guard(guard);
    }
}

#[cfg(not(feature = "no_edge"))]
#[no_mangle]
pub unsafe extern "C" fn __sanitizer_cov_trace_pc_guard_init(
    mut start: *mut u32,
    stop: *mut u32,
) {
    if std::env::var("HDHUNTER_TRACE").is_ok() {
        if start == stop || *start != 0 {
            return;
        }

        while start < stop {
            *start = MAX_EDGES_FOUND as u32;
            start = start.offset(1);

            MAX_EDGES_FOUND = MAX_EDGES_FOUND.wrapping_add(1);
        }
    } else {
        libafl_targets::__sanitizer_cov_trace_pc_guard_init(start, stop);
    }
}

#[no_mangle]
pub unsafe extern "C" fn hdhunter_init() {
    if SHMEM_PROVIDER.is_none() {
        SHMEM_PROVIDER = Some(StdShMemProvider::new().unwrap());
    }

    #[cfg(not(feature = "no_edge"))]
    {
        // Set the global pointer to temporary memory
        EDGES_MAP_PTR = EDGES_MAP.as_mut_ptr();
        // Get and set the shared memory for the edges map
        EDGES_MAP_SHMEM = Some(
            SHMEM_PROVIDER
                .as_mut()
                .unwrap()
                .existing_from_env("__AFL_SHM")
                .unwrap(),
        );
        EDGES_MAP_PTR = EDGES_MAP_SHMEM
            .as_mut()
            .unwrap()
            .as_slice_mut()
            .as_mut_ptr();

        if let Ok(value) = std::env::var("HDHUNTER_TRACE") {
            TRACE = Some(OpenOptions::new()
                .create(true)
                .append(true)
                .open(&value)
                .unwrap()
            );
        }
    }

    // Get mode
    if let Ok(value) = std::env::var("HDHUNTER_MODE") {
        MODE = Mode::from_string(&value);
    }

    // Get and set the shared memory for the feedback param
    PARAM_SHMEM = Some(
        SHMEM_PROVIDER
            .as_mut()
            .unwrap()
            .existing_from_env("__HTTP_PARAM")
            .unwrap(),
    );
    __http_param = PARAM_SHMEM.as_mut().unwrap().as_slice_mut().as_mut_ptr() as *mut HttpParam;
}

#[cfg(feature = "java")]
pub use java::*;

#[no_mangle]
pub unsafe extern "C" fn hdhunter_set_content_length(length: i64, mode: i32) {
    // println!("hdhunter_set_content_length, length: {}, mode: {}", length, mode);
    if mode & (MODE as i32) == 0 {
        return;
    }
    if (*__http_param).message_processed == 1 {
        (*__http_param).message_count += 1;
        (*__http_param).message_processed = 0;
    }
    if (*__http_param).message_count >= (*__http_param).content_length.len() as i32 {
        return;
    }
    (*__http_param).content_length[(*__http_param).message_count as usize] = length;
}

#[no_mangle]
pub unsafe extern "C" fn hdhunter_set_chunked_encoding(chunked: i8, mode: i32) {
    // println!("hdhunter_set_chunked_encoding, chunked: {}, mode: {}", chunked, mode);
    if mode & (MODE as i32) == 0 {
        return;
    }

    if (*__http_param).message_processed == 1 {
        (*__http_param).message_count += 1;
        (*__http_param).message_processed = 0;
    }
    if (*__http_param).message_count >= (*__http_param).chunked_encoding.len() as i32 {
        return;
    }
    (*__http_param).chunked_encoding[(*__http_param).message_count as usize] = chunked;
}

#[no_mangle]
pub unsafe extern "C" fn hdhunter_inc_consumed_length(length: i64, mode: i32) {
    // println!("hdhunter_inc_consumed_length, length: {}, mode: {}", length, mode);
    if mode & (MODE as i32) == 0 {
        return;
    }

    if (*__http_param).message_processed == 1 {
        (*__http_param).message_count += 1;
        (*__http_param).message_processed = 0;
    }
    if (*__http_param).message_count >= (*__http_param).consumed_length.len() as i32 {
        return;
    }
    let consumed_length =
        &mut (*__http_param).consumed_length[(*__http_param).message_count as usize];
    if *consumed_length == -1 {
        *consumed_length = 0;
    }
    *consumed_length += length;
}

#[no_mangle]
pub unsafe extern "C" fn hdhunter_inc_body_length(length: i64, mode: i32) {
    // println!("hdhunter_inc_body_length, length: {}, mode: {}", length, mode);
    if mode & (MODE as i32) == 0 {
        return;
    }

    if (*__http_param).message_processed == 1 {
        (*__http_param).message_count += 1;
        (*__http_param).message_processed = 0;
    }
    if (*__http_param).message_count >= (*__http_param).body_length.len() as i32 {
        return;
    }
    let body_length = &mut (*__http_param).body_length[(*__http_param).message_count as usize];
    if *body_length == -1 {
        *body_length = 0;
    }
    *body_length += length;
}

#[no_mangle]
pub unsafe extern "C" fn hdhunter_mark_message_processed(mode: i32) {
    // println!("mark_message_processed, mode: {}", mode);
    if mode & (MODE as i32) == 0 {
        return;
    }

    (*__http_param).message_processed = 1;
}
