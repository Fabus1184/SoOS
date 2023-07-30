use alloc::{format, string::String};
use log::{info, trace};

use crate::SCHEDULER;

#[derive(Debug, Clone, Copy)]
pub enum Syscall {
    Print(*const i8),
    Sleep(i64),
}

impl Syscall {
    pub unsafe fn from_stack_ptr(ptr: *const u64) -> Result<Self, String> {
        let n: u64 = *ptr;
        let arg1: u64 = *(ptr.offset(1));

        match n {
            0 => Ok(Self::Print(arg1 as *const i8)),
            1 => Ok(Self::Sleep(arg1 as i64)),
            _ => Err(format!("unknown syscall: n {:?}, arg1: {:?}", n, arg1)),
        }
    }

    pub fn execute(self) {
        match self {
            Self::Print(ptr) => {
                let cstr = unsafe { core::ffi::CStr::from_ptr(ptr) };
                info!("syscall print: ({:?}) {:?}", ptr, cstr);
            }
            Self::Sleep(ms) => {
                info!("syscall sleep: {:?}", ms);
                unsafe {
                    SCHEDULER
                        .try_lock()
                        .map(|mut s| (&mut **s).sleep(ms))
                        .unwrap_or_else(|| trace!("failed to lock scheduler"));
                };
            }
        }
    }
}
