<div align="center">
<br>
<img src="https://raw.githubusercontent.com/mariinkys/starrydex/main/res/icons/hicolor/256x256/apps/dev.mariinkys.StarryDex.svg">
<h1 align="center">StarryDex</h1>

![Flathub Version](https://img.shields.io/flathub/v/dev.mariinkys.StarryDex)
![Flathub Downloads](https://img.shields.io/flathub/downloads/dev.mariinkys.StarryDex)
![GitHub License](https://img.shields.io/github/license/mariinkys/starrydex)
![GitHub Repo stars](https://img.shields.io/github/stars/mariinkys/starrydex)

<h3>A Pokédex application for the COSMIC™ desktop</h3>

<img src="https://raw.githubusercontent.com/mariinkys/starrydex/main/screenshots/main.png" width=350>
<img src="https://raw.githubusercontent.com/mariinkys/starrydex/main/screenshots/pokemon.png" width=350>

<br><br>

<a href="https://flathub.org/apps/dev.mariinkys.StarryDex">
   <img width='240' alt='Download on Flathub' src='https://flathub.org/api/badge?locale=en'/>
 </a>
 </div>

## Information
> [!TIP]
> The application can work offline after initial setup is completed.

> [!CAUTION]
> Right now the application configuration does not get saved and it's lost upon application restart.

Created by [mariinkys](https://github.com/mariinkys). Pokémon and Pokémon character names are trademarks of Nintendo.

This application uses [PokeApi](https://github.com/PokeAPI/) and its resources.

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
