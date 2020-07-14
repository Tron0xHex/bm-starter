use crate::consts::MAPPED_FILE_NAME;

use core::ptr::null_mut;
use winapi::{
    shared::minwindef::{LPVOID, TRUE},
    um::{
        handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
        memoryapi::{CreateFileMappingW, MapViewOfFile, FILE_MAP_ALL_ACCESS},
        winnt::{HANDLE, PAGE_READWRITE},
    },
};

pub struct FileMapping {
    file_handle: HANDLE,
    file_pointer: LPVOID,
}

impl FileMapping {
    pub fn new() -> Self {
        Self {
            file_handle: null_mut(),
            file_pointer: null_mut(),
        }
    }

    pub unsafe fn create(&mut self, size: usize) -> bool {
        self.file_handle = CreateFileMappingW(
            INVALID_HANDLE_VALUE,
            null_mut(),
            PAGE_READWRITE,
            0,
            size as u32,
            MAPPED_FILE_NAME.as_ptr(),
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

    pub fn get_file_ptr(&self) -> LPVOID {
        self.file_pointer
    }
}
