use winresource::WindowsResource;


fn main() {

    if cfg!( target_os = "windows" ) {

        WindowsResource::new().set_icon("./res/images/icon.ico").compile().unwrap();

    }
}
