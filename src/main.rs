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

fn main() -> std::io::Result<()> {
    let mut path = String::new();
    let args = std::env::args().nth(1);
    match args {
        Some(arg) => path = arg,
        None => {
            println!("Whar is the file list (.txt file containing the paths to the ds roms, with each rom path on a seperate line):");
            stdin().read_line(&mut path).unwrap();
        }
    }

    path = path.trim_matches(['\n', '\r', '\'', '\"']).to_owned();

    if path.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidFilename,
            "You can't point me to nothing, sorry!",
        ));
    }

    // I really don't know why a temporary variable is needed here. Now I can (and did)
    // codegolf this but I really want this one to be feature rich in the future.
    let temp_1 = fs::read_to_string(path)?;
    let paths = temp_1.lines();

    for p in paths {
        let p_str = p.to_string();
        let path = Path::new(p_str.trim_matches(['\'', '\"']).try_into().unwrap());

        // This should be a file...
        let oldname = path.file_name().unwrap().to_str().unwrap();

        let rom = fs::read(path).unwrap();
        // Offset of banner file relative to rom start plus offset of english title
        // relative to banner
        let offset =
            u32::from_le_bytes([rom[0x68], rom[0x69], rom[0x6A], rom[0x6B]]) as usize + 832;
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

        name += ".nds";

        let new_path = path.parent().unwrap().join(name.clone());

        match fs::rename(path, new_path) {
            Ok(_) => {
                println!("{oldname} -> {name}");
            },
            Err(err) => {
                if err.kind() == ErrorKind::AlreadyExists {
                    println!("Game {name} is duplicated!");
                } else {
                    println!("Error encountered: {err}");
                }
            }
        }
    }

    Ok(())
}
