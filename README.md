# Disconic

A bot to play your subsonic-compatible server's song on discord!

At the moment, one bot instance means one subsonic instance, and the bot only uses a single song queue, so there's no support for multi-server bots. Running this is pretty simple, though!

I use `sunk` to interact with subsonic, and `serenity`/`poise` to interact with discord.

## Dependencies

You need `rustc` >= 1.70 to compile this crate. Use `rustup` to get it.

The only crate with non-rust dependencies is `audiopus_sys`, this means you need `libopus` installed to run this. If building, you'll also need `cmake` and `pkg-config`. Consult your distro's documentation on how to get these.

You can quickly get everything you need with nix:

```bash
nix develop
```

## Building

To run from source, clone the repo and run:

```bash
# Build
cargo build --release && ./target/release/disconic --help
# Or build and run
cargo run --release -- --help
```

If you use nix:

```bash
# Build
nix build . && ./result/bin/disconic --help
# Or build and run
nix run . -- --help

# Or get a shell with disconic
nix shell .
disconic --help
```

With nix, you don't even need to clone the repo. Simply replace `.` with `github:misterio77/disconic`.

## Usage

Start by creating a discord app, getting its bot token, and inviting it to your server. Also get your guild (server) ID. These steps are already documented elsewhere, so will not be covered here.

You can configure the application through CLI arguments (very convenient) or environment variables (better for deployments, dotenv is also supported). Use `--help` to see what the arguments or environment variables are.

Simply run the binary and everything will be setup for you. The bot will automatically register its commands on your server (if guild_id is set). If you don't see the commands, try sending a message mentioning the bot and `register` (all commands can be ran like that, too).

## Usage

You can use `/song` to search for individual songs, `/album` to search for entire albums, `/random` to queue a random song.

You may use `/pause`, `/resume`, and `/stop` to control playback.

Your queue can be viewed with `/queue`, songs can be removed with `/remove` and/or `/skip`.

The bot should run your voice channel automatically when you do any command related to it. You can run `/join` to call it explicitly, and `/leave` to make it go away.

## Contributing

Patches are welcome! See the "building" section on how to hack on the package.

I will not try to document the project structure, as this will just get outdated. Just get `rust-analyzer` and jump around the project, it should be pretty self-explanatory.

## TODO

- Dialog to choose among multiple matches
- Implement volume
- Show album arts
- Playlist support
- Seeking
- Multi-guild support?
