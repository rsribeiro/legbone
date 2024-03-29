legbone
===============

legbone is an experimental server for early versions of the game Tibia.

The objective of this project is not to create a polished and fully featured playable server for these versions. Instead, legbone is intended as a documentation of the peculiarities of the communication protocol of the early versions of the game (from 1.0 to 6.x). So, it is not really a game, more of a sandbox with lots of hardcoded values in which clients can join with any user name and password, but cannot see or communicate with each other.

Some parts of this project were heavily based on other projects, such as [OpenTibia](https://sourceforge.net/projects/opentibia/) (more specifically v0.1.0) and [TOSSERVER](https://sourceforge.net/projects/tosserver/).

legbone current works with versions 3.0 up to 6.x and has initial support for version 1.03. For now the differences between the protocols and its peculiarities are described only as code.

### Server

```
USAGE:
    legbone.exe [OPTIONS]

OPTIONS:
    -h, --help       Print help information
    -v, --verbose    Verbosity level (-v or -vv)
    -V, --version    Print version information
```

### Client

Older versions of the game can be found throughout the web. They can be run on modern computers using [winevdm](https://github.com/otya128/winevdm) or virtual machines.

### Debug Commands

Some debug commands can be sent with the in-game chat system.

`\d <command> <arguments>`

* chars: prints the different characters recognized by the chat system
* char arg: prints an specific character
* echo: echo message
* item arg1 arg2: gives item on slot
* i arg: gives item on right hand slot
* stats: sends stats message to client
* skills: sends skills message to client
* me arg: creates magic effect
* wlight arg: changes world light level
* plight arg: changes player light level
* userlist: requests user list
* userinfo arg: requests info on specific user
* info arg: sends info message to client
* error arg: sends error message to client
* panic arg: causes server panic
* chat: cycles between different chat types
* outfit arg: changes character outfit
