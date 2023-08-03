use alloc::{format, string::String};
use log::info;

use crate::SCHEDULER;

#[derive(Debug, Clone, Copy)]
pub enum Syscall {
    Print(*const i8),
    Sleep(u64),
    Getpid(*mut u64),
}

impl Syscall {
    pub unsafe fn from_regs(rax: u64, rbx: u64) -> Result<(), String> {
        match rax {
            0 => Ok(Self::Print(rbx as *const i8)),
            1 => Ok(Self::Sleep(rbx)),
            2 => Ok(Self::Getpid(rbx as *mut u64)),
            _ => Err(format!("unknown syscall: rax {:?}, rbx {:?}", rax, rbx)),
        }
        .map(|s| {
            info!("syscall {:?}", s);
            s.execute();
        })
    }

    pub fn execute(self) {
        match self {
            Self::Print(ptr) => {
                let cstr = unsafe { core::ffi::CStr::from_ptr(ptr) };
                let str = cstr.to_str().expect("failed to convert cstr to str");
                info!("{}", str);
            }
            Self::Sleep(ms) => {
                unsafe {
                    SCHEDULER.sleep(ms);
                };
            }
            Self::Getpid(ptr) => unsafe {
                *ptr = (*SCHEDULER.current_process.expect("no current process"))
                    .pid
                    .as_u32() as u64;
            },
        }
    }
}
