# Stig View
![](https://github.com/joshuardecker/stig-view/blob/road-to-0.1/showcase.gif)

# How to run the application
To run the application:
- Install cargo
- run ```cargo run --release -p stig-view-desktop``` in the base directory of this project

# Experimental Software
This software is not fully complete. Some features are not implemented yet, and bugs may be found. If you have suggestions, or find a bug, feel free to either make an issue, or just message me about it.

# Roadmap
See the roadmap [here.](TODO.md)

# Useful keybinds
All of these features are accessible by clicking buttons, but here are some keybinds:
1. Ctrl + q to quit
2. Ctrl + tab to scroll list

# How to build for flatpak
This is more for me, but this is how I have gotten it to build for flatpak.

```
flatpak run org.flatpak.Builder --repo <some dir> <build dir, another one than repo> flatpak_builder.yml --force-clean

flatpak build-bundle <repo dir> <export name> io.github.joshuardecker.stig-view
```
