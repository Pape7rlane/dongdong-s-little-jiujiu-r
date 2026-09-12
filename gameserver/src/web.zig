//! Local HTTP endpoint the dispatch server talks to.
//!
//! SRTools itself uploads to dispatch on port 21000 (see
//! `dispatch/src/srtools.zig`), which writes `freesr-data.json` and then calls
//! `/reload` here so the live config matches the file.

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
    var srv = httpz.Server(void).init(allocator, .{ .port = PORT }, {}) catch |err| {
        log.err("could not start the local endpoint on port {d}: {}", .{ PORT, err });
        return;
    };

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

/// Re-reads the game config from freesr-data.json.
fn onReload(_: *httpz.Request, res: *httpz.Response) !void {
    ConfigManager.config_lock.lock();
    defer ConfigManager.config_lock.unlock();

    ConfigManager.UpdateGameConfig() catch |err| {
        log.err("reload failed: {}", .{err});
        res.status = 500;
        res.body = "reload failed";
        return;
    };

    log.info("game config reloaded", .{});
    res.status = 200;
    res.body = "OK";
}

/// Exports the current freesr-data.json used by the game server.
fn onExport(_: *httpz.Request, res: *httpz.Response) !void {
    res.status = 200;
    res.content_type = .JSON;
    res.header("Content-Disposition", "attachment; filename=freesr-data.json");
    res.body = std.fs.cwd().readFileAlloc(res.arena, "freesr-data.json", std.math.maxInt(usize)) catch {
        res.status = 404;
        res.content_type = .TEXT;
        res.body = "freesr-data.json not found";
        return;
    };
}

