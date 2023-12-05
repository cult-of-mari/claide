use {
    crate::{owned_ptr::OwnedPtr, sys, Model},
    std::{any, ffi, fmt, marker::PhantomData},
};

/// An inference session.
pub struct Session<'a> {
    ptr: OwnedPtr,
    model: PhantomData<&'a Model>,
}

/// Options and flags which can be used to configure how a session is created.
pub struct SessionOptions {
    ptr: OwnedPtr,
}

impl<'a> Session<'a> {
    pub fn as_ptr(&self) -> *const ffi::c_void {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut ffi::c_void {
        self.ptr.as_mut_ptr()
    }
}

impl SessionOptions {
    /// Creates a new set of session options ready for configuration.
    pub fn new() -> Self {
        let ptr = unsafe {
            OwnedPtr::new(
                sys::bindings_session_options_new(),
                sys::bindings_session_options_drop,
            )
        };

        Self { ptr }
    }

    pub fn as_ptr(&self) -> *const ffi::c_void {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut ffi::c_void {
        self.ptr.as_mut_ptr()
    }

    /// Creates a session with the specified model.
    pub fn with_model(self, model: &mut Model) -> Session<'_> {
        let ptr = unsafe {
            OwnedPtr::new(
                sys::bindings_model_new_session(model.as_mut_ptr(), self.as_ptr()),
                sys::bindings_session_drop,
            )
        };

        Session {
            ptr,
            model: PhantomData,
        }
    }
}

impl Default for SessionOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for SessionOptions {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct(any::type_name::<Self>())
            .finish_non_exhaustive()
    }
}

impl<'a> fmt::Debug for Session<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct(any::type_name::<Self>())
            .finish_non_exhaustive()
    }
}
