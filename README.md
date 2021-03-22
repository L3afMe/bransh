<p align="center">
    <h1 align="center">Bransh</h1>
    <p align="center">A simple shell written in Rust</p>
</p>

## Building

As this is still a very early beta there are no releases yet and Bransh has to be built from source.

#### Prerequisites
 - Rustc
 - Cargo

To build the latest nightly, run the following
```sh
git clone https://github.com/L3afMe/bransh
cd bransh
cargo build --release
```

## Installing

Bransh can be run as normal or be set to default shell with the following

(Currently can't be run from inside Bransh as `>>` isn't implemented yet)
```sh
cp target/release/bransh /usr/bin/bransh
sudo echo /usr/bin/bransh >> /etc/shells
chsh -s /usr/bin/bransh
```

## Config

Config can be modified from `branshrc.br` which is located in `ENV:XDG_CONFIG_DIR/bransh/` (`~/.config/bransh/branshrc.br` by default).<br>
All lines in here will be executed as if they're run in shell. When you first run Bransh, a config will be created with the default values but you can also refer to [the wiki](https://github.com/L3afMe/bransh/wiki/Options) for values that exist and their defaults.

Example `branshrc.br`
```
set PROMPT "{WD} | "                                                                                                            
                                                                                                                                
set P_HOME_TRUNC true                                                                                                           
set P_HOME_CHAR  "~"                                                                                                            
                                                                                                                                 
set P_DIR_TRUNC 2                                                                                                               
set P_DIR_CHAR  "…"                                                                                                             
                                                                                                                                 
set SYN_HIGHLIGHTING true
```
