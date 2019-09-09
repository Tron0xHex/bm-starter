#![cfg(windows)]
extern crate winapi;

extern crate spin;

extern crate libc;

mod consts;
mod dll_window;
mod event_loop;
mod loader_emulator;
mod mapped_file;
mod message;
mod request_type;
mod win32;

use winapi::shared::{
    minwindef,
    minwindef::{BOOL, DWORD, HINSTANCE, LPVOID},
};

use crate::loader_emulator::LoaderEmulatorInner;

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub unsafe extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: DWORD,
    reserved: LPVOID,
) -> BOOL {
    const DLL_PROCESS_ATTACH: DWORD = 1;
    const DLL_PROCESS_DETACH: DWORD = 0;

    match call_reason {
        DLL_PROCESS_ATTACH => {
            LoaderEmulatorInner::new().start(dll_module);
        }
        DLL_PROCESS_DETACH => (),
        _ => (),
    }
    minwindef::TRUE
}
