#![no_std]

use core::ffi;

extern "C" {
    pub fn bindings_clip_model_drop(model: *mut ffi::c_void);
    pub fn bindings_clip_model_open(
        path: *const ffi::c_char,
        verbosity: ffi::c_int,
    ) -> *mut ffi::c_void;
    pub fn bindings_init(numa_aware: bool);
    pub fn bindings_model_bos_token(model: *const ffi::c_void) -> i32;
    pub fn bindings_model_detokenize(
        model: *const ffi::c_void,
        token: i32,
        string_ptr: *mut ffi::c_char,
        string_len: u32,
    ) -> i32;
    pub fn bindings_model_drop(model: *mut ffi::c_void);
    pub fn bindings_model_eos_token(model: *const ffi::c_void) -> i32;
    pub fn bindings_model_eot_token(model: *const ffi::c_void) -> i32;
    pub fn bindings_model_middle_token(model: *const ffi::c_void) -> i32;
    pub fn bindings_model_nl_token(model: *const ffi::c_void) -> i32;
    pub fn bindings_model_open(
        path: *const ffi::c_char,
        options: *const ffi::c_void,
    ) -> *mut ffi::c_void;
    pub fn bindings_model_options_drop(options: *mut ffi::c_void);
    pub fn bindings_model_options_gpu_layers(options: *const ffi::c_void) -> u16;
    pub fn bindings_model_options_new() -> *mut ffi::c_void;
    pub fn bindings_model_options_set_gpu_layers(options: *mut ffi::c_void, value: u16);
    pub fn bindings_model_options_set_use_mlock(options: *mut ffi::c_void, value: bool);
    pub fn bindings_model_options_set_use_mmap(options: *mut ffi::c_void, value: bool);
    pub fn bindings_model_options_use_mlock(options: *const ffi::c_void) -> bool;
    pub fn bindings_model_options_use_mmap(options: *const ffi::c_void) -> bool;
    pub fn bindings_model_prefix_token(model: *const ffi::c_void) -> i32;
    pub fn bindings_model_requires_bos_token(model: *const ffi::c_void) -> ffi::c_int;
    pub fn bindings_model_requires_eos_token(model: *const ffi::c_void) -> ffi::c_int;
    pub fn bindings_model_suffix_token(model: *const ffi::c_void) -> i32;
    pub fn bindings_model_tokenize(
        model: *const ffi::c_void,
        string_ptr: *const ffi::c_char,
        string_len: u32,
        tokens: *const i32,
        tokens_capacity: u32,
        add_bos: bool,
        special: bool,
    ) -> i32;
    pub fn bindings_session_batch_drop(batch: *mut ffi::c_void);
    pub fn bindings_session_batch_init(
        token_capacity: u16,
        embedding_size: u16,
        max_sequence_ids: u16,
    ) -> *mut ffi::c_void;
    pub fn bindings_session_decode(
        session: *mut ffi::c_void,
        batch: *mut ffi::c_void,
    ) -> ffi::c_int;
    pub fn bindings_session_drop(session: *mut ffi::c_void);
    pub fn bindings_session_new(
        model: *mut ffi::c_void,
        options: *const ffi::c_void,
    ) -> *mut ffi::c_void;
    pub fn bindings_session_options_drop(options: *mut ffi::c_void);
    pub fn bindings_session_options_new() -> *mut ffi::c_void;
    pub fn bindings_session_sampling_drop(sampling: *mut ffi::c_void);
    pub fn bindings_session_sampling_new(options: *mut ffi::c_void) -> *mut ffi::c_void;
    pub fn bindings_session_sampling_options_drop(options: *mut ffi::c_void);
    pub fn bindings_session_sampling_options_new() -> *mut ffi::c_void;
    pub fn bindings_session_sampling_options_set_temperature(options: *mut ffi::c_void, value: f32);
    pub fn bindings_session_sampling_options_set_top_k(options: *mut ffi::c_void, value: f32);
    pub fn bindings_session_sampling_options_set_top_p(options: *mut ffi::c_void, value: f32);
    pub fn bindings_session_sampling_options_temperature(options: *const ffi::c_void) -> f32;
    pub fn bindings_session_sampling_options_top_k(options: *const ffi::c_void) -> f32;
    pub fn bindings_session_sampling_options_top_p(options: *const ffi::c_void) -> f32;
    pub fn bindings_session_sampling_reset(sampling: *mut ffi::c_void);
}
