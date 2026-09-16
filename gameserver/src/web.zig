//! Local HTTP endpoint the dispatch server talks to.
//!
//! Dispatch forwards SRTools uploads to /reload here. The game server validates
//! and saves them before publishing a new config generation to game sessions.

const std = @import("std");
const httpz = @import("httpz");
const ConfigManager = @import("manager/config_mgr.zig");

const Allocator = std.mem.Allocator;
const log = std.log.scoped(.web);

pub const PORT: u16 = 21001;

pub fn start(allocator: Allocator) !std.Thread {
    return std.Thread.spawn(.{}, run, .{allocator});
}

fn run(allocator: Allocator) void {
    var srv = httpz.Server(void).init(allocator, .{
        .address = "127.0.0.1",
        .port = PORT,
        .request = .{ .max_body_size = 16 * 1024 * 1024 },
    }, {}) catch |err| {
        log.err("could not start the local endpoint on port {d}: {}", .{ PORT, err });
        return;
    };

    defer srv.deinit();

    var router = srv.router(.{}) catch |err| {
        log.err("could not build the router: {}", .{err});
        return;
    };
    router.post("/reload", onReload, .{});
    router.get("/sr-tools-export", onExport, .{});

    log.info("local endpoint is listening at localhost:{d}", .{PORT});
    srv.listen() catch |err| {
        log.err("local endpoint stopped: {}", .{err});
    };
}

/// An uploaded body replaces the config; an empty POST reloads the local file.
fn onReload(req: *httpz.Request, res: *httpz.Response) !void {
    ConfigManager.config_lock.lock();
    defer ConfigManager.config_lock.unlock();

    const body = req.body() orelse "";
    const result = if (body.len == 0)
        ConfigManager.UpdateGameConfig()
    else
        ConfigManager.applyGameConfig(body);
    result catch |err| {
        log.err("reload failed; previous config retained: {}", .{err});
        res.status = 500;
        res.body = try std.fmt.allocPrint(res.arena, "reload failed: {s}", .{@errorName(err)});
        return;
    };

    log.info("game config reloaded", .{});
    res.status = 200;
    res.body = "OK";
}

/// Exports the current freesr-data.json used by the game server.
fn onExport(_: *httpz.Request, res: *httpz.Response) !void {
    ConfigManager.config_lock.lock();
    defer ConfigManager.config_lock.unlock();
    res.status = 200;
    res.content_type = .JSON;
    res.header("Content-Disposition", "attachment; filename=freesr-data.json");
    res.body = try res.arena.dupe(u8, ConfigManager.global_game_config_cache.game_config_json);
}
