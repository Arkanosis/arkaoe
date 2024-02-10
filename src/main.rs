use std::process;

use serde_derive::Deserialize;

const USAGE: &str = "
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
";

#[derive(Deserialize)]
struct Args {
    cmd_serve: bool,
    flag_hostname: String,
    flag_port: u16,
    flag_version: bool,
}

fn main() {
    let args: Args =
        docopt::Docopt::new(USAGE)
            .and_then(|docopts|
                docopts.argv(std::env::args().into_iter())
                   .deserialize()
            )
            .unwrap_or_else(|error|
                error.exit()
            );

    if args.flag_version {
        println!("arkaoe v{}", arkaoe::version());
    } else {
        if args.cmd_serve {
            if arkaoe::serve(args.flag_hostname, args.flag_port).is_err() {
                process::exit(1);
            }
        }
    }
}
