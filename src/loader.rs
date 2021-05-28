use crate::{
    consts::{EXE_NAME, IN_MESSAGE_ID, MAPPED_FILE_NAME},
    enums::SecuRomRequest,
    message::Message,
    utils::{bytes_to_struct, read_mem, struct_to_bytes, write_mem},
    window::Window,
};

use anyhow::{anyhow, Result};
use log::{error, info};
use native_windows_gui::init as init_window;
use std::{
    env,
    mem::{self, zeroed},
    process::{exit, Command},
};
use winapi::{
    shared::minwindef::{DWORD, LRESULT, TRUE},
    um::winuser::{WM_CLOSE, WM_DESTROY},
};

use shared_memory::ShmemConf;

#[derive(Default)]
pub struct Loader;

impl Loader {
    pub fn start(&self) -> Result<()> {
        init_window()?;

        let cwd = env::current_dir()?;

        let window = Window::new()?;
        let mut message: Message = unsafe { zeroed() };

        message.in_message_id = IN_MESSAGE_ID;
        message.window_handle = {
            let handle = window.handle().ok_or_else(|| anyhow!("Invalid handle!"))?;
            handle as u32
        };

        let buf = struct_to_bytes(&message);
        let mem = ShmemConf::new()
            .os_id(MAPPED_FILE_NAME)
            .size(256)
            .create()?;

        unsafe {
            mem.as_ptr().copy_from(buf.as_ptr(), buf.len());
        }

        let mut child = Command::new(cwd.join(EXE_NAME)).spawn()?;
        let pid = child.id();

        info!("Game process id: {}", pid);

        let window_cb = Box::new(move |_hwnd, msg, _w, l| {
            info!("Loader received msg: {}", msg);

            match msg {
                WM_DESTROY | WM_CLOSE => exit(0),
                IN_MESSAGE_ID if l > 0 => {
                    let addr = l as usize;

                    match Self::process(pid, addr) {
                        Ok(res) => return Some(res),
                        Err(err) => {
                            error!("{}", err);
                        }
                    }
                }
                _ => {}
            }

            None
        });

        let dispatch_cb = Box::new(move || match child.try_wait() {
            Ok(Some(_)) => exit(0),
            Err(err) => error!("{}", err),
            _ => {}
        });

        window.listen(window_cb, dispatch_cb)?;

        Ok(())
    }

    fn process(pid: u32, addr: usize) -> Result<LRESULT> {
        let mut buf = [0x0; mem::size_of::<Message>()];
        let len = buf.len();

        let _ = read_mem(pid, addr, &mut buf, len)?;

        #[repr(C)]
        struct Response {
            value: DWORD,
        }

        let msg = bytes_to_struct::<Message>(&buf);
        let req: SecuRomRequest = msg.in_message_id.into();

        info!(
            "Received message from game process id: {:?} -> {:#?}",
            req, msg
        );

        match req {
            SecuRomRequest::Hit => {
                return Ok((msg.payload.address + msg.payload.value) as LRESULT);
            }
            SecuRomRequest::Fly => {
                let addr = msg.payload.address as usize;
                let resp = Response { value: 1 };

                info!("Write response to addr: {}", addr);

                let buf = struct_to_bytes(&resp);
                let _ = write_mem(pid, addr, &buf, mem::size_of::<Response>())?;
            }
            _ => {}
        }

        Ok(TRUE as LRESULT)
    }
}
