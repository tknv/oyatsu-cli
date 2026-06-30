[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/tknv/oyatsu-cli)  

# oyatsu (CLI Version)

A command-line tool, porting the logic of [Oyatsu (Android Version)](https://github.com/tknv/Oyatsu) to help keep track of "oyatsu" (snack) time.

It displays the Japanese calendar year (Wareki), traditional Japanese month names (Wafūgetsu-mei), traditional Japanese time (seasonal time clock) / Futei-jiho), the 24 solar terms (Nijūshi-sekki), and the sexagenary cycle (Eto) in your terminal.
During the hour of the "Goat" (Hitsuji-no-koku, or Yatsu-doki, which is roughly 13:00 to 15:00 in modern time), the word "おやつ" (Oyatsu) will be prefixed to the output.

## Build

Rust (`cargo`) is required.

```sh
cargo build --release
```

The executable file will be created at `target/release/oyatsu`. You can copy it to a location in your `PATH` (e.g., `~/.local/bin` or `/usr/local/bin`) to use it as the `oyatsu` command.

```sh
sudo cp target/release/oyatsu /usr/local/bin/oyatsu
# or
cp target/release/oyatsu ~/.local/bin/oyatsu
# or
sudo make install
```

## Usage

```sh
oyatsu
```

```
Reiwa 8  Minazuki (Jun)  I 3  Summer Solstice  Hinoe-Horse
(Sunrise 03:45, Sunset 19:15)
```

During snack time (Hour of the Goat):

```
Oyatsu (snack time!) Reiwa 8  Minazuki (Jun)  Hitsuji 3  Summer Solstice  Hinoe-Horse
(Sunrise 03:45, Sunset 19:15)
```

Sunrise and sunset times are displayed as supplementary information in the standard error output (stderr).

```sh
oyatsu 2> /dev/null   # Hide supplementary information
Reiwa 8  Minazuki (Jun)  Tori 3  Summer Solstice  Hinoe-Horse
```

### Location Settings

The traditional Japanese time (temporal hour system) calculates sunrise and sunset based on latitude and longitude.

Priority order:

1. Values specified with the `-lat` / `-lon` options
2. `~/.config/oyatsu` (Latitude on the 1st line, Longitude on the 2nd line)
3. If neither is available, Tokyo Station (approx. `35.6895`, `139.6917`) is used as the default.

```sh
# Specify latitude and longitude for a one-time execution
oyatsu -lat 35.0116 -lon 135.7681

# Using the configuration file
mkdir -p ~/.config
cat > ~/.config/oyatsu << EOF
35.0116
135.7681
EOF
oyatsu   # Can be executed without -lat or -lon

```

If only one of `-lat` or `-lon` is specified, the missing value is supplemented by the configuration file (if available), or falls back to the default (Tokyo Station).

## Specifications

* CUI (Runs in the terminal)
* Display items: Japanese calendar year, traditional month name, traditional Japanese time (temporal hour system), 24 solar terms, sexagenary cycle
* Prefixes "おやつ" (Oyatsu) during the hour of the Goat (Yatsu-doki)
* Can run without `-lat`/`-lon` if latitude/longitude are set in `~/.config/oyatsu`
* Can run via `oyatsu -lat <latitude> -lon <longitude>` even if `~/.config/oyatsu` does not exist

## License

[GNU GPLv3 or later](http://www.gnu.org/licenses/gpl.html)

(Inherited from the original project, [Oyatsu](https://github.com/tknv/Oyatsu))

### Oyatsu (Snack)

© [Wikipedia, CC BY-SA 4.0](https://ja.wikipedia.org/wiki/%E3%81%8A%E3%82%84%E3%81%A4)

### Temporal Hour System (Futei-jiho)

© [National Astronomical Observatory of Japan](https://eco.mtk.nao.ac.jp/koyomi/wiki/BBFEB9EF2FC4EABBFECBA1A4C8C9D4C4EABBFECBA1.html)

### Traditional Japanese Month Names (Wafūgetsu-mei)

Source: National Diet Library "The Calendar, History and Lore" [https://www.ndl.go.jp/koyomi/e/](https://www.ndl.go.jp/koyomi/e/)

### Dango (Dumpling)

Source: Ministry of Agriculture, Forestry and Fisheries Website [MAFF Our Regional Cuisines: Kushi-dango, Tokyo](https://www.maff.go.jp/j/keikaku/syokubunka/k_ryouri/search_menu/menu/34_29_tokyo.html#:~:text=%E5%9B%A3%E5%AD%90%E3%81%AF%E5%B9%B3%E5%AE%89%E6%99%82%E4%BB%A3%E3%81%AB,%E5%A3%B2%E3%82%8B%E5%BA%97%E3%82%82%E3%81%A7%E3%81%8D%E3%81%9F%E3%80%82)

> Skewer dango (Kushi-dango) with five dumplings on a single stick became so popular that it inspired the saying "Hana yori Dango" (dumplings over flowers) and spread nationwide, originating from Kyoto.
> In Tokyo during the Edo period, the five-dumpling skewer was also the mainstream and was sold for 5 mon coins per stick. However, after the 4 mon coin began circulating, some customers took advantage of the crowded shops by leaving a 4 mon coin and taking the 5-dumpling skewer illegally. Troubled by this, shop owners came up with a desperate measure: they reduced the number of dumplings to four per stick. This is recorded as the origin of the 4-dumpling skewer. Even today, skewer dango is predominantly **four per stick in Kanto (Eastern Japan)** and **five per stick in Kansai (Western Japan)**.

Copyright (C) [2026]

This work incorporates AI-assisted.

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see [https://www.gnu.org/licenses/](https://www.gnu.org/licenses/).

```

```
