use crate::consts::{BUFFER_SIZE, IN_MESSAGE_ID};
use crate::{filemapping::FileMapping, message::Message, window::Window};

use core::{
    intrinsics::{copy_nonoverlapping, transmute},
    mem::{size_of, zeroed},
    ptr::{null, null_mut},
    slice::from_raw_parts,
};
use winapi::{
    shared::minwindef::{FALSE, HINSTANCE, TRUE},
    um::{
        fileapi::{GetFileAttributesW, INVALID_FILE_ATTRIBUTES},
        minwinbase::STILL_ACTIVE,
        processthreadsapi::{
            CreateProcessW, ExitProcess, GetExitCodeProcess, PROCESS_INFORMATION, STARTUPINFOW,
        },
        synchapi::Sleep,
        winnt::HANDLE,
        winuser::{SetWindowLongPtrW, GWLP_USERDATA},
    },
};

use wchar::wch_c;

pub struct Emulator {
    dll_window: Window,
    file_mapping: FileMapping,
    h_process: HANDLE,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            dll_window: Window::new(),
            file_mapping: FileMapping::new(),
            h_process: null_mut(),
        }
    }

    pub unsafe fn start(&mut self, h_module: HINSTANCE) {
        if !self.dll_window.create(h_module) {
            panic!("unable to create the loader window!");
        }

        let mut message: Message = zeroed();

        if !self.file_mapping.create(BUFFER_SIZE as usize) {
            panic!("unable to create the file.");
        }

        message.in_message_id = IN_MESSAGE_ID as i32;
        message.window_handle = self.dll_window.get_handle_as_int();

        let message_ptr: *const Message = &message;
        let message_ptr: *const u8 = message_ptr as *const u8;
        let message_buff: &[u8] = from_raw_parts(message_ptr, size_of::<Message>());

        copy_nonoverlapping(
            message_buff.as_ptr(),
            self.file_mapping.get_file_ptr() as *mut u8,
            size_of::<Message>(),
        );

        if let Ok(handle) = self.spawn_process() {
            self.h_process = handle;
        } else {
            panic!("unable to start the process.")
        }

        SetWindowLongPtrW(
            self.dll_window.get_handle(),
            GWLP_USERDATA,
            transmute(self.h_process),
        );

        loop {
            if self.is_process_running() {
                self.peek_next_window_msg();
                Sleep(10);
            } else {
                break;
            }
        }

        self.shutdown();
        self.exit();
    }

    pub unsafe fn is_process_running(&self) -> bool {
        let mut exit_code = 0;

        if GetExitCodeProcess(self.h_process, &mut exit_code) == TRUE {
            if exit_code != STILL_ACTIVE {
                return false;
            }
        }

        true
    }

    pub unsafe fn peek_next_window_msg(&mut self) {
        self.dll_window.peek_wnd_message();
    }

    pub unsafe fn shutdown(&mut self) {
        self.dll_window.close();
        self.file_mapping.close();
    }

    pub unsafe fn exit(&mut self) {
        ExitProcess(0);
    }

    pub unsafe fn spawn_process(&mut self) -> Result<HANDLE, ()> {
        let mut si: STARTUPINFOW = zeroed();
        {
            si.cb = size_of::<STARTUPINFOW>() as u32;
        }

        let mut pi: PROCESS_INFORMATION = zeroed();

        let file_name: &[u16] = {
            if cfg!(debug_assertions) {
                wch_c!("X:\\Games\\Batman Arkham Asylum\\Binaries\\ShippingPC-BmGame.exe")
            } else {
                wch_c!("ShippingPC-BmGame.exe")
            }
        };

        if GetFileAttributesW(file_name.as_ptr()) == INVALID_FILE_ATTRIBUTES {
            return Err(());
        }

        if CreateProcessW(
            file_name.as_ptr(),
            null_mut(),
            null_mut(),
            null_mut(),
            FALSE,
            0,
            null_mut(),
            null(),
            &mut si,
            &mut pi,
        ) == TRUE
        {
            return Ok(pi.hProcess);
        }

        Err(())
    }
}
