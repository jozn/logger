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

fn main() {
    loop {
        for i in 1..256 {
            let key_code = unsafe { GetAsyncKeyState(i) };
            if key_code == 0 {
                continue;
            }
            if key_code & (1 << 15) == 0 {
                continue;
            }

            let mut keyboard_state = [0u8; 256];
            let t =  unsafe { GetKeyboardState(&mut keyboard_state) };
            println!("{:?}",t);
            println!("{:?}",keyboard_state);

            // let layout = unsafe { GetKeyboardLayout(0) };
            // let layout = unsafe { GetKeyboardLayout(GetCurrentThreadId()) };

            // First get the handle to the foreground window
            let foreground_window = unsafe { GetForegroundWindow() };

// Now get the thread id of the foreground window
//             let thread_id = unsafe { GetWindowThreadProcessId(foreground_window, std::ptr::null_mut()) };
            let thread_id = unsafe { GetWindowThreadProcessId(foreground_window, None) };

// Finally get the keyboard layout for the foreground thread
            let layout = unsafe { GetKeyboardLayout(thread_id) };

            let mut pwszbuff = [0u16; 10]; // Create a buffer

            // Use the correct arguments for ToUnicodeEx
            let code = unsafe {
                ToUnicodeEx(
                    i as u32, // Use the key code as `wvirtkey`
                    0, // `wscancode`, it's usually obtained from a WM_KEYDOWN or WM_KEYUP message.
                    &keyboard_state,
                    &mut pwszbuff, // Pass the buffer
                    0, // `wflags`, always 0 for this usage
                    layout
                )
            };

            println!("typeing {}: {} kb: {:?}", i, code, layout);
            println!(">>> buff {:?}",  pwszbuff);

            // let code_point: u32 = 1567;
            let code_point: u32 = pwszbuff[0] as u32;
            if let Some(character) = char::from_u32(code_point) {
                // let character = character.to_string();
                if character.is_control() {
                    println!("{} is a control character", code_point);
                } else {
                    let character_string = character.to_string();
                    println!("{}", character_string);
                }
                // println!(">>> {}", character_string);
            } else {
                println!("Invalid Unicode code point: {}", code_point);
            }

        }
    }
}
fn main1() {
    let mut keycodes = vec![];
    // Iterate from 0 to 255 inclusive
    for i in 0..=255 {
        // let vk = match VirtualKey::try_from(i) {
        //     Ok(vk) => vk,
        //     Err(_) => continue,  // Skip invalid values
        // };

        if !is_known_win_key(i){
            continue;
        }
        keycodes.push(i);

        let state = unsafe { GetAsyncKeyState(i) };

        // Print the state of the key
        println!("VirtualKey {:?}: {}", i, state);
    }

    loop {
        for i in keycodes.iter() {
            let key_code = unsafe { GetAsyncKeyState(*i) };
            if key_code == 0 {
                continue;
            }
            if key_code & (1 << 15) == 0 {
                continue;
            }

            let mut keyboard_state = [0u8; 256];
            let t =  unsafe {GetKeyboardState(&mut keyboard_state)};
            println!("{:?}",t);
            println!("{:?}",keyboard_state);

           let layout = unsafe {GetKeyboardLayout(0)};

            let pstr = PWSTR(0 as *mut u16);
            // let code = unsafe {ToUnicodeEx(key_code as u32, 0, &mut keyboard_state, &pstr, 1, 1, layout)};

            println!("typeing {:?}: {} kb: {:?}", i, key_code, layout);
        }


        thread::sleep(Duration::from_millis(5));
    }

    unsafe {
        let mut keyboard_state = [0u8; 256];
        let t = GetKeyboardState(&mut keyboard_state);
        println!("{:?}",t);
        println!("{:?}",keyboard_state)
    }

}

/*pub fn to_unicode(wvirtkey: u32, wscancode: u32) -> Result<Option<String>> {
    unsafe {
        let mut keyboard_state = [0u8; 256];
        let mut buffer_ = [0u16; 2];
        let mut buffer = 0u16;

        let x = GetKeyboardState(&mut keyboard_state);

        let unicode_len = ToUnicode(
            wvirtkey,
            wscancode,
            Some(&keyboard_state),
             &mut buffer,
            buffer.len().try_into().unwrap(),
        );

        match unicode_len {
            0 | -1 => Ok(None), // no translation or dead key
            _ => {
                // Convert to a Rust String
                let result = String::from_utf16_lossy(&buffer[0..unicode_len as usize]);
                Ok(Some(result))
            }
        }
    }
}
*/
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


fn f1() {
    let device_state = DeviceState::new();
    let buffer = Arc::new(Mutex::new(String::new()));

    let buffer_clone = Arc::clone(&buffer);
    thread::spawn(move || {
        loop {

            thread::sleep(Duration::from_secs(4));
            let buffer_lock = buffer_clone.lock().unwrap();
            println!("Buffer: {}", *buffer_lock);
        }
    });

    loop {
        let keys: Vec<Keycode> = device_state.get_keys();

        if keys.contains(&Keycode::H) || keys.contains(&Keycode::M) {
            let mut buffer_lock = buffer.lock().unwrap();
            for key in keys.iter() {
                match key {
                    Keycode::H => buffer_lock.push_str("h"),
                    Keycode::M => buffer_lock.push_str("m"),
                    _ => {}
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}
