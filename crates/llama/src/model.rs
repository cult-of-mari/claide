use {
    crate::{owned_ptr::OwnedPtr, sys, Error},
    std::{
        any,
        ffi::{self, CString},
        fmt,
        path::Path,
    },
};

pub struct ModelOptions {
    ptr: OwnedPtr,
    verbosity: u8,
}

pub struct Model {
    ptr: OwnedPtr,
}

impl Model {
    pub fn as_ptr(&self) -> *const ffi::c_void {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut ffi::c_void {
        self.ptr.as_mut_ptr()
    }
}

impl ModelOptions {
    pub fn new() -> Self {
        let ptr = unsafe {
            OwnedPtr::new(
                sys::bindings_model_options_new(),
                sys::bindings_model_options_drop,
            )
        };

        Self { ptr, verbosity: 1 }
    }

    pub fn as_ptr(&self) -> *const ffi::c_void {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut ffi::c_void {
        self.ptr.as_mut_ptr()
    }

    pub fn gpu_layers(&mut self, layers: u8) -> &mut Self {
        unsafe {
            sys::bindings_model_options_set_gpu_layers(
                self.ptr.as_mut_ptr(),
                layers.try_into().unwrap(),
            )
        }

        self
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<Model, Error> {
        fn inner(options: &ModelOptions, path: &Path) -> Result<Model, Error> {
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

impl Default for ModelOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ModelOptions {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct(any::type_name::<Self>())
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for Model {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct(any::type_name::<Self>())
            .finish_non_exhaustive()
    }
}
