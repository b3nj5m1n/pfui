# mpd

Module for the [Music Player Daemon](https://wiki.archlinux.org/title/Music_Player_Daemon).

## Recognised events

See [mpd's documentation](https://mpd.readthedocs.io/en/stable/protocol.html#querying-mpd-s-status) for more details.

|  Event  |                  Description                   |
|---------|------------------------------------------------|
| Player  | The player has been started, stopped or seeked |
| Mixer   | The volume has been changed                    |
| Options | Repeat, random, etc.                           |

## JSON structure

```json
{
    "ok": "Was there some kind of problem updating the data?",
    "data": {
        "song": {
            "file_path": "👻 Path to the audio file being played",
            "title": "👻 The title of the current song",
            "album": "👻 The name of the album of the current song",
            "artist": "👻 The name of the artist of the current song",
            "date": "👻 The date on which the song was released",
            "genre": "👻 The genre of the current song",
        }
        "state": {
            "elapsed": "👻 How many seconds of the song have been played so far",
            "duration": "👻 How long the song is in total in seconds",
            "progress": "👻 How far along the current song is in percent, rounded",
            "status": "Is mpd currently playing, paused, or stopped? 0 = playing, 1 = paused, 2 = stopped"
        },
        "options": {
            "volume": "The current volume that mpd is set to (percentage)",
            "repeat": "Is mpd going to repeat this song?",
            "repeat": "Is mpd going to play a random song next?"
        }
    }
}
```

_👻 this field might be null_
