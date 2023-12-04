#![no_std]

use core::ffi;

extern "C" {
    pub fn bindings_clip_model_open(
        path: *const ffi::c_char,
        verbosity: ffi::c_int,
    ) -> *mut ffi::c_void;

    pub fn bindings_init(numa_aware: bool);

    pub fn bindings_model_open(
        path: *const ffi::c_char,
        options: *const ffi::c_void,
    ) -> *mut ffi::c_void;

    pub fn bindings_model_new_session(
        model: *const ffi::c_void,
        options: *const ffi::c_void,
    ) -> *mut ffi::c_void;
}
