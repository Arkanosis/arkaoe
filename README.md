# arkaoe [![](https://img.shields.io/crates/v/arkaoe.svg)](https://crates.io/crates/arkaoe) [![License](https://img.shields.io/badge/license-ISC-blue.svg)](/LICENSE)

**arkaoe** is a web server providing tools for the [Age of Empires II](https://www.ageofempires.com/games/aoeiide/) real-time strategy video-game.

## Usage

```
Usage: arkaoe serve [--hostname=<hostname>] [--port=<port>]
       arkaoe -h | --help
       arkaoe --version

Commands:
    serve                    Start a small HTTP server to serve the tools.

Options:
    -h, --help               Show this screen.
    --hostname=<hostname>    Hostname to resolve to find the network interface to serve the tools [default: localhost].
    --port=<port>            Port on which to serve the tools [default: 8080].
    --version                Show version.
```

## Compiling

Run `cargo build --release` in your working copy.

## Contributing and reporting bugs

Contributions are welcome through [GitHub pull requests](https://github.com/Arkanosis/arkaoe/pulls).

Please report bugs and feature requests on [GitHub issues](https://github.com/Arkanosis/arkaoe/issues).

## License

arkaoe is copyright (C) 2024 Jérémie Roquet <jroquet@arkanosis.net> and licensed under the ISC license.
