use crate::consts::{CLASS_NAME, WINDOW_NAME};
use crate::win32::win32_string;

use winapi::shared::{
    minwindef::{FALSE, HINSTANCE, LPARAM, LRESULT, TRUE, UINT, WPARAM},
    windef::HWND,
};

use winapi::um::winuser::{
    CloseWindow, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetWindowLongPtrW,
    PeekMessageW, PostQuitMessage, RegisterClassW, SetWindowLongPtrW, ShowWindow, TranslateMessage,
    UnregisterClassW, UpdateWindow, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA, MSG, PM_REMOVE, SW_HIDE,
    WM_DESTROY, WNDCLASSW, WS_OVERLAPPED,
};

use spin::RwLock;
use std::collections::HashMap;
use std::mem::{transmute, zeroed};
use std::ptr::null_mut;
use std::sync::Arc;

pub type OnMessageCallback = Box<dyn Fn(HWND, WPARAM, LPARAM)>;

pub struct DllWindow {
    pub inner: Arc<RwLock<DllWindowInner>>,
}

impl DllWindow {
    pub fn new() -> DllWindow {
        DllWindow {
            inner: Arc::new(RwLock::new(DllWindowInner::new())),
        }
    }

    pub unsafe fn create(&mut self, h_instance: HINSTANCE) -> bool {
        let mut dll_window = self.inner.write();

        let class_name = win32_string(CLASS_NAME);
        let window_name = win32_string(WINDOW_NAME);

        dll_window.h_instance = h_instance;

        if !dll_window.register_dllwindow_class() {
            panic!("Unable to register window class!");
        }

        dll_window.handle = CreateWindowExW(
            0,
            class_name.as_ptr(),
            window_name.as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            200,
            100,
            null_mut(),
            null_mut(),
            dll_window.h_instance,
            null_mut(),
        );

        if dll_window.handle.is_null() {
            panic!("Unable to create window!");
        }

        SetWindowLongPtrW(
            dll_window.handle,
            GWLP_USERDATA,
            transmute(self.inner.clone()),
        );

        ShowWindow(dll_window.handle, SW_HIDE);
        UpdateWindow(dll_window.handle);

        true
    }
}

pub struct DllWindowInner {
    callbacks: HashMap<u32, OnMessageCallback>,
    h_instance: HINSTANCE,
    handle: HWND,
}

impl DllWindowInner {
    pub fn new() -> DllWindowInner {
        DllWindowInner {
            callbacks: HashMap::new(),
            h_instance: null_mut(),
            handle: null_mut(),
        }
    }

    pub unsafe extern "system" fn dll_window_proc(
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let self_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        if msg == WM_DESTROY {
            PostQuitMessage(0);
            return 0;
        } else {
            if self_ptr != 0 {
                let dll_window: Arc<RwLock<DllWindowInner>> = transmute(self_ptr);
                for (id, callback) in dll_window.read().callbacks.iter() {
                    if msg == *id {
                        callback(hwnd, wparam, lparam);
                    }
                }
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            }
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
    }

    pub fn get_handle_as_int(&self) -> i32 {
        self.handle as i32
    }

    pub unsafe fn register_dllwindow_class(&mut self) -> bool {
        let class_name = win32_string(CLASS_NAME);

        let wcex = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(DllWindowInner::dll_window_proc),
            hInstance: self.h_instance,
            lpszClassName: class_name.as_ptr(),
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

    pub unsafe fn unregister_dllwindow_class(&mut self) -> bool {
        let class_name = win32_string(CLASS_NAME);

        if UnregisterClassW(class_name.as_ptr(), self.h_instance) == FALSE {
            return false;
        }

        return true;
    }

    pub unsafe fn close(&mut self) -> bool {
        self.unregister_dllwindow_class() && CloseWindow(self.handle) == TRUE
    }

    pub unsafe fn peek_wnd_message() {
        let mut msg: MSG = zeroed();

        if PeekMessageW(&mut msg as *mut MSG, 0 as HWND, 0, 0, PM_REMOVE) == TRUE {
            TranslateMessage(&msg as *const MSG);
            DispatchMessageW(&msg as *const MSG);
        }
    }

    pub fn register_on_message_callback(&mut self, message: u32, callback: OnMessageCallback) {
        if !self.callbacks.contains_key(&message) {
            self.callbacks.insert(message, callback);
        }
    }

    pub fn un_register_on_message_callback(&mut self, message: u32) {
        if self.callbacks.contains_key(&message) {
            self.callbacks.remove_entry(&message);
        }
    }
}
