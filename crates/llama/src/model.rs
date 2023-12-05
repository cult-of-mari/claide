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

    pub fn gpu_layers(&self) -> u16 {
        unsafe { sys::bindings_model_options_gpu_layers(self.as_ptr()) }
    }

    pub fn set_gpu_layers(&mut self, layers: u16) -> &mut Self {
        unsafe { sys::bindings_model_options_set_gpu_layers(self.as_mut_ptr(), layers) }

        self
    }

    pub fn use_mlock(&self) -> bool {
        unsafe { sys::bindings_model_options_use_mlock(self.as_ptr()) }
    }

    pub fn set_use_mlock(&mut self, mlock: bool) -> &mut Self {
        unsafe { sys::bindings_model_options_set_use_mlock(self.as_mut_ptr(), mlock) }

        self
    }

    pub fn use_mmap(&self) -> bool {
        unsafe { sys::bindings_model_options_use_mmap(self.as_ptr()) }
    }

    pub fn set_use_mmap(&mut self, mmap: bool) -> &mut Self {
        unsafe { sys::bindings_model_options_set_use_mlock(self.as_mut_ptr(), mmap) }

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
            .field("gpu_layers", &self.gpu_layers())
            .field("use_mlock", &self.use_mlock())
            .field("use_mmap", &self.use_mmap())
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for Model {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct(any::type_name::<Self>())
            .finish_non_exhaustive()
    }
}
