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

fn rename_rom<P: AsRef<Path>>(rom: P, lang_idx: usize) {
    let path = rom.as_ref();
    let oldname = path.file_name().unwrap().to_str().unwrap();

    let rom = fs::read(path).unwrap();
    if rom.len() < 0x140 {
        println!(
            "File {oldname} is too short. Skipping."
        );
        return;
    }

    // See https://problemkaputt.de/gbatek-bios-misc-functions.htm; this was modified to
    // use the CRC-16-IBM reversed polynomial over the lookup table provided in the
    // psuedocode because apparently that psuedocode is incorrect.
    let mut crc = 0xFFFF;
    for &i in rom[0x0..0x15E].iter() {
        crc ^= i as u16;
        for _ in 0..8 {
            let carry = crc & 1 != 0;
            crc >>= 1;
            if carry {
                crc ^= 0xA001;
            }
        }
    }

    if crc != u16::from_le_bytes([rom[0x15E], rom[0x15F]]) {
        println!("Header checksum does not match for {oldname}! Skipping for saftey.");
        return;
    }

    // From here on out it's assumed that it is a valid ROM.

    let base_offset = u32::from_le_bytes([rom[0x68], rom[0x69], rom[0x6A], rom[0x6B]]) as usize;

    let mut name: String;
    if base_offset == 0x0 {
        println!(
            "File {oldname} has a banner offset of 0x0! Falling back to game title string (significantly less detailed!)"
        );
        // By definition this is valid UTF-8 and if you have a corrupted rom with a valid
        // CRC-16 you're kinda screwed anyway so there's rlly no point in checking if
        // this is valid UTF-8.
        name = unsafe { String::from_utf8_unchecked(rom[0x0..0xC].to_vec()) };
    } else {
        // Offset of banner file relative to rom start plus offset of chosen title
        // relative to banner
        let offset = base_offset + (832 * lang_idx);
        name = String::from_utf16_lossy(
            &rom[offset..offset + 256]
                .chunks(2)
                .map(|e| u16::from_le_bytes(e.try_into().unwrap()))
                .collect::<Vec<u16>>(),
        );
    }
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

    let new_path = path.parent().unwrap().join(&name);

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
    let args = std::env::args().collect::<Vec<String>>();

    let mut read_next = true;
    let mut lang_idx = 1 /* The DS rom rename index, defaults to english (1) */;
    for (i, arg) in args.iter().enumerate() {
        if i == 0 {
            continue;
        } // Skips the program path :P
        if read_next {
            match arg.to_ascii_lowercase().as_str() {
                "-h" => {
                    println!(include_str!("./help.txt"));
                    return Ok(());
                }
                "-l" | "-lang" => {
                    lang_idx = args[i + 1].parse::<usize>().unwrap_or(1);
                    read_next = false;
                }
                "-f" => {
                    path = args[i + 1].clone();
                    read_next = false;
                }
                _ => (),
            }
        }
    }

    if path.is_empty() {
        println!(
            "Where are the files? Or a file containing a list of the files on seperate lines is fine too."
        );
        stdin().read_line(&mut path)?;
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
            rename_rom(rom.path(), lang_idx);
        }
    } else if p.is_file() {
        for p in fs::read_to_string(p).unwrap().lines() {
            rename_rom(Path::new(p.trim_matches(['\n', '\r', '\'', '"'])), lang_idx);
        }
    } else {
        panic!("The path is neither a file nor a directory. Hwat?");
    }

    Ok(())
}
