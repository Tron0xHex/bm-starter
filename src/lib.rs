#![no_std]
#![allow(non_snake_case)]

mod consts;
mod emulator;
mod enums;
mod filemapping;
mod message;
mod window;

use core::panic::PanicInfo;

use emulator::Emulator;
use winapi::shared::{
    minwindef,
    minwindef::{BOOL, DWORD, HINSTANCE, LPVOID},
};

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub unsafe extern "system" fn _DllMainCRTStartup(
    dll_module: HINSTANCE,
    call_reason: DWORD,
    reserved: LPVOID,
) -> BOOL {
    const DLL_PROCESS_ATTACH: DWORD = 1;
    const DLL_PROCESS_DETACH: DWORD = 0;

    match call_reason {
        DLL_PROCESS_ATTACH => {
            Emulator::new().start(dll_module);
        }
        DLL_PROCESS_DETACH => (),
        _ => (),
    }
    minwindef::TRUE
}

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
