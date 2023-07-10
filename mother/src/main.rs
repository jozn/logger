use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

// Include the image file as a binary blob
const IMAGE_DATA: &'static [u8] = include_bytes!("image.jpg");


const IMAGE_FILE_OUT: &'static str = "./golzar.jpg";

fn main() {

    // unsafe { windows::Win32::System::Console::FreeConsole() };

    write_file();

/*    let file_path: Vec<u16> = std::env::current_dir()
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
            PWSTR(file_path.as_ptr()),
            PWSTR("\0".as_ptr() as _),
            PWSTR("\0".as_ptr() as _),
            1,
        );
    }*/

    // match Command::new("open")
    //     .arg(IMAGE_FILE_OUT)
    //     .spawn() {
    //     Ok(child) => {
    //         println!("Successfully opened the image.");
    //     },
    //     Err(e) => println!("Failed to execute command 222: {}", e),
    // };

    // match Command::new("mspaint.exe")
    //     .arg(IMAGE_FILE_OUT)
    //     .spawn() {
    //     Ok(child) => {
    //         println!("Successfully opened the image.");
    //     },
    //     Err(e) => println!("Failed to execute command 222: {}", e),
    // };


    // Open the image with the default program
    match Command::new("cmd")
        .args(&["/C", "start", "/B", IMAGE_FILE_OUT])  // add /B argument to start process in the background
        .spawn() {  // use spawn instead of status to not wait for the process to finish
        Ok(_) => {
            // println!("Successfully opened the image.");
        },
        Err(e) => println!("Failed to execute command: {}", e),
    };


   /* // Open the image with the default program
    match Command::new("cmd")
        .args(&["/C", "start",IMAGE_FILE_OUT])
        .status() {
        Ok(status) => {
            if status.success() {
                println!("Successfully opened the image.");
            } else {
                println!("Command executed, but reported failure.");
            }
        },
        Err(e) => println!("Failed to execute command 222: {}", e),
    };*/

    // sleep for 5 seconds
    // std::thread::sleep(std::time::Duration::from_secs(3));
}

fn write_file() {
    // Write the image to a file
    let mut file = match File::create(IMAGE_FILE_OUT) {
        Ok(file) => file,
        Err(e) => {
            // println!("Failed to create file: {}", e);
            return;
        }
    };

    if let Err(e) = file.write_all(IMAGE_DATA) {
        // println!("Failed to write to file: {}", e);
        return;
    }

    // Use the 'attrib' command to hide the file
    // This needs to be run in a subprocess, so it won't work if the Rust script is run in the background
    match Command::new("cmd")
        .args(&["/C", "attrib", "+H", IMAGE_FILE_OUT])
        .status() {
        Ok(status) => {
            if !status.success() {
                // println!("Command executed, but reported failure.");
            }
        },
        Err(e) => {
            // println!("Failed to execute command: {}", e)
        },
    };
}

