use uniffi_build::uniffi_build;

fn main() {
    uniffi_build("yzchat.udl").expect("Failed to generate UniFFI bindings");
}