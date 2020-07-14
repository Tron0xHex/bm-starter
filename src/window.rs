use crate::{
    consts::{CLASS_NAME, WINDOW_NAME},
    enums::SecuRomRequests,
    message::Message,
};

use core::{
    intrinsics::transmute,
    mem::{size_of, zeroed},
    ptr::null_mut,
    slice::from_raw_parts,
};
use winapi::{
    shared::{
        minwindef::{DWORD, FALSE, HINSTANCE, LPARAM, LPCVOID, LPVOID, LRESULT, TRUE, UINT, WPARAM},
        windef::HWND,
    },
    um::{
        memoryapi::{ReadProcessMemory, WriteProcessMemory},
        winnt::HANDLE,
        winuser::{
            CloseWindow, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetWindowLongPtrW, InSendMessage, PeekMessageW, PostQuitMessage, RegisterClassW,
            ReplyMessage, ShowWindow, TranslateMessage, UnregisterClassW, UpdateWindow, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA, MSG, PM_REMOVE, SW_HIDE,
            WM_DESTROY, WNDCLASSW, WS_OVERLAPPED,
        },
    },
};

pub struct Window {
    h_instance: HINSTANCE,
    h_window: HWND,
}

impl Window {
    pub fn new() -> Self {
        Self {
            h_instance: null_mut(),
            h_window: null_mut(),
        }
    }

    pub unsafe fn create(&mut self, h_instance: HINSTANCE) -> bool {
        self.h_instance = h_instance;

        if !self.register_window_class() {
            panic!("unable to register the window class!");
        }

        self.h_window = CreateWindowExW(
            0,
            CLASS_NAME.as_ptr(),
            WINDOW_NAME.as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            200,
            100,
            null_mut(),
            null_mut(),
            self.h_instance,
            null_mut(),
        );

        if self.h_window.is_null() {
            panic!("unable to create the window!");
        }

        ShowWindow(self.h_window, SW_HIDE);
        UpdateWindow(self.h_window);

        true
    }

    pub fn get_handle(&self) -> HWND {
        self.h_window
    }

    pub fn get_handle_as_int(&self) -> i32 {
        self.h_window as i32
    }

    pub unsafe fn register_window_class(&self) -> bool {
        let wcex = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::dll_window_proc),
            hInstance: self.h_instance,
            lpszClassName: CLASS_NAME.as_ptr(),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
        };

        if RegisterClassW(&wcex) == 0 {
            return false;
        }

        return true;
    }

    pub unsafe fn unregister_window_class(&self) -> bool {
        if UnregisterClassW(CLASS_NAME.as_ptr(), self.h_instance) == FALSE {
            return false;
        }

        return true;
    }

    pub unsafe fn close(&self) -> bool {
        self.unregister_window_class() && CloseWindow(self.h_window) == TRUE
    }

    pub unsafe fn peek_wnd_message(&self) {
        let mut msg: MSG = zeroed();

        if PeekMessageW(&mut msg as *mut MSG, 0 as HWND, 0, 0, PM_REMOVE) == TRUE {
            TranslateMessage(&msg as *const MSG);
            DispatchMessageW(&msg as *const MSG);
        }
    }

    pub unsafe extern "system" fn dll_window_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let h_process: HANDLE = transmute(GetWindowLongPtrW(hwnd, GWLP_USERDATA));

        if msg == WM_DESTROY {
            PostQuitMessage(0);
            return 0;
        } else {
            if InSendMessage() == TRUE {
                let address = lparam as LPVOID;
                let mut buffer = [0x0; size_of::<Message>()];

                ReadProcessMemory(h_process, address, buffer.as_mut_ptr() as LPVOID, size_of::<Message>(), null_mut());

                #[repr(C, packed)]
                struct Response {
                    value: DWORD,
                }

                let srom_msg: *const Message = buffer.as_ptr() as *const Message;

                if (*srom_msg).in_message_id == SecuRomRequests::Hit as i32 {
                    ReplyMessage(((*srom_msg).payload.address + (*srom_msg).payload.value) as LRESULT);
                } else if (*srom_msg).in_message_id == SecuRomRequests::Fly as i32 {
                    let address = (*srom_msg).payload.address as LPVOID;

                    let response = Response { value: 1 };

                    let response_ptr: *const Response = &response;
                    let response_ptr: *const u8 = response_ptr as *const u8;
                    let response_buff: &[u8] = from_raw_parts(response_ptr, size_of::<Response>());

                    WriteProcessMemory(h_process, address, response_buff.as_ptr() as LPCVOID, size_of::<DWORD>(), null_mut());

                    ReplyMessage(TRUE as LRESULT);
                } else {
                    ReplyMessage(TRUE as LRESULT);
                }

                return DefWindowProcW(hwnd, msg, wparam, lparam);
            }
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
    }
}
