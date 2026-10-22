# CLI-Documentation

This document contains all available actions and commands for the CLI.

## Sleeping

With these commands you can set the camera to sleep or wake it up.

```shell
t4l turn on
```

| Action | Description               | Command | Aliases    |
| ------ | ------------------------- | ------- | ---------- |
| Sleep  | Sets the camera to sleep. | `sleep` | `turn off` |
| Wake   | Wakes the camera up.      | `wake`  | `turn on`  |

## Tracking Mode

With these commands you can control the AI-tracking of the camera.
You can either turn it completely off or set it to a specific mode.

All tracking commands begin with `tracking` or `track`, e.g.:

```shell
t4l tracking normal
```

| Action      | Description                                                                 | Command      | Aliases          |
| ----------- | --------------------------------------------------------------------------- | ------------ | ---------------- |
| No Tracking | Turns off the AI-tracking and keeps the camera still                        | `static`     | `none`, `off`    |
| Standard    | Turns on the standard AI-tracking, keeping the upper body and face in focus | `normal`     | `standard`, `on` |
| Closeup     | Similar to standard tracking, zoomed closer                                 | `close-up`   | `close`          |
| Upper Body  | Tracks the upper body, similar to standard tracking                         | `upper-body` |                  |
| Headless    | Tracks the upper body without the head                                      | `headless`   |                  |
| Lower Body  | Tracks the lower body                                                       | `lower-body` |                  |
| Desk        | Points the camera at the desk                                               | `desk`       |                  |
| Whiteboard  | Points the camera at a whiteboard                                           | `whiteboard` |                  |
| Hand        | Tracks a hand motion                                                        | `hand`       | `point`          |
| Group       | Tracks a group of people                                                    | `group`      |                  |

## Tracking Speed

With these commands you can control the speed that the camera is tracking with.
You can only use the presets ( `standard` and `sport`).

All tracking-speed commands begin with `speed` or `tracking-speed`, e.g.:

```shell
t4l speed standard
```

| Action   | Description                                          | Command    | Aliases                            |
| -------- | ---------------------------------------------------- | ---------- | ---------------------------------- |
| Standard | Sets the speed to the default, slower tracking speed | `standard` | `normal`, `default`, `slow`, `low` |
| Sport    | Sets the speed to the faster tracking speed          | `fast`     | `sport`, `high`                    |

## Presets

With these commands you can set the position of the camera to one of the 3 possible presets.

The command is `preset` or `position`, e.g.:

```shell
t4l preset 2
```

WARNING: Only the numbers 1 to 3 are valid.

You can set the presets with OBSBOT Center, e.g., running in a virtual machine.

## HDR

You can turn HDR on or off with the command `hdr`.

```shell
t4l hdr on
t4l hdr off
```

## Exposure Mode

With these commands you can control the exposure mode of the camera.

All exposure commands begin with `exposure`, e.g.:

```shell
t4l exposure manual
```

| Action | Description                                       | Command  |
| ------ | ------------------------------------------------- | -------- |
| Manual | Uses custom settings as exposure                  | `manual` |
| Global | Uses global reference for setting the exposure    | `global` |
| Face   | Uses a face as reference for setting the exposure | `face`   |

## Info

You can display available information about the current state of the camera with the command `info`.

```shell
t4l info
```

## Verbose

With the flag `--verbose` you can enable verbose logging.
You will see the communication with the camera in the console.

## Auto-Completion

If you use the cli more often, it might be useful to enable auto-completion for the commands.
Therefore, you can use the command `completions` to generate the shell-completion scripts for your shell.

In the following example, the shell-completion scripts are generated for bash with `bash-completion`:

```shell
t4l completions bash > /.local/share/bash-completion/completions/t4l
```

## Pan, tilt, zoom and local presets

With no value, `pan`, `tilt` and `zoom` print the current V4L2 value and range.
With a value, they set that control. Pan and tilt use arcseconds (3600 per degree)
and disable AI tracking. Use the range reported by your camera for zoom.

```shell
t4l pan
t4l tilt
t4l zoom
t4l pan 0
t4l tilt 7200
t4l preset-save Desk
t4l preset-list
t4l preset-recall Desk
t4l preset-delete Desk
```

Local presets are separate from the three presets stored in OBSBOT Center. They live
in `$XDG_CONFIG_HOME/tiny4linux/custom_presets.json`, falling back to
`~/.config/tiny4linux/custom_presets.json`. V4L2 pan/tilt readback can be cached, so
save a local preset after positioning the camera with these controls.
