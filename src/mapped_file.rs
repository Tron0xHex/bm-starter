use crate::win32::win32_string;
use winapi::um::winnt::{HANDLE, PAGE_READWRITE};

use winapi::shared::minwindef::{LPVOID, TRUE};

use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::memoryapi::{CreateFileMappingW, MapViewOfFile, FILE_MAP_ALL_ACCESS};

use crate::consts::MAPPED_FILE_NAME;

pub struct MappedFile {
    file_handle: HANDLE,
    file_pointer: LPVOID,
}

impl MappedFile {
    pub fn new() -> MappedFile {
        MappedFile {
            file_handle: std::ptr::null_mut(),
            file_pointer: std::ptr::null_mut(),
        }
    }

    pub unsafe fn create(&mut self, size: usize) -> bool {
        let mapped_file_name = win32_string(MAPPED_FILE_NAME);

        self.file_handle = CreateFileMappingW(
            INVALID_HANDLE_VALUE,
            std::ptr::null_mut(),
            PAGE_READWRITE,
            0,
            size as u32,
            mapped_file_name.as_ptr(),
        );

        if self.file_handle.is_null() {
            return false;
        }

        self.file_pointer = MapViewOfFile(self.file_handle, FILE_MAP_ALL_ACCESS, 0, 0, size);

        if self.file_pointer.is_null() {
            return false;
        }

        true
    }

    pub unsafe fn close(&self) -> bool {
        CloseHandle(self.file_handle) == TRUE
    }

    pub fn get_handle(&self) -> HANDLE {
        self.file_handle
    }

    pub fn get_file_ptr(&self) -> LPVOID {
        self.file_pointer
    }
}
