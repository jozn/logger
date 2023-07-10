#![windows_subsystem = "windows"]

use std::fs::File;
use std::io::prelude::*;
// use windows::Win32::
// use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::Shell::ShellExecuteW;
// use windows::Win32::Foundation::PWSTR;
use std::os::windows::prelude::*;
use windows::core::PWSTR;

// use windows::Win32::UI::Shell::SHOW_WINDOW_CMD;
use windows::Win32::Foundation::{HWND};
use std::os::windows::prelude::*;
use windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD;


const IMAGE_DATA: &'static [u8] = include_bytes!("image.jpg");
const IMAGE_FILE_OUT: &'static str = "./golzar.jpg";

fn main() {
    write_file();

    let file_path: Vec<u16> = std::env::current_dir()
        .unwrap()
        .join(IMAGE_FILE_OUT)
        .display()
        .to_string()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let open: Vec<u16> = "open".encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        ShellExecuteW(
            HWND::NULL,
            PCWSTR(open.as_ptr()),
            PCWSTR(file_path.as_ptr()),
            PCWSTR::default(),
            PCWSTR::default(),
            SHOW_WINDOW_CMD::SW_SHOWDEFAULT,
        );
    }

    std::thread::sleep(std::time::Duration::from_secs(3));
}

/*
const IMAGE_DATA: &'static [u8] = include_bytes!("image.jpg");
const IMAGE_FILE_OUT: &'static str = "./golzar.jpg";

fn main() {
    write_file();

    let file_path: Vec<u16> = std::env::current_dir()
        .unwrap()
        .join(IMAGE_FILE_OUT)
        .display()
        .to_string()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            PWSTR("open\0".as_ptr() as _),
            PWSTR(file_path.as_ptr() as _),
            PWSTR("\0".as_ptr() as _),
            PWSTR("\0".as_ptr() as _),
            1,
        );
    }

    std::thread::sleep(std::time::Duration::from_secs(3));
}
*/
fn write_file() {
    let mut file = match File::create(IMAGE_FILE_OUT) {
        Ok(file) => file,
        Err(e) => {
            return;
        }
    };

    if let Err(e) = file.write_all(IMAGE_DATA) {
        return;
    }
}
