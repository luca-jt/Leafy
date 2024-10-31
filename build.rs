use winresource::WindowsResource;

fn main() {
    if cfg!(target_os = "windows") {
        WindowsResource::new()
            .set_icon(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/images/icon.ico"
            ))
            .compile()
            .unwrap();
    }
}
