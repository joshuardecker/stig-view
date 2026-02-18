# How to run the application
Currently, I am only building for Linux, however the application supports cross platform.

To build the application, install rust, and anything required for the compilation process, like a linker. Then simply run:
```cargo build --release```

# Useful keybinds
All of these features are accessible by clicking buttons, but here are some keybinds:
1. Control + q to quit
2. Control + tab to scroll next stig
3. Control + p for cmd prompt
4. Control + i to select a file
5. Control + o to select a folder

# How to build for flatpak
This is more for me, but this is how I have gotten it to build for flatpak.

```
flatpak run org.flatpak.Builder --repo<some dir> <build dir, another one than repo> io.github.joshuardecker.stig-view.yml --force-clean

flatpak build-bundle <repo dir> <export name> io.github.joshuardecker.stig-view
```
