# pfui

Efficiently generate content for statusbars, especially eww

## The What

When you're setting up your status bar, there's a few things you probably want in there, examples are the current keyboard layout, the status of your music player, and most importantly the workspaces of your window manager.

Some status bars have modules for this built-in, for example [polybar](https://github.com/polybar/polybar), but this can also lead to it being limiting.

pfui provides modules like this in a generic way that you can can use with just about every configurable status bar. (_I'm writing this tool to use alongside my [eww](https://github.com/elkowar/eww) config, so that's what it's mainly developed for, but you should be able to use it with every status bar that can run a script and listen for output._)

## The How

pfui provides built-in modules that start event listeners so the data is only updated when it needs to be, and, perhaps more importantly, it's updated exactly when something changes.

When you run the pfui executable with the module to start, for example mpd, pfui will continously run until you stop it, and whenever something happens with your music player, it outputs the data you might want to use in your status bar as a json string. Eww natively supports json, with other bars you might have to write a wrapper script to process the data the way you want to and output a string formatted for your bar.

## The Why

When a module is not built in, you have two options:
- you can write a shell script that starts an event listener just like pfui, then outputs data whenever something changes. This is annoying.
- you can just update the data every second, or minute, etc. This is inefficient. (Probably not enough to be noticable, to be clear) More importantly, this means you might get updates delayed.

In short, using pfui, when you pause your music, it says so in your statusbar instantly, no 1 second delay and no manual shell scripting required.

## Usage

At the time of writing this readme, the first real commit was about 20 minutes ago, so it's in very early development. Forgive me for not including detailed installation instructions.

### Build the project

Git clone, make sure you have cargo installed, run `cargo build --release`. By default, all modules are included, you can manually exclude/include some using feature flags, since there's currently only one module I won't add a list now because I'll forget to update it, just look in the Cargo.toml.

Optionally make sure the binary (`target/release/pfui`) is somewhere on your `PATH`.

### Running a module

Execute the binary and specify the module you want to run.

```
pfui start mpd
```

### Running from eww

In your `eww.yuck`:
```lisp
(deflisten mpd-info "pfui start mpd")
```
