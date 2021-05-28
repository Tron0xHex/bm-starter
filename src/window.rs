use crate::consts::WINDOW_NAME;
use anyhow::Result;
use native_windows_gui::{
    bind_raw_event_handler, dispatch_thread_events_with_callback, unbind_raw_event_handler,
    Window as WinApiWindow, WindowFlags,
};
use winapi::shared::{
    minwindef::{LPARAM, LRESULT, UINT, WPARAM},
    windef::HWND,
};

pub struct Window {
    window: WinApiWindow,
}

impl Window {
    pub fn new() -> Result<Self> {
        let mut flags = WindowFlags::empty();

        flags.set(WindowFlags::WINDOW, true);
        flags.set(WindowFlags::VISIBLE, false);

        let mut window = Default::default();
        let _ = WinApiWindow::builder()
            .title(WINDOW_NAME)
            .size((200, 200))
            .flags(flags)
            .build(&mut window)?;

        Ok(Self { window })
    }

    pub fn listen<F, C>(&self, cb: Box<F>, dispatch_cb: Box<C>) -> Result<()>
    where
        F: Fn(HWND, UINT, WPARAM, LPARAM) -> Option<LRESULT> + 'static,
        C: FnMut() -> () + 'static,
    {
        let id = 0x10000;
        let handler = bind_raw_event_handler(&self.window.handle, id, move |hwnd, msg, w, l| {
            cb(hwnd, msg, w, l)
        })?;

        dispatch_thread_events_with_callback(dispatch_cb);
        unbind_raw_event_handler(&handler)?;

        Ok(())
    }

    pub fn handle(&self) -> Option<HWND> {
        self.window.handle.hwnd()
    }
}
