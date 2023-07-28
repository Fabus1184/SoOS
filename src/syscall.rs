use alloc::{format, string::String};

use crate::printk;

#[derive(Debug, Clone, Copy)]
pub enum Syscall {
    Print(*const i8),
}

impl Syscall {
    pub unsafe fn from_stack_ptr(ptr: *const u64) -> Result<Self, String> {
        let n: u64 = *ptr;
        let arg1 = *(ptr.offset(1));

        match n {
            0 => Ok(Self::Print(arg1 as *const i8)),
            _ => Err(format!("unknown syscall: n {:?}, arg1: {:?}", n, arg1)),
        }
    }

    pub fn execute(self) {
        match self {
            Self::Print(ptr) => {
                let cstr = unsafe { core::ffi::CStr::from_ptr(ptr) };
                printk!("syscall print: ({:?}) {:?}\n", ptr, cstr);
            }
        }
    }
}
