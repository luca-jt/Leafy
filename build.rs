use winresource::WindowsResource;

fn main() {
    if cfg!(target_os = "windows") {
        WindowsResource::new()
            .set_icon_with_id(
                concat!(env!("CARGO_MANIFEST_DIR"), "/assets/images/icon.ico"),
                "32512",
            )
            .compile()
            .unwrap();
    }
}
