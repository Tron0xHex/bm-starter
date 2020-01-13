use crate::consts::{BUFFER_SIZE, EXE_NAME, IN_MESSAGE_ID};
use crate::dll_window::{DllWindow, DllWindowInner};
use crate::event_loop::EventLoop;
use crate::mapped_file::MappedFile;
use crate::message::Message;
use crate::request_type::RequestType;
use crate::win32::win32_string;

use winapi::shared::minwindef::{
    DWORD, FALSE, HINSTANCE, LPARAM, LPCVOID, LPVOID, LRESULT, TRUE, WPARAM,
};
use winapi::um::fileapi::{GetFileAttributesW, INVALID_FILE_ATTRIBUTES};
use winapi::um::minwinbase::STILL_ACTIVE;
use winapi::um::processthreadsapi::{
    CreateProcessW, GetExitCodeProcess, PROCESS_INFORMATION, STARTUPINFOW,
};
use winapi::um::winnt::HANDLE;

use winapi::shared::windef::HWND;
use winapi::um::winuser::{InSendMessage, ReplyMessage};

use winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};

use spin::RwLock;
use std::cell::RefCell;
use std::env::current_dir;
use std::mem::{size_of, zeroed};
use std::process::exit;
use std::ptr::{copy_nonoverlapping, null, null_mut};
use std::slice::from_raw_parts;
use std::sync::Arc;
use std::time::Duration;

pub struct LoaderEmulator {
    inner: Arc<RwLock<LoaderEmulatorInner>>,
}

impl LoaderEmulator {
    pub fn new() -> LoaderEmulator {
        LoaderEmulator {
            inner: Arc::new(RwLock::new(LoaderEmulatorInner::new())),
        }
    }

    pub unsafe fn start(&mut self, h_module: HINSTANCE) -> bool {
        let loader_emulator = &mut self.inner.write();

        loader_emulator.exe_path = current_dir().unwrap().to_str().unwrap().to_string();

        let process_handle_arc = loader_emulator.process_handle.clone();

        loader_emulator
            .dll_window
            .inner
            .write()
            .register_on_message_callback(
                IN_MESSAGE_ID,
                Box::new(move |hwnd: HWND, wparam: WPARAM, lparam: LPARAM| {
                    let process_handle_arc_clone = process_handle_arc.clone();

                    if InSendMessage() == TRUE {
                        LoaderEmulatorInner::on_income_message_callback_proxy(
                            *process_handle_arc_clone.borrow(),
                            hwnd,
                            wparam,
                            lparam,
                        );
                    }
                }),
            );

        if !loader_emulator.dll_window.create(h_module) {
            panic!("Unable to create the loader window!");
        }

        let mut message: Message = zeroed();

        if !loader_emulator.mapped_file.create(BUFFER_SIZE as usize) {
            panic!("Unable to create the file.");
        }

        message.in_message_id = IN_MESSAGE_ID as i32;
        message.window_handle = loader_emulator.dll_window.inner.read().get_handle_as_int();

        let message_ptr: *const Message = &message;
        let message_ptr: *const u8 = message_ptr as *const u8;
        let message_buff: &[u8] = from_raw_parts(message_ptr, size_of::<Message>());

        copy_nonoverlapping(
            message_buff.as_ptr(),
            loader_emulator.mapped_file.get_file_ptr() as *mut u8,
            size_of::<Message>(),
        );

        let process_handle_arc = loader_emulator.process_handle.clone();

        loader_emulator.event_loop.add_callback(
            "WatchDog",
            Box::new(move || {
                let mut exit_code = 0;
                let process_handle_arc_clone = process_handle_arc.clone();

                if GetExitCodeProcess(*process_handle_arc_clone.borrow(), &mut exit_code) == TRUE {
                    if exit_code != STILL_ACTIVE {
                        return Err(());
                    }
                }

                Ok(())
            }),
        );

        loader_emulator.event_loop.add_callback(
            "PeekWndMessage",
            Box::new(move || {
                DllWindowInner::peek_wnd_message();
                Ok(())
            }),
        );

        if let Ok(handle) = loader_emulator.spawn_process() {
            loader_emulator.process_handle.replace(handle);
        } else {
            panic!("Unable to start the process.")
        }

        loader_emulator.event_loop.start();
        loader_emulator.shutdown();

        exit(0);
    }
}

pub struct LoaderEmulatorInner {
    exe_path: String,
    process_handle: Arc<RefCell<HANDLE>>,
    dll_window: DllWindow,
    event_loop: EventLoop,
    mapped_file: MappedFile,
}

impl LoaderEmulatorInner {
    pub fn new() -> LoaderEmulatorInner {
        LoaderEmulatorInner {
            exe_path: "".to_string(),
            process_handle: Arc::new(RefCell::new(null_mut())),
            dll_window: DllWindow::new(),
            event_loop: EventLoop::new(Duration::from_millis(25)),
            mapped_file: MappedFile::new(),
        }
    }

    pub unsafe fn shutdown(&mut self) {
        self.event_loop.stop();
        self.event_loop.remove_all_callbacks();
        self.dll_window
            .inner
            .write()
            .un_register_on_message_callback(IN_MESSAGE_ID);
        self.dll_window.inner.write().close();
        self.mapped_file.close();
    }

    pub unsafe fn spawn_process(&mut self) -> Result<HANDLE, ()> {
        let mut si: STARTUPINFOW = zeroed();
        si.cb = size_of::<STARTUPINFOW>() as u32;
        let mut pi: PROCESS_INFORMATION = zeroed();

        let file_name: String = {
            if cfg!(debug_assertions) {
                format!("D:\\Gamers\\Batman Arkham Asylum\\Binaries\\{}", &EXE_NAME)
            } else {
                format!("{}\\{}", &self.exe_path, &EXE_NAME)
            }
        };

        let file_name_w32 = win32_string(&file_name);

        if GetFileAttributesW(file_name_w32.as_ptr()) == INVALID_FILE_ATTRIBUTES {
            return Err(());
        }

        if CreateProcessW(
            file_name_w32.as_ptr(),
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

    pub unsafe fn on_income_message_callback_proxy(
        process_handle: HANDLE,
        _: HWND,
        _: WPARAM,
        lparam: LPARAM,
    ) {
        let address = lparam as LPVOID;
        let mut buffer = [0x0; size_of::<Message>()];

        ReadProcessMemory(
            process_handle,
            address,
            buffer.as_mut_ptr() as LPVOID,
            size_of::<Message>(),
            null_mut(),
        );

        #[repr(C, packed)]
        struct Response {
            value: DWORD,
        }

        let msg: *const Message = buffer.as_ptr() as *const Message;

        if (*msg).in_message_id == RequestType::HitRequest as i32 {
            ReplyMessage(((*msg).payload.address + (*msg).payload.value) as LRESULT);
        } else if (*msg).in_message_id == RequestType::FlyRequest as i32 {
            let address = (*msg).payload.address as LPVOID;

            let response = Response { value: 1 };

            let response_ptr: *const Response = &response;
            let response_ptr: *const u8 = response_ptr as *const u8;
            let response_buff: &[u8] = from_raw_parts(response_ptr, size_of::<Response>());

            WriteProcessMemory(
                process_handle,
                address,
                response_buff.as_ptr() as LPCVOID,
                size_of::<DWORD>(),
                null_mut(),
            );

            ReplyMessage(TRUE as LRESULT);
        } else {
            ReplyMessage(TRUE as LRESULT);
        }
    }
}
