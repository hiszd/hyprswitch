# hyprswitch

**hyprswitch** is a "on monitor attach/detach" event listener for **hyprland** that executes commands listed in a config file.

## Features
 - Execute a command when a monitor is added, or removed
 - Configure with a simple JSON file

## Configuration

Here is an example configuration:
```json
[
  {
    "mons": "eDP-1,DP-2",
    "cmds": [
      "keyword monitor ${mons1},1920x1080,0x0,1.5"
      "keyword monitor ${mons2},highrr,0x0,1"
    ]
  },
  {
    "mons": "eDP-1",
    "cmds": [
      "keyword monitor ${mons1},highrr,0x0,1"
    ]
  }
]
```
This configuration is stored in XDG_CONFIG_HOME/hyprswitch/config.json
As of right now this config is not automatically created, so make sure you create this before launching hyprswitch.

How the config works is that each item in the array is essentially its own config.
### Monitors
The "mons" item is the name of the monitors seperated by a comma.
The name of the monitor is can be gotten from running `hyprctl monitors`.

### Commands
The "cmds" item is a list of commands to run when the configuration is applied.
These commands are executed in order.
You can write them just like you type them into your terminal.(Meaning they look in your $PATH)
You can also add in the "cmds" string a string like "${mons1}" which will substitute the first monitor in the list of monitors, or "${mons2}" for the second.
This makes it more composable and easier to copy and paste.
Any paths using tilde in them(e.g. "~/execme.sh") will redirect to the current users XDG_CONFIG_HOME directory.


## Usage

To use hyprswitch all you need to do is download the release bin(or build from source) and either put it somewhere in your $PATH,
or use an absolute path in your hyprland.conf
