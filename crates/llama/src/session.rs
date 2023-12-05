use {
    crate::{owned_ptr::OwnedPtr, sys, Model},
    std::{any, fmt},
};

/// An inference session.
pub struct Session {
    pub(crate) session_ptr: OwnedPtr,
    pub(crate) sampling_ptr: OwnedPtr,
    model: Model,
}

/// Options and flags which can be used to configure how a session is created.
pub struct SessionOptions {
    pub(crate) options_ptr: OwnedPtr,
    pub(crate) sampling_options_ptr: OwnedPtr,
}

impl Session {
    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn into_model(self) -> Model {
        self.model
    }
}

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
    pub fn with_model(mut self, mut model: Model) -> Session {
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
                model,
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

impl fmt::Debug for Session {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct(any::type_name::<Self>())
            .finish_non_exhaustive()
    }
}
