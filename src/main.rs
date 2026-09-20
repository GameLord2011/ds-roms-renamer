use std::{
    fs,
    io::{Error, ErrorKind, stdin},
    path::Path,
};

#[cfg(not(target_os = "macos"))]
#[used]
#[unsafe(link_section = ".text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

// The XNU kernel has strange executable section handling.
#[cfg(target_os = "macos")]
#[used]
#[unsafe(link_section = "__TEXT,__text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

fn rename_rom<P: AsRef<Path>>(rom: P) {
    let path = rom.as_ref();
    let oldname = path.file_name().unwrap().to_str().unwrap();

    let rom = fs::read(path).unwrap();
    // Checks if the rom is less then 359 bytes long or the Nintendo logo & associated
    // checksum are not present at offset `0xC0`. I know that technically the smallest DS
    // rom is only 352 bytes but that doesn't have a banner file and thus you can't do
    // this renaming process on it. In the future I __may__ add gameid renaming but I
    // don't want to right now as that is close to gibberish that no one but technical DS
    // people can read or understand.
    if rom.len() < 0x15D
        || rom[0xC0..0x15E]
            != [
                0x24, 0xFF, 0xAE, 0x51, 0x69, 0x9A, 0xA2, 0x21, 0x3D, 0x84, 0x82, 0x0A, 0x84, 0xE4,
                0x09, 0xAD, 0x11, 0x24, 0x8B, 0x98, 0xC0, 0x81, 0x7F, 0x21, 0xA3, 0x52, 0xBE, 0x19,
                0x93, 0x09, 0xCE, 0x20, 0x10, 0x46, 0x4A, 0x4A, 0xF8, 0x27, 0x31, 0xEC, 0x58, 0xC7,
                0xE8, 0x33, 0x82, 0xE3, 0xCE, 0xBF, 0x85, 0xF4, 0xDF, 0x94, 0xCE, 0x4B, 0x09, 0xC1,
                0x94, 0x56, 0x8A, 0xC0, 0x13, 0x72, 0xA7, 0xFC, 0x9F, 0x84, 0x4D, 0x73, 0xA3, 0xCA,
                0x9A, 0x61, 0x58, 0x97, 0xA3, 0x27, 0xFC, 0x03, 0x98, 0x76, 0x23, 0x1D, 0xC7, 0x61,
                0x03, 0x04, 0xAE, 0x56, 0xBF, 0x38, 0x84, 0x00, 0x40, 0xA7, 0x0E, 0xFD, 0xFF, 0x52,
                0xFE, 0x03, 0x6F, 0x95, 0x30, 0xF1, 0x97, 0xFB, 0xC0, 0x85, 0x60, 0xD6, 0x80, 0x25,
                0xA9, 0x63, 0xBE, 0x03, 0x01, 0x4E, 0x38, 0xE2, 0xF9, 0xA2, 0x34, 0xFF, 0xBB, 0x3E,
                0x03, 0x44, 0x78, 0x00, 0x90, 0xCB, 0x88, 0x11, 0x3A, 0x94, 0x65, 0xC0, 0x7C, 0x63,
                0x87, 0xF0, 0x3C, 0xAF, 0xD6, 0x25, 0xE4, 0x8B, 0x38, 0x0A, 0xAC, 0x72, 0x21, 0xD4,
                0xF8, 0x07, 0x56, 0xCF,
            ]
    {
        println!(
            "File {oldname} is too short or does not have the Nintendo logo and checksum! Skipping."
        )
    }
    // From here on out it's assumed that it is a valid ROM.

    // Offset of banner file relative to rom start plus offset of english title
    // relative to banner
    let offset = u32::from_le_bytes([rom[0x68], rom[0x69], rom[0x6A], rom[0x6B]]) as usize + 832;
    let mut name = String::from_utf16_lossy(
        &rom[offset..offset + 256]
            .chunks(2)
            .map(|e| u16::from_le_bytes(e.try_into().unwrap()))
            .collect::<Vec<u16>>(),
    );
    // Two seperate ones because [TODO: INSERT VALID REASON HERE].
    name = name.replace("\n", " ").replace("\0", "");

    // Invalid symbols for NTOS paths vs. POSIX paths.
    #[cfg(target_os = "windows")]
    {
        name = name.replace(['<', '>', ':', '"', '/', '\\', '|', '?', '*'], "");
    }
    #[cfg(not(target_os = "windows"))]
    {
        name = name.replace('/', "");
    }

    match rom[15] {
        // Standard reigon codes from the gameid (EA doesn't respect this 9 / 10
        // times).
        0x35 => name += "US",
        0x43 => name += "CH",
        0x4A => name += "JP",
        0x50 => name += "EU",
        0x55 => name += "AU",
        0x5B => name += "KE",
        _ => (),
    }

    name += ".";
    name += path.extension().unwrap().to_str().unwrap();

    let new_path = path.parent().unwrap().join(name.clone());

    match fs::rename(path, new_path) {
        Ok(_) => {
            println!("{oldname} -> {name}");
        }
        Err(err) => match err.kind() {
            ErrorKind::AlreadyExists => println!("Game {name} is duplicated!"),
            _ => println!("Error encountered: {err}"),
        },
    }
}

fn main() -> std::io::Result<()> {
    let mut path = String::new();
    let args = std::env::args().nth(1);
    match args {
        Some(arg) => path = arg,
        None => {
            println!(
                "Where are the files? Or a file containing a list of the files on seperate lines is fine too."
            );
            stdin().read_line(&mut path)?;
        }
    }

    path = path.trim_matches(['\n', '\r', '\'', '"']).to_owned();

    if path.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidFilename,
            "You can't point me to nothing, sorry!",
        ));
    }

    let p = Path::new(&path);

    if p.is_dir() {
        for rom in fs::read_dir(p).unwrap().flatten() {
            rename_rom(rom.path());
        }
    } else if p.is_file() {
        let paths = fs::read_to_string(p).unwrap();
        for p in paths.lines() {
            rename_rom(Path::new(p.trim_matches(['\n', '\r', '\'', '"'])));
        }
    } else {
        panic!("The path is neither a file nor a directory. Hwat?");
    }

    Ok(())
}
