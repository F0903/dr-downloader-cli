#[cfg(all(windows, not(debug_assertions)))]
use winapi::um::{consoleapi, errhandlingapi, processenv, winbase, wincon};

/// Set virtual console mode to use colors, or other control characters. (win32)
#[cfg(all(windows, not(debug_assertions)))]
pub fn set_virtual_console_mode() {
    let mut error = false;
    unsafe {
        let out = processenv::GetStdHandle(winbase::STD_OUTPUT_HANDLE);
        let mut out_mode: u32 = 0;
        if consoleapi::GetConsoleMode(out, &mut out_mode as *mut u32) == 0 {
            let err = errhandlingapi::GetLastError();
            println!("GetConsoleMode failed with err code {}", err);
            error = true;
        }
        out_mode |= wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        if consoleapi::SetConsoleMode(out, out_mode) == 0 {
            let err = errhandlingapi::GetLastError();
            println!("SetConsoleMode failed with err code {}", err);
            error = true;
        }
    }

    if error {
        println!("Error(s) detected in set_virtual_console_mode(). The console might contain strange characters due to this.");
    }
}
