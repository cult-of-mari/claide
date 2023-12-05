#![no_std]

use core::ffi;

extern "C" {
    pub fn bindings_init(numa_aware: bool);

    pub fn bindings_clip_model_open(
        path: *const ffi::c_char,
        verbosity: ffi::c_int,
    ) -> *mut ffi::c_void;
    pub fn bindings_clip_model_drop(model: *mut ffi::c_void);

    pub fn bindings_model_open(
        path: *const ffi::c_char,
        options: *const ffi::c_void,
    ) -> *mut ffi::c_void;
    pub fn bindings_model_drop(model: *mut ffi::c_void);

    pub fn bindings_model_options_new() -> *mut ffi::c_void;
    pub fn bindings_model_options_gpu_layers(options: *const ffi::c_void) -> u16;
    pub fn bindings_model_options_set_gpu_layers(options: *mut ffi::c_void, value: u16);
    pub fn bindings_model_options_use_mlock(options: *const ffi::c_void) -> bool;
    pub fn bindings_model_options_set_use_mlock(options: *mut ffi::c_void, value: bool);
    pub fn bindings_model_options_use_mmap(options: *const ffi::c_void) -> bool;
    pub fn bindings_model_options_set_use_mmap(options: *mut ffi::c_void, value: bool);
    pub fn bindings_model_options_drop(options: *mut ffi::c_void);

    pub fn bindings_session_options_new() -> *mut ffi::c_void;
    pub fn bindings_session_options_drop(options: *mut ffi::c_void);

    pub fn bindings_session_new(
        model: *mut ffi::c_void,
        options: *const ffi::c_void,
    ) -> *mut ffi::c_void;
    pub fn bindings_session_drop(options: *mut ffi::c_void);

    pub fn bindings_session_sampling_options_new() -> *mut ffi::c_void;
    pub fn bindings_session_sampling_options_temperature(options: *const ffi::c_void) -> f32;
    pub fn bindings_session_sampling_options_set_temperature(options: *mut ffi::c_void, value: f32);
    pub fn bindings_session_sampling_options_top_k(options: *const ffi::c_void) -> f32;
    pub fn bindings_session_sampling_options_set_top_k(options: *mut ffi::c_void, value: f32);
    pub fn bindings_session_sampling_options_top_p(options: *const ffi::c_void) -> f32;
    pub fn bindings_session_sampling_options_set_top_p(options: *mut ffi::c_void, value: f32);
    pub fn bindings_session_sampling_options_drop(options: *mut ffi::c_void);

    pub fn bindings_session_sampling_new(options: *mut ffi::c_void) -> *mut ffi::c_void;
    pub fn bindings_session_sampling_reset(sampling: *mut ffi::c_void);
    pub fn bindings_session_sampling_drop(sampling: *mut ffi::c_void);
}
