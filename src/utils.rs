use anyhow::{anyhow, Result};
use std::{io::Error, mem, slice::from_raw_parts};

use winapi::{
    shared::{
        minwindef::{FALSE, LPVOID, TRUE},
        ntdef::HANDLE,
    },
    um::{
        handleapi::CloseHandle,
        memoryapi::{ReadProcessMemory, WriteProcessMemory},
        processthreadsapi::OpenProcess,
        winnt::{
            PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
        },
    },
};

struct SafeHandle(pub HANDLE);

impl Drop for SafeHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

#[inline]
fn get_process_handle(pid: u32) -> Result<SafeHandle> {
    const NULL_HANDLE: HANDLE = 0 as HANDLE;

    unsafe {
        match OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE | PROCESS_VM_OPERATION,
            FALSE,
            pid,
        ) {
            NULL_HANDLE => Err(anyhow!("{}", Error::last_os_error())),
            hanlde => Ok(SafeHandle(hanlde)),
        }
    }
}

#[inline]
pub fn struct_to_bytes<'a, T>(s: &T) -> &'a [u8]
where
    T: Sized,
{
    let data = s as *const _ as *const u8;
    let len = mem::size_of::<T>();

    unsafe { from_raw_parts(data, len) }
}

#[inline]
pub fn bytes_to_struct<'a, T>(buf: &[u8]) -> &T
where
    T: Sized,
{
    let s: *const T = buf.as_ptr() as *const T;
    let s = unsafe { &*s };

    s
}

#[inline]
pub fn read_mem(pid: u32, address: usize, buffer: &mut [u8], len: usize) -> Result<usize> {
    let mut bytes_read = 0;
    let hanlde = get_process_handle(pid)?;

    unsafe {
        match ReadProcessMemory(
            hanlde.0,
            address as LPVOID,
            buffer.as_mut_ptr() as LPVOID,
            len,
            &mut bytes_read,
        ) {
            TRUE => Ok(bytes_read),
            FALSE => Err(anyhow!("{}", Error::last_os_error())),
            _ => unreachable!(),
        }
    }
}

#[inline]
pub fn write_mem(pid: u32, address: usize, buffer: &[u8], len: usize) -> Result<usize> {
    let mut bytes_written = 0;
    let hanlde = get_process_handle(pid)?;

    unsafe {
        match WriteProcessMemory(
            hanlde.0,
            address as LPVOID,
            buffer.as_ptr() as LPVOID,
            len,
            &mut bytes_written,
        ) {
            TRUE => Ok(bytes_written),
            FALSE => Err(anyhow!("{}", Error::last_os_error())),
            _ => unreachable!(),
        }
    }
}
