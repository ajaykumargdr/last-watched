# Plugins

Some media players support plugins and can be extended to automatically mark video files as watched.
Currently the only implementation is for [mpv](https://mpv.io), [`last-watched.lua`](last-watched.lua).

To install just go to your mpv config directory (`%APPDATA%/mpv`) create a `scripts` directory if one dose not already exist and copy in the lua script.
