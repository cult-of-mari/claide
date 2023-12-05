use {
    crate::owned_ptr::OwnedPtr,
    std::{ffi::CString, marker::PhantomData, path::Path},
    thiserror::Error,
};

pub use llama_sys as sys;

mod owned_ptr;

#[derive(Clone, Debug, Error)]
pub enum Error {
    #[error("load model")]
    LoadModel,
}

pub struct ModelOptions {
    ptr: OwnedPtr,
    verbosity_level: u8,
}

pub struct Model {
    ptr: OwnedPtr,
}

pub struct SessionOptions {
    ptr: OwnedPtr,
}

pub struct Session<'a> {
    ptr: OwnedPtr,
    model: PhantomData<&'a Model>,
}

impl ModelOptions {
    pub fn new() -> Self {
        let ptr = unsafe {
            OwnedPtr::new(
                sys::bindings_model_options_new(),
                sys::bindings_model_options_drop,
            )
        };

        Self {
            ptr,
            verbosity_level: 1,
        }
    }

    pub fn gpu_layers(&mut self, layers: u8) {
        unsafe {
            sys::bindings_model_options_set_gpu_layers(
                self.ptr.as_mut_ptr(),
                layers.try_into().unwrap(),
            )
        }
    }

    pub fn open<P: AsRef<Path>>(self, path: P) -> Result<Model, Error> {
        fn inner(options: ModelOptions, path: &Path) -> Result<Model, Error> {
            let bytes = path.as_os_str().as_encoded_bytes();
            let cstr = CString::new(bytes).map_err(|_error| Error::LoadModel)?;

            unsafe {
                let ptr = sys::bindings_model_open(cstr.as_ptr(), options.ptr.as_ptr());
                if ptr.is_null() {
                    Err(Error::LoadModel)
                } else {
                    Ok(Model {
                        ptr: OwnedPtr::new(ptr, sys::bindings_model_drop),
                    })
                }
            }
        }

        inner(self, path.as_ref())
    }
}

impl SessionOptions {
    pub fn new() -> Self {
        let ptr = unsafe {
            OwnedPtr::new(
                sys::bindings_session_options_new(),
                sys::bindings_session_options_drop,
            )
        };

        Self { ptr }
    }

    pub fn with_model(self, model: &mut Model) -> Session<'_> {
        let ptr = unsafe {
            OwnedPtr::new(
                sys::bindings_model_new_session(self.ptr.as_ptr(), self.ptr.as_ptr()),
                sys::bindings_session_drop,
            )
        };

        Session {
            ptr,
            model: PhantomData,
        }
    }
}

impl Default for ModelOptions {
    fn default() -> Self {
        Self::new()
    }
}

pub fn init(numa_aware: bool) {
    unsafe {
        sys::bindings_init(numa_aware);
    }
}

fn main() {
    init(false);

    let mut model = ModelOptions::new()
        .open("../models/teknium_openhermes-2.5-mistral-7b.gguf")
        .unwrap();

    let mut session = SessionOptions::new().with_model(&mut model);
}
