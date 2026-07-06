# GameLord's Ds Rom Renamer

This is a shrimple batch DS rom renamer that I made out of personal want to not spend 7
hours combing through my collection of random DS roms (that were legally obtained).

## Usage

First you need to make a roms list file. Nothing too fancy, just one rom path per line.
Then, install rust (as I will not distrubute executable files for no reason other than
I'm lazy) after installing Rust run:

```bash
cargo build --release
```

after building it run

```bash
[PATH TO EXECUTABLE HERE] [PATH TO ROMLIST HERE]
```

or just run the executable file on it's own and provide the path when prompted. Then,
assuming there are no errors, that's it! All your DS roms are now renamed.
