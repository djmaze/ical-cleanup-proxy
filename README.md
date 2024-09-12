# iCal-Cleanup-Proxy

This is an http proxy server which cleans up iCalendar (`.ics`) files given via URL and returns a stripped-down version with only the latest events left.

*TODO Add more description*

## Building / Running

This project is built using Rust. You need to have `rustc` and `cargo` installed.

Run it like this with default options:

```bash
cargo run --bin server
```

The server is now running on http://localhost:3077 (but it listens on all addresses).

### Using the server

In order to use it, you need to supply the url to the `.ics` file in the `url` query parameter. Example:

```bash
curl http://localhost:3077/?url=https://calendar.google.com/calendar/ical/.../basic.ics
```

This will return a shortened, cleaned-up version of the ics file content.

### Options

You can add `--` to add additional args. Try the following to see the available options:

```bash
cargo run --bin server -- --help
```

## Running via docker

There is also a docker image. You can use the prebuilt image directly:

```bash
docker run -it --rm -p 3077:3077 decentralize/ical-cleanup-proxy
```

### Building the docker image

You can also build the docker image locally yourself:

```bash
docker compose build
```

Afterwards, you run the image like this:

```bash
docker compose up app
```
