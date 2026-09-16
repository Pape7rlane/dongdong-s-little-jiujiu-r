const std = @import("std");
const protocol = @import("protocol");
const Session = @import("../Session.zig");
const AvatarManager = @import("../manager/avatar_mgr.zig");
const ConfigManager = @import("../manager/config_mgr.zig");
const commandhandler = @import("../command.zig");

/// Called on the session thread with config_lock held. Use the same inventory
/// IDs as login; remove deleted items instead of generating a second inventory.
pub fn syncCurrent(session: *Session, allocator: std.mem.Allocator) !void {
    const config = &ConfigManager.global_game_config_cache.game_config;
    var sync = protocol.PlayerSyncScNotify.init(allocator);
    defer sync.deinit();

    for (session.known_equipment.items) |id| {
        var found = false;
        for (config.lightcones.items) |item| {
            if (item.unique_id == id) {
                found = true;
                break;
            }
        }
        if (!found) try sync.del_equipment_list.append(id);
    }
    for (session.known_relics.items) |id| {
        var found = false;
        for (config.relics.items) |item| {
            if (item.unique_id == id) {
                found = true;
                break;
            }
        }
        if (!found) try sync.del_relic_list.append(id);
    }
    for (config.lightcones.items) |lc| {
        try sync.equipment_list.append(try AvatarManager.createEquipment(lc, lc.equip_avatar));
    }
    for (config.relics.items) |relic| {
        try sync.relic_list.append(try AvatarManager.createRelic(allocator, relic, relic.equip_avatar));
    }
    sync.avatar_sync = protocol.AvatarSync.init(allocator);
    try AvatarManager.appendAvatarData(allocator, &sync.avatar_sync.?.avatar_list, &sync.avatar_sync.?.avatar_path_data_info_list);
    try session.send(protocol.CmdID.CmdPlayerSyncScNotify, sync);
    try session.rememberInventory();
    session.config_generation = ConfigManager.config_generation;
}

pub fn onGenerateAndSync(session: *Session, _: []const u8, allocator: std.mem.Allocator) !void {
    ConfigManager.UpdateGameConfig() catch |err| {
        std.log.err("config reload failed: {}", .{err});
        try commandhandler.sendMessage(session, "Sync failed; the previous configuration is still active.", allocator);
        return;
    };
    try syncCurrent(session, allocator);
    try commandhandler.sendMessage(session, "Configuration synced.", allocator);
}
