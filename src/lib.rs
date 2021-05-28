mod consts;
mod enums;
mod loader;
mod message;
mod utils;
mod window;

use anyhow::Result;
use loader::Loader;
use log::error;
use simplelog::{CombinedLogger, ConfigBuilder, LevelFilter, WriteLogger};
use std::fs::File;
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, TRUE};

const LOG_FILE: &str = "Log.log";

#[inline]
fn setup_logger() -> Result<()> {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        ConfigBuilder::new()
            .set_target_level(LevelFilter::Off)
            .set_location_level(LevelFilter::Debug)
            .set_time_format_str("%F %T%.3f")
            .build(),
        File::create(LOG_FILE)?,
    )])?;

    Ok(())
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub unsafe extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: DWORD,
    reserved: LPVOID,
) -> BOOL {
    const DLL_PROCESS_ATTACH: DWORD = 1;

    match call_reason {
        DLL_PROCESS_ATTACH => {
            if cfg!(debug_assertions) {
                if let Err(err) = setup_logger() {
                    panic!("Unable to setup logger!");
                }
            }

            if let Err(err) = Loader::default().start() {
                error!("{}", err);
            }
        }
        _ => (),
    }

    TRUE
}
