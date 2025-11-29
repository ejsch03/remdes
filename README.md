# remdes

An extremely lightweight remote desktop solution focused on speed, simplicity, and low-latency screen sharing.

## Usage
Client
```cmd
Usage: client [OPTIONS]

Options:
      --lu <LU>    Local UDP address [default: 127.0.0.1:49152]    
      --ru <RU>    Remote UDP IP address [default: 127.0.0.1:54287]
  -f, --fps <FPS>  Specify the FPS [default: 120]
  -h, --help       Print help
```
Server
```cmd
Usage: server [OPTIONS] --window <WINDOW>

Options:
  -w, --window <WINDOW>  Target window whose title contains the given substring
      --lu <LU>          Local UDP IP address [default: 127.0.0.1:54287]
  -t, --tps <TPS>        Server ticks/sec [default: 128]
  -h, --help             Print help
```

## Features
- [x] Any FPS (configurable with the `--fps` flag).
- [x] Near-instant latency (on LAN only, WiFi is rough at the moment).
- [ ] Client-to-Server Input.
- [ ] Server-to-Client Audio.
- [ ] Window resizing.

## Compatibility
- Client is cross-platform.
- Server is Windows-only.

## Todo
1) Implement client-server regional tiling.
2) Implement client-to-server user input.
3) Implement sharing server audio.
4) Improve/guarantee atomic ordering.
5) Improve overall documentation.
6) Fix client's windows resizing.
7) Upgrade to SDL3.