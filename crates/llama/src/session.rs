use {
    crate::{owned_ptr::OwnedPtr, sys, Model},
    std::{any, fmt, slice},
};

/// An inference session.
pub struct Session {
    pub(crate) session_ptr: OwnedPtr,
    pub(crate) sampler_ptr: OwnedPtr,
    model: Model,
}

/// Options and flags which can be used to configure how a session is created.
pub struct SessionOptions {
    pub(crate) options_ptr: OwnedPtr,
    pub(crate) sampler_options_ptr: OwnedPtr,
}

pub(crate) struct SessionBatch {
    pub(crate) batch_ptr: OwnedPtr,
    pub(crate) capacity: u32,
}

impl SessionBatch {
    pub fn new(token_capacity: u32, embedding_size: u32, max_sequence_ids: u32) -> Self {
        unsafe {
            Self {
                batch_ptr: OwnedPtr::new(
                    sys::bindings_session_batch_init(
                        token_capacity,
                        embedding_size,
                        max_sequence_ids,
                    ),
                    sys::bindings_session_batch_drop,
                ),
                capacity: token_capacity,
            }
        }
    }

    pub fn add_token(&mut self, token: i32, index: u32) {
        unsafe {
            sys::bindings_session_batch_add_token(self.batch_ptr.as_mut_ptr(), token, index);
        }
    }
}

impl Session {
    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn into_model(self) -> Model {
        self.model
    }

    pub fn infer(&mut self, tokens: &[i32]) {
        let mut batch = SessionBatch::new(4096, 0, 4096);

        for (index, token) in tokens.iter().copied().enumerate() {
            batch.add_token(token, index.try_into().unwrap());
        }

        unsafe {
            let tokens = sys::bindings_session_decode(
                self.session_ptr.as_mut_ptr(),
                batch.batch_ptr.as_mut_ptr(),
            );

            println!("{tokens:?}");
        }
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
                sampler_options_ptr: OwnedPtr::new(
                    sys::bindings_session_sampler_options_new(),
                    sys::bindings_session_sampler_options_drop,
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
                sampler_ptr: OwnedPtr::new(
                    sys::bindings_session_sampler_new(self.sampler_options_ptr.as_mut_ptr()),
                    sys::bindings_session_sampler_drop,
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
