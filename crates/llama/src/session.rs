use {
    crate::{owned_ptr::OwnedPtr, sys, Model},
    std::{any, fmt, slice},
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

pub(crate) struct SessionBatch {
    pub(crate) batch_ptr: OwnedPtr,
    pub(crate) capacity: u16,
}

impl SessionBatch {
    pub fn new(token_capacity: u16, embedding_size: u16, max_sequence_ids: u16) -> Self {
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

    pub fn tokens_len(&self) -> u32 {
        unsafe { sys::bindings_session_batch_tokens_len(self.batch_ptr.as_ptr()) }
    }

    pub unsafe fn set_tokens_len(&mut self, new_len: u32) {
        sys::bindings_session_batch_tokens_set_len(self.batch_ptr.as_mut_ptr(), new_len)
    }

    pub fn tokens(&self) -> &[i32] {
        unsafe {
            slice::from_raw_parts(
                dbg!(sys::bindings_session_batch_tokens_ptr(
                    self.batch_ptr.as_ptr()
                )),
                self.capacity.try_into().unwrap(),
            )
        }
    }

    pub fn tokens_mut(&mut self) -> &mut [i32] {
        unsafe {
            slice::from_raw_parts_mut(
                dbg!(sys::bindings_session_batch_tokens_mut_ptr(
                    self.batch_ptr.as_mut_ptr()
                )),
                self.capacity.try_into().unwrap(),
            )
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

    pub fn infer(&mut self, _tokens: &[i32]) {
        let mut batch = SessionBatch::new(4098, 4096, 4096);

        //println!("{:?}", batch.tokens_len());
        println!("{:?}", batch.tokens());
        println!("{:?}", batch.tokens().len());
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
