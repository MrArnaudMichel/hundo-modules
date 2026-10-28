# Hundo — game modules

[Hundo](https://github.com/MrArnaudMichel/hundo) tells you what is left to do
in your games, read from your own save files. It learns each game through a
**game module**: a small WebAssembly program that reads a save and reports
what is done. This repository holds everything to write one, free and open:

- `wit/driver.wit` — the contract between Hundo and a module;
- `drivers/sdk` — the Rust SDK (`hundo-sdk`) that implements it;
- `drivers/<game>` — the official modules, as real examples: Clair Obscur:
  Expedition 33, Hades, Hollow Knight, Hollow Knight: Silksong, Schedule I;
- `catalog/catalog.json` — the modules Hundo offers to install and update.

A module never touches the disk or the network: Hundo gives it the save's
files and runs it in a sandbox.

## Write a module

```sh
rustup target add wasm32-wasip2
hundo module new my-game --name "My Game"   # a project ready to build
cd my-game && cargo build --release
hundo module check . --save <a save file>   # try it on a real save
hundo module pack . -o my-game.hundo        # a package to share
```

The `hundo` command comes with Hundo's releases. See
[docs/AUTHORING.md](docs/AUTHORING.md) for the manifest (`driver.toml`), the
report a module returns, options, the default page layout, and the rules of
the catalog.

To use the SDK from your own repository:

```toml
[dependencies]
hundo-sdk = { git = "https://github.com/MrArnaudMichel/hundo-modules" }
```

## Get a module into Hundo

Open a pull request that adds it to `catalog/catalog.json` (see the guide):
its id, version, where its signed package is, and your public key. Hundo
then offers it to every reader, and updates it when you publish a new
version — without reinstalling the app.

## En français

Ce dépôt contient tout ce qu'il faut pour écrire un module de jeu pour
Hundo : le contrat, le SDK Rust, les modules officiels comme exemples et le
catalogue. Tout y est libre : MIT OR Apache-2.0, sauf Silksong (GPL-3.0-or-later, sa liste de complétion venant d'un projet GPL). Le guide d'auteur est dans
`docs/AUTHORING.md`.

## License

The contract, the SDK and the modules are licensed under either of
[Apache License 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT), at your
option — except where a module's `driver.toml` says otherwise: Hollow Knight:
Silksong is GPL-3.0-or-later, because its completion list comes from a GPL
project. Contributions are accepted under the license of what they change.
