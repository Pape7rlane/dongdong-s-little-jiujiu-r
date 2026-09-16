const std = @import("std");
const protocol = @import("protocol");
const handlers = @import("handlers.zig");
const Packet = @import("Packet.zig");
const ConfigManager = @import("../src/manager/config_mgr.zig");

const Allocator = std.mem.Allocator;
const Stream = std.net.Stream;
const Address = std.net.Address;

const Self = @This();
const log = std.log.scoped(.session);

address: Address,
stream: Stream,
allocator: Allocator,
main_allocator: Allocator,
game_config_cache: *ConfigManager.GameConfigCache,
config_generation: u64 = 0,
ready_for_sync: bool = false,
known_equipment: std.ArrayList(u32),
known_relics: std.ArrayList(u32),

pub fn init(
    address: Address,
    stream: Stream,
    session_allocator: Allocator,
    main_allocator: Allocator,
    game_config_cache: *ConfigManager.GameConfigCache,
) Self {
    return .{
        .address = address,
        .stream = stream,
        .allocator = session_allocator,
        .main_allocator = main_allocator,
        .game_config_cache = game_config_cache,
        .known_equipment = std.ArrayList(u32).init(session_allocator),
        .known_relics = std.ArrayList(u32).init(session_allocator),
    };
}

pub fn run(self: *Self) !void {
    defer self.stream.close();
    defer self.known_equipment.deinit();
    defer self.known_relics.deinit();

    var reader = self.stream.reader();
    while (true) {
        var packet = Packet.read(&reader, self.allocator) catch break;
        defer packet.deinit();
        try handlers.handle(self, &packet);
    }
}

/// Caller holds ConfigManager.config_lock.
pub fn rememberInventory(self: *Self) !void {
    const config = &ConfigManager.global_game_config_cache.game_config;
    try self.known_equipment.ensureTotalCapacity(config.lightcones.items.len);
    try self.known_relics.ensureTotalCapacity(config.relics.items.len);
    self.known_equipment.clearRetainingCapacity();
    self.known_relics.clearRetainingCapacity();
    for (config.lightcones.items) |item| self.known_equipment.appendAssumeCapacity(item.unique_id);
    for (config.relics.items) |item| self.known_relics.appendAssumeCapacity(item.unique_id);
}

pub fn send(self: *Self, cmd_id: protocol.CmdID, proto: anytype) !void {
    const data = try proto.encode(self.allocator);
    defer self.allocator.free(data);

    const packet = try Packet.encode(@intFromEnum(cmd_id), &.{}, data, self.allocator);
    defer self.allocator.free(packet);

    try self.stream.writeAll(packet);
}

pub fn send_empty(self: *Self, cmd_id: protocol.CmdID) !void {
    const packet = try Packet.encode(@intFromEnum(cmd_id), &.{}, &.{}, self.allocator);
    defer self.allocator.free(packet);

    try self.stream.writeAll(packet);
    log.debug("sent EMPTY packet with id {}", .{cmd_id});
}
