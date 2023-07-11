extern crate winres;

// jepg/png to .ico converter
// https://image.online-convert.com/convert-to-ico
fn main() {
    static_vcruntime::metabuild();

    if cfg!(target_os = "windows") {

        let mut res = winres::WindowsResource::new();
        res.set_icon("app_icon.ico");
        res.compile().unwrap();
    }
}
