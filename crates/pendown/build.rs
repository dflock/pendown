//! On Windows, gives pendown.exe its icon and version details, for Explorer,
//! the taskbar and the window. gpui loads the window's icon from the icon
//! resource with ID 1. gpui embeds its own manifest, so this doesn't.

use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=windows/pendown.ico");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("windows");
    let icon = dir
        .join("pendown.ico")
        .display()
        .to_string()
        .replace('\\', "\\\\");
    let [major, minor, patch] = ["MAJOR", "MINOR", "PATCH"]
        .map(|part| env::var(format!("CARGO_PKG_VERSION_{part}")).unwrap());
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    // An rc string writes a quote as two.
    let description = env::var("CARGO_PKG_DESCRIPTION")
        .unwrap()
        .replace('"', "\"\"");
    // FILEOS is VOS_NT_WINDOWS32 and FILETYPE is VFT_APP, written as numbers
    // so the file needs no headers.
    let rc = format!(
        r#"1 ICON "{icon}"

1 VERSIONINFO
FILEVERSION {major},{minor},{patch},0
PRODUCTVERSION {major},{minor},{patch},0
FILEOS 0x40004
FILETYPE 0x1
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904B0"
        BEGIN
            VALUE "FileDescription", "Pendown"
            VALUE "FileVersion", "{version}"
            VALUE "InternalName", "pendown"
            VALUE "OriginalFilename", "pendown.exe"
            VALUE "ProductName", "Pendown"
            VALUE "ProductVersion", "{version}"
            VALUE "Comments", "{description}"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x0409, 0x04B0
    END
END
"#
    );
    let path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("pendown.rc");
    fs::write(&path, rc).unwrap();
    embed_resource::compile(&path, embed_resource::NONE)
        .manifest_optional()
        .unwrap();
}
