<h1 align='center'><b>Sprite Packer</b></h1>
<p align='center'>
    <a href="https://github.com/jngo102/sprite-packer/actions/workflows/main.yml">
        <img src="https://img.shields.io/github/actions/workflow/status/jngo102/sprite-packer/main.yml?branch=main" 
             alt="sprite-packer build status">
    </a>
    <a href="https://github.com/jngo102/sprite-packer">
        <img src="https://img.shields.io/github/downloads/jngo102/sprite-packer/total" 
             alt="sprite-packer build status">
    </a>
    <a href="https://github.com/jngo102/sprite-packer/commits">
        <img src="https://img.shields.io/github/commit-activity/m/jngo102/sprite-packer"
             alt="sprite-packer commit frequency">
    </a>
    <a href="https://github.com/jngo102/sprite-packer/blob/main/LICENSE.md">
        <img src="https://img.shields.io/github/license/jngo102/sprite-packer"
             alt="sprite-packer software license">
    </a>
</p>
<p align='center'>
    <a href="https://discord.gg/VDsg3HmWuB">
        <img src="https://img.shields.io/discord/879125729936298015?logo=discord"
            alt="Visit the Hollow Knight Modding Discord server">
    </a>
    <a href="https://twitter.com/intent/follow?screen_name=JngoCreates">
        <img src="https://img.shields.io/twitter/follow/JngoCreates?style=social&logo=twitter"
             alt="Follow JngoCreates on Twitter">
    </a>
</p>

![sprite-packer screenshot](/images/window.png)

This is a re-implementation of [HollowKnight.SpritePacker](https://github.com/magegihk/HollowKnight.SpritePacker), originally created by [magegihk](https://github.com/magegihk), using the [Tauri](https://tauri.app) framework with a [ReactJS](https://reactjs.org/) frontend. It mainly serves as a playground for me to learn React.

## **Installation**
### Windows
1. Navigate to the [Releases](https://github.com/jngo102/sprite-packer/releases) page.
2. Download the latest release for your platform. This will be `sprite-packer_{version}_x64_en-US.msi`.
3. Open up the location of the download in your file system and double click on the file that you downloaded.
4. Follow the steps in the wizard to install Sprite Packer. Most steps can be left with their default values.

### macOS
1. Navigate to the [Releases](https://github.com/jngo102/sprite-packer/releases) page.
2. Download the latest release for your platform. This will be `sprite-packer_{version}_x64.dmg`.
3. Open up the location of the download in your file system and double click on the file that you downloaded.
4. Follow the steps in the wizard to install Sprite Packer.

### Linux
1. Navigate to the [Releases](https://github.com/jngo102/sprite-packer/releases) page.
2. The file that you will download varies depending on which package manager you use. Refer to your distribution's manual for information on how to install packages for your system. For Ubuntu and Debian distributions, this is `sprite-packer_{version}_amd64.deb`.
3. Open up the location of the download in your file system.
4. Using your package manager, install the package that you downloaded.

## **Usage**
1.  When you first open the app, you will be asked to choose a folder location. This will be where your sprites are stored (*Note*: *Not* an animation folder containing the PNG files!). You can change this location at any time by clicking on `Options` at the top of the app and selecting `Set Sprites Path`.
2.  Before packing, you must check that each sprite and its duplicates are identical by clicking on the "Check" button at the bottom. Any sprites that are not identical will appear in the "Changed Sprites" list on the right. You can then click the sprite that you want to replace all duplicates with and then click on the "Replace Duplicates" button to replace them.
3.  After packing, a file dialog will open to ask where to save the generated atlas.

## **Issues**
If you encounter any issues, please report them on the [Issues](https://github.com/jngo102/sprite-packer/issues) page.