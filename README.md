<p align="center">
 <img src="https://raw.githubusercontent.com/mariinkys/starrydex/main/res/icons/hicolor/256x256/apps/dev.mariinkys.StarryDex.svg">
</p>

<h1 align="center">StarryDex</h1>

<p align="center">
 This project contains a small Pokédex application for the COSMIC™ desktop written in Rust with <a href="https://github.com/pop-os/libcosmic" target="_blank">libcosmic</a>.
</p>

<p align="center">
 <img src="https://raw.githubusercontent.com/mariinkys/starrydex/main/screenshots/main.png" width=350>
 <img src="https://raw.githubusercontent.com/mariinkys/starrydex/main/screenshots/pokemon.png" width=350>
</p>

## Information

> [!TIP]
> The application can work offline after initial setup is completed.

> [!WARNING]
> Beware there are some visual bugs when COSMIC is not installed on the system.
> (It seems like you do not need to be using COSMIC, but you need to have it installed to avoid visual issues with libcosmic).

Created by [mariinkys](https://github.com/mariinkys). Pokémon and Pokémon character names are trademarks of Nintendo.

This application uses [PokeApi](https://github.com/PokeAPI/) and it's resources. 

Bundled Icons:  "[Cosmic Icons](http://github.com/pop-os/cosmic-icons)" by [System76](http://system76.com/) is licensed under [CC-SA-4.0](http://creativecommons.org/licenses/by-sa/4.0/)

## Install

To install your COSMIC application, you will need [just](https://github.com/casey/just), if you're on Pop!\_OS, you can install it with the following command:

```sh
sudo apt install just
```

After you install it, you can run the following commands to build and install your application:

```sh
just build-release
sudo just install
```
