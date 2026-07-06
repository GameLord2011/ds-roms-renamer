use std::{
    fs,
    io::{Error, ErrorKind, stdin},
    path::Path,
};

#[cfg(not(target_os = "macos"))]
#[used]
#[unsafe(link_section = ".text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

#[cfg(target_os = "macos")]
#[used]
#[unsafe(link_section = "__TEXT,__text")]
static MESSAGE: [u8; include_bytes!("message.txt").len()] = *include_bytes!("message.txt");

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect::<Vec<String>>();
    let mut path = String::new();

    if args.len() > 1 {
        path = args[1].clone();
    } else {
        println!("Whar is the file list (.txt file containing the paths to the ds roms):");
        stdin().read_line(&mut path).unwrap();
    }

    path = path.trim_matches(&['\n', '\r', '\'', '\"']).try_into().unwrap();

    if path.is_empty() {
        println!("You can't point me to nothing, sorry!");
        return Err(Error::new(
            ErrorKind::InvalidFilename,
            "You can't point me to nothing, sorry!",
        ));
    }

    let p = Path::new(&path);
    // I really don't know why a temporary variable is needed here.
    let temp_1 = fs::read_to_string(p).unwrap();
    let paths = temp_1.lines().collect::<Vec<&str>>();

    for p in paths {
        let mut path: String = p.to_string();
        path = path.trim_matches(&['\'', '\"']).try_into().unwrap();

        let oldname ;
        let mut og_name_idx;

        #[cfg(target_os = "windows")]
        {
            og_name_idx = path.rfind("\\").unwrap_or(0);
            if og_name_idx == 0 {
                og_name_idx = path.rfind("/").unwrap();
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            og_name_idx = new_path.rfind("/").unwrap();
        }

        og_name_idx += 1;
        oldname = path.get(og_name_idx..path.len()).unwrap();

        let rom = fs::read(&path).unwrap();
        // Offset of banner file relative to rom start plus offset of english title
        // relative to banner
        let offset =
            u32::from_le_bytes([rom[0x68], rom[0x69], rom[0x6A], rom[0x6B]]) as usize + 832; 
        let mut name = String::from_utf16_lossy(
            &rom[offset..(offset + 256)]
                .chunks(2)
                .map(|e| u16::from_le_bytes(e.try_into().unwrap()))
                .collect::<Vec<u16>>()
        );
        // Two seperate ones because [TODO: INSERT VALID REASON HERE].
        name = name.replace("\n", " ").replace("\0", "");
        match rom[15] {
            // Standard reigon codes from the gameid (EA doesn't respect this 9 / 10
            // times)
            0x35 => name = name + "US",
            0x43 => name = name + "CH",
            0x4A => name = name + "JP",
            0x50 => name = name + "EU",
            0x55 => name = name + "AU",
            0x5B => name = name + "KE",
            _ => ()
        }

        #[cfg(target_os = "windows")] {
            name = name.replace(&['<', '>', ':', '\"', '/', '\\', '|', '?', '*'], "");
        }
        #[cfg(not(target_os = "windows"))] {
            name = name.replace("/", "");
        }

        println!("{oldname} -> {name}.nds");

        let mut i: usize;
        let mut new_path = path.clone();

        #[cfg(target_os = "windows")]
        {
            i = new_path.rfind("\\").unwrap_or(0);
            if i == 0 {
                i = path.rfind("/").unwrap();
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            i = new_path.rfind("/").unwrap();
        }

        i += 1;
        loop {
            if i >= new_path.len() {
                break;
            }
            new_path.remove(i);
        }

        new_path = new_path + &name + ".nds";
        match fs::rename(path, new_path) {
            Ok(_) => (),
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
