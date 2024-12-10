# `last-watched`

Easily keep track of what episodes you've watched from ~~pirated TV shows~~ your collection of ripped DVDs.
This system is implemented as a Windows shell (explorer) extension that displays overlays on watched video file icons and uses scripts loaded by MPV to mark a video as watched.

![GrF7AYYsFf](https://github.com/user-attachments/assets/85217e19-cfd2-4af8-b2c6-b5e3b9859394)

## Installation

Download and extract the zip from the [latest release](https://github.com/connorslade/last-watched/releases).
Put it in a permanent spot because once the shell extension is registered, it will check that directory for the needed dll, exe, and ico files.
Now to register the extension, open an administrator command prompt in that folder and run `regsvr32 last_watched.dll`, if you ever want to remove the extension in the future, instead run `regsvr32 /u last_watched.dll`.
To get to see the changed take effect, try restarting Windows Explorer with Task Manager or just restart your system.

Depending on what media player you use the plugin installation will differ, all instructions can be found [here](plugins).
Currently only MPV is supported.

## How it Works

When you play a video file, the plugin for your video player will add the video's file name to a hidden `.watched` file in the same directory, creating it if it doesn't exist.
Then, when the shell comes across a video file (mp4, mkv, avi, webm, flv, mov, wmv) it will invoke a method provided by `last_watched.dll` to check for the watched sidecar file and add the icon if needed.
