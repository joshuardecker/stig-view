# Stig View
![](https://github.com/joshuardecker/stig-view/blob/road-to-0.1/showcase.gif)

# How to run the application
Latest releases will contain at least the binary to run on Windows and Macos. A flatpak will be provided for Linux distribution. 

If you are feeling adventurous, or need to build from source, it is relatively straight forward. Install rust and its dependencies, and run the command ```cargo build --release``` Run this command in the base directory of the project, and it will produce a binary in /target/release.

The strongest use case of this software is opening a directory that has been unpacked by bedit. I personally just select the entire Xylok folder, but if many directories are unpacked, consider being more specific with your folder choice. Xml versions of the stig from DISA are not yet supported.

# Experimental Software
This software is not fully complete. Some features are not implemented yet, and bugs may be found. If you have suggestions, or find a bug, feel free to either make an issue, or just message me about it.

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
flatpak run org.flatpak.Builder --repo <some dir> <build dir, another one than repo> io.github.joshuardecker.stig-view.yml --force-clean

flatpak build-bundle <repo dir> <export name> io.github.joshuardecker.stig-view
```
