use {
    crate::owned_ptr::OwnedPtr,
    std::{ffi::CString, path::Path},
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

pub fn init(numa_aware: bool) {
    unsafe {
        sys::bindings_init(numa_aware);
    }
}

fn main() {}
