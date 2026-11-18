use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=../../assets/art/parkbound.ico");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let icon = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../../assets/art/parkbound.ico")
        .canonicalize()
        .expect("Parkbound icon must exist");
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("parkbound.rc");
    let icon_path = icon.to_string_lossy().replace('\\', "\\\\");
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    let numbers = format!(
        "{},{},{},0",
        env::var("CARGO_PKG_VERSION_MAJOR").unwrap(),
        env::var("CARGO_PKG_VERSION_MINOR").unwrap(),
        env::var("CARGO_PKG_VERSION_PATCH").unwrap(),
    );
    fs::write(
        &output,
        format!(
            r#"1 ICON "{icon_path}"
1 VERSIONINFO
FILEVERSION {numbers}
PRODUCTVERSION {numbers}
FILEOS 0x40004L
FILETYPE 0x1L
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904b0"
        BEGIN
            VALUE "FileDescription", "Parkbound\0"
            VALUE "FileVersion", "{version}\0"
            VALUE "ProductName", "Parkbound\0"
            VALUE "ProductVersion", "{version}\0"
            VALUE "OriginalFilename", "Parkbound.exe\0"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x409, 1200
    END
END
"#
        ),
    )
    .expect("write Windows icon resource");
    embed_resource::compile(&output, embed_resource::NONE)
        .manifest_required()
        .expect("compile the Parkbound Windows icon with the Windows SDK");
}
