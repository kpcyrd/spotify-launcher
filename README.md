# spotify-launcher

![Top Language](https://img.shields.io/github/languages/top/kpcyrd/spotify-launcher?logo=rust)
![GitHub stars](https://img.shields.io/github/stars/kpcyrd/spotify-launcher)

spotify-launcher **adapts the Spotify installation** from the official apt repository to **Arch Linux**.

If you work for spotify please reach out so we can talk. 👉👈

## Usage
First clone the repository and install through PKGBUILD:
```bash
git clone https://github.com/kpcyrd/spotify-launcher
cd spotify-launcher
makepkg -si
```
Then, to use the launcher you can just type:
```bash
spotify-launcher
```
You can also use the following flags:
```bash
--keyring <PATH>          # Override the default keyring path
--deb <PATH>              # Use a local .deb file
--install-dir <PATH>      # Custom installation directory
-v, --verbose             # Verbose logging
--check-update            # Check if there's any available update
--skip-update             # Never check for updates
--force-update            # Force an update even if the latest version is installed
--print-deb-url           # Print the URL of the .deb file
--no-exec                 # Install or update but don't launch
--timeout <SECONDS>       # HTTP connection timeout
--download-attempts <NUM> # Download retry attempts (0 for unlimited)
```

## Configuration

spotify-launcher is going to look for a configuration file at the following locations (in this order):

- `${XDG_CONFIG_HOME:-$HOME/.config}/spotify-launcher.conf`
- `/etc/spotify-launcher.conf`

If no config is found it's going to start with default settings. Your configuration file may look like this:

```toml
[spotify]
## Pass extra arguments to the spotify executable
## You can test this with `spotify-launcher -v --skip-update --no-exec`
#extra_arguments = []
## On HiDPI displays spotify doesn't pick up the scale factor automatically
#extra_arguments = ["--force-device-scale-factor=2.0"]
## On wayland you might need
#extra_arguments = ["--enable-features=UseOzonePlatform", "--ozone-platform=wayland"]
```

## License

`Apache-2.0 OR MIT`
