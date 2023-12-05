use {
    crate::{owned_ptr::OwnedPtr, sys, Model},
    std::{any, fmt, marker::PhantomData},
};

/// An inference session.
pub struct Session<'a> {
    pub(crate) session_ptr: OwnedPtr,
    pub(crate) sampling_ptr: OwnedPtr,
    _model_phantom: PhantomData<&'a Model>,
}

/// Options and flags which can be used to configure how a session is created.
pub struct SessionOptions {
    pub(crate) options_ptr: OwnedPtr,
    pub(crate) sampling_options_ptr: OwnedPtr,
}

impl<'a> Session<'a> {}

impl SessionOptions {
    /// Creates a new set of session options ready for configuration.
    pub fn new() -> Self {
        unsafe {
            Self {
                options_ptr: OwnedPtr::new(
                    sys::bindings_session_options_new(),
                    sys::bindings_session_options_drop,
                ),
                sampling_options_ptr: OwnedPtr::new(
                    sys::bindings_session_sampling_options_new(),
                    sys::bindings_session_sampling_options_drop,
                ),
            }
        }
    }

    /// Creates a session with the specified model.
    pub fn with_model(mut self, model: &mut Model) -> Session<'_> {
        unsafe {
            Session {
                session_ptr: OwnedPtr::new(
                    sys::bindings_session_new(
                        model.model_ptr.as_mut_ptr(),
                        self.options_ptr.as_ptr(),
                    ),
                    sys::bindings_session_drop,
                ),
                sampling_ptr: OwnedPtr::new(
                    sys::bindings_session_sampling_new(self.sampling_options_ptr.as_mut_ptr()),
                    sys::bindings_session_sampling_drop,
                ),
                _model_phantom: PhantomData,
            }
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
