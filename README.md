# remdes

An extremely lightweight remote desktop solution focused on speed, simplicity, and low-latency screen sharing.

## Usage
Client
```cmd
Usage: client [OPTIONS]

Options:
      --rt <RT>    Remote TCP address [default: 127.0.0.1:54277]
      --lu <LU>    Local UDP address [default: 127.0.0.1:49152]
      --ru <RU>    Remote UDP address [default: 127.0.0.1:54287]
  -f, --fps <FPS>  Specify the FPS [default: 120]
  -h, --help       Print help
```
Server
```cmd
Usage: server [OPTIONS] --window <WINDOW>

Options:
  -w, --window <WINDOW>  Target window whose title contains the given substring
      --lt <LT>          Local TCP address [default: 127.0.0.1:54277]
      --lu <LU>          Local UDP address [default: 127.0.0.1:54287]
  -t, --tps <TPS>        Server ticks/sec [default: 128]
  -h, --help             Print help
```

## Compatibility
- Client is cross-platform.
- Server is Windows-only.

## Todo
- [x] Server-to-Client video.
  - [x] UDP implementation.
  - [x] compressed ([lz4](https://crates.io/crates/lz4)) chunks.
  - [ ] regional (dirty) tiling.
- [ ] Server-to-Client audio.
  - [ ] UDP implementation.
  - [ ] [Opus](https://crates.io/crates/opus)?
- [ ] Client-to-Server input.
  - [ ] UDP implementation.
- [ ] Client/Server window resizing.
  - [ ] Modify client vertex shader?
- [ ] Screen-capturing for Unix.
- [ ] Improve atomic ordering.
- [ ] Complete documentation.
- [ ] Migrate to SDL3?