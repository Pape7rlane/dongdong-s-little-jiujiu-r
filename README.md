# PearlSR 
#### Honkai: Star Rail server emulator (4.6.51) written in Zig.

![Screenshot](Screenshot.png)

## Requirements
- [Zig 0.14.1 x64](https://ziglang.org/download/0.14.1/zig-x86_64-windows-0.14.1.zip)

## Configuration and SRTools

`freesr-data.json` is the only game and battle configuration source. The legacy `config.json` file is not used.

To sync with [SRTools](https://srtools.neonteam.dev/): start both dispatch and gameserver, choose the `RobinSR` provider, and set the server URL to `http://localhost:21000`. SRTools uploads to `POST /srtools`; dispatch forwards the payload to gameserver, which validates it and atomically saves `freesr-data.json` in its own working directory. A failed sync returns an HTTP error and keeps the previous active configuration.

Online players receive updated avatars, skill levels and inventory on their next game request (including the automatic heartbeat). No manual `/sync` or relog is required. Removed inventory items are also removed on the client. Start a new battle after syncing to apply battle-stat changes consistently.

Inventory updates are sent before a separate avatar equipment update, so re-equipping a relic or lightcone resolves against the refreshed inventory. Item IDs remain stable when equipping, unequipping, or transferring an item between avatars.

Use `GET /srtools-export` to download the active configuration while gameserver is running. After editing the file manually, use `/sync` in game to reload it. Both services must be running for website uploads.

## Running

### From source

Windows:
```
git clone https://git.xeondev.com/HonkaiSlopRail/pearl-sr
cd pearl-sr
zig build run-dispatch
zig build run-gameserver
```

Linux with tmux (上下分屏同时查看两个服务)：
```bash
chmod +x start.sh
./start.sh
```

`start.sh` 会在缺少 `tmux` 时尝试通过系统包管理器安装，并在同一个会话中启动
dispatch 和 gameserver。再次执行脚本会重新连接已有的 `pearlsr` 会话。

### Setup launcher.exe

Copy `launcher.exe` and `hkprg.dll` from launcher folder inside pearl-sr and paste them inside your client folder.
Then open your `launcher.exe` with administrator.

### Using Pre-built Binaries
Navigate to the [Releases](https://git.xeondev.com/HonkaiSlopRail/pearl-sr/releases)
page and download the latest release for your platform.

## Connecting
Get 4.5.5X client: [Gofile](https://gofile.io/d/vtV5JfkO)

Get 4.6.5X client: [Gofile](https://gofile.io/d/cZx9oACJ)

## Functionality (work in progress)
- Login and player spawn
- Test battle via calyx
- MoC/PF/AS simulator with custom stage sellection
- Anomaly Arbitration (Challenge Peak)
- Starward Mode (Challenge Tierce)
- Gacha simulator 
- Support command for Sillyism

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss
what you would like to change, and why.

## Bug Reports

If you find a bug, please open an issue with as much detail as possible. If you
can, please include steps to reproduce the bug.

Bad issues such as "This doesn't work" will be closed immediately, be _sure_ to
provide exact detailed steps to reproduce your bug. If it's hard to reproduce, try
to explain it and write a reproducer as best as you can.
