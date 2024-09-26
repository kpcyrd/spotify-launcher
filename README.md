# spotify-launcher

Spotify has a free linux client but prohibits re-distribution, so this is a freely distributable opensource program that manages a spotify installation in your home folder from the official spotify release server.

If you work for spotify please reach out so we can talk. 👉👈

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
[launcher]
## Skip update (override check_update option)
#skip_update = true
## Force update (override check_update = true and skip_update = false)
#force_update = true
## Check update
#check_update = true
```

## License

GPLv3+
