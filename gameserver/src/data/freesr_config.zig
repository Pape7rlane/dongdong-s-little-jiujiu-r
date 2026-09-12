const std = @import("std");
const GameConfigMod = @import("game_config.zig");

const Allocator = std.mem.Allocator;
const ArrayList = std.ArrayList;
const GameConfig = GameConfigMod.GameConfig;
const BattleConfig = GameConfigMod.BattleConfig;
const Avatar = GameConfigMod.Avatar;
const Lightcone = GameConfigMod.Lightcone;
const Relic = GameConfigMod.Relic;

pub const FILENAME = "freesr-data.json";
const SP_SCALE = 100;

fn getInt(obj: std.json.Value, key: []const u8, fallback: u32) u32 {
    if (obj != .object) return fallback;
    const value = obj.object.get(key) orelse return fallback;
    return switch (value) {
        .integer => |i| if (i < 0) fallback else @intCast(i),
        .float => |f| if (f < 0) fallback else @intFromFloat(f),
        else => fallback,
    };
}

fn getArray(obj: std.json.Value, key: []const u8) ?[]std.json.Value {
    if (obj != .object) return null;
    const value = obj.object.get(key) orelse return null;
    return if (value == .array) value.array.items else null;
}

fn getObject(obj: std.json.Value, key: []const u8) ?std.json.Value {
    if (obj != .object) return null;
    const value = obj.object.get(key) orelse return null;
    return if (value == .object) value else null;
}

fn resolveAvatarId(key: []const u8, value: std.json.Value) ?u32 {
    const inner = getInt(value, "avatar_id", 0);
    if (inner != 0) return inner;
    return std.fmt.parseInt(u32, key, 10) catch null;
}

fn parseRelic(entry: std.json.Value) Relic {
    var relic = Relic{ .id = getInt(entry, "relic_id", 0), .level = getInt(entry, "level", 0), .main_affix_id = getInt(entry, "main_affix_id", 0), .sub_count = 0,
        .stat1 = 0, .cnt1 = 0, .step1 = 0, .stat2 = 0, .cnt2 = 0, .step2 = 0, .stat3 = 0, .cnt3 = 0, .step3 = 0, .stat4 = 0, .cnt4 = 0, .step4 = 0 };
    const subs = getArray(entry, "sub_affixes") orelse return relic;
    relic.sub_count = @intCast(@min(subs.len, 4));
    for (subs, 0..) |sub, i| {
        const id = getInt(sub, "sub_affix_id", 0);
        const count = getInt(sub, "count", 0);
        const step = getInt(sub, "step", 0);
        switch (i) {
            0 => { relic.stat1 = id; relic.cnt1 = count; relic.step1 = step; },
            1 => { relic.stat2 = id; relic.cnt2 = count; relic.step2 = step; },
            2 => { relic.stat3 = id; relic.cnt3 = count; relic.step3 = step; },
            3 => { relic.stat4 = id; relic.cnt4 = count; relic.step4 = step; },
            else => break,
        }
    }
    return relic;
}

fn parseBattle(root: std.json.Value, allocator: Allocator) !BattleConfig {
    var battle = BattleConfig{ .battle_id = 1, .stage_id = 0, .cycle_count = 30, .monster_wave = ArrayList(ArrayList(u32)).init(allocator), .monster_level = 80, .blessings = ArrayList(u32).init(allocator) };
    errdefer { for (battle.monster_wave.items) |*wave| wave.deinit(); battle.monster_wave.deinit(); battle.blessings.deinit(); }
    const cfg = getObject(root, "battle_config") orelse return battle;
    battle.stage_id = getInt(cfg, "stage_id", 0);
    battle.cycle_count = getInt(cfg, "cycle_count", 30);
    var level_found = false;
    if (getArray(cfg, "monsters")) |waves| for (waves) |wave_value| {
        if (wave_value != .array) continue;
        var wave = ArrayList(u32).init(allocator);
        errdefer wave.deinit();
        for (wave_value.array.items) |entry| {
            const monster_id = getInt(entry, "monster_id", 0);
            if (monster_id == 0) continue;
            if (!level_found) { const level = getInt(entry, "level", 0); if (level != 0) { battle.monster_level = level; level_found = true; } }
            var n: u32 = 0;
            while (n < @max(getInt(entry, "amount", 1), 1)) : (n += 1) try wave.append(monster_id);
        }
        try battle.monster_wave.append(wave);
    };
    if (getArray(cfg, "blessings")) |blessings| for (blessings) |blessing| {
        const id = getInt(blessing, "id", 0); if (id != 0) try battle.blessings.append(id);
    };
    const resonance = getInt(cfg, "path_resonance_id", 0);
    if (resonance != 0) try battle.blessings.append(resonance);
    return battle;
}

pub fn parseConfig(root: std.json.Value, allocator: Allocator) !GameConfig {
    var battle = try parseBattle(root, allocator);
    errdefer { for (battle.monster_wave.items) |*wave| wave.deinit(); battle.monster_wave.deinit(); battle.blessings.deinit(); }
    var avatars = ArrayList(Avatar).init(allocator);
    errdefer { for (avatars.items) |*avatar| avatar.relics.deinit(); avatars.deinit(); }
    var indexes = std.AutoHashMap(u32, usize).init(allocator);
    defer indexes.deinit();
    if (getObject(root, "avatars")) |entries| {
        var it = entries.object.iterator();
        while (it.next()) |entry| {
            const id = resolveAvatarId(entry.key_ptr.*, entry.value_ptr.*) orelse continue;
            const data = getObject(entry.value_ptr.*, "data");
            const techniques = getArray(entry.value_ptr.*, "techniques");
            try indexes.put(id, avatars.items.len);
            try avatars.append(.{ .id = id, .hp = 100, .sp = getInt(entry.value_ptr.*, "sp_value", 5000) / SP_SCALE, .level = getInt(entry.value_ptr.*, "level", 80), .promotion = getInt(entry.value_ptr.*, "promotion", 6), .rank = if (data) |d| getInt(d, "rank", 0) else 0, .lightcone = .{ .id = 0, .rank = 1, .level = 80, .promotion = 6 }, .relics = ArrayList(Relic).init(allocator), .use_technique = techniques != null and techniques.?.len != 0 });
        }
    }
    if (getArray(root, "lightcones")) |items| for (items) |entry| {
        const idx = indexes.get(getInt(entry, "equip_avatar", 0)) orelse continue;
        avatars.items[idx].lightcone = .{ .id = getInt(entry, "item_id", 0), .rank = @max(getInt(entry, "rank", 1), 1), .level = getInt(entry, "level", 80), .promotion = getInt(entry, "promotion", 6) };
    };
    if (getArray(root, "relics")) |items| for (items) |entry| {
        const idx = indexes.get(getInt(entry, "equip_avatar", 0)) orelse continue;
        const relic = parseRelic(entry); if (relic.id != 0) try avatars.items[idx].relics.append(relic);
    };
    return .{ .battle_config = battle, .avatar_config = avatars };
}
