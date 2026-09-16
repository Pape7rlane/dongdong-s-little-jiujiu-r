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
        .integer => |i| std.math.cast(u32, i) orelse fallback,
        .float => |f| if (!std.math.isFinite(f) or f < 0 or f > std.math.maxInt(u32)) fallback else @intFromFloat(f),
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

fn parseSkills(data: std.json.Value, key: []const u8, allocator: Allocator) !?[]GameConfigMod.Skill {
    const value = data.object.get(key) orelse return null;
    if (value != .object) return error.InvalidSkills;
    var skills = ArrayList(GameConfigMod.Skill).init(allocator);
    errdefer skills.deinit();
    var it = value.object.iterator();
    while (it.next()) |entry| {
        const id = try std.fmt.parseInt(u32, entry.key_ptr.*, 10);
        if (entry.value_ptr.* != .integer) return error.InvalidSkillLevel;
        const level = std.math.cast(u32, entry.value_ptr.integer) orelse return error.InvalidSkillLevel;
        try skills.append(.{ .id = id, .level = level });
    }
    return try skills.toOwnedSlice();
}

fn requiredInt(obj: std.json.Value, key: []const u8) !u32 {
    if (obj != .object) return error.InvalidEntry;
    const value = obj.object.get(key) orelse return error.MissingField;
    if (value != .integer) return error.InvalidNumber;
    return std.math.cast(u32, value.integer) orelse error.InvalidNumber;
}

/// Preserve SRTools IDs, allocating missing IDs above every explicit ID.
/// Equipment and relics have separate protocol ID namespaces.
fn nextItemId(entry: std.json.Value, used: *std.AutoHashMap(u32, void), next: *u32) !u32 {
    const id = if (entry.object.get("internal_uid") != null) try requiredInt(entry, "internal_uid") else 0;
    if (id != 0) {
        const result = try used.getOrPut(id);
        if (result.found_existing) return error.DuplicateItemId;
        return id;
    }
    next.* = std.math.add(u32, next.*, 1) catch return error.TooManyItems;
    try used.put(next.*, {});
    return next.*;
}

fn maxItemId(items: []std.json.Value) !u32 {
    var max_id: u32 = 0;
    for (items) |item| {
        if (item != .object) return error.InvalidItem;
        if (item.object.get("internal_uid") != null) {
            max_id = @max(max_id, try requiredInt(item, "internal_uid"));
        }
    }
    return max_id;
}

pub fn parseConfig(root: std.json.Value, allocator: Allocator) !GameConfig {
    const entries = getObject(root, "avatars") orelse return error.InvalidAvatars;
    const equipment_entries = getArray(root, "lightcones") orelse return error.InvalidLightcones;
    const relic_entries = getArray(root, "relics") orelse return error.InvalidRelics;
    var config = GameConfig{
        .battle_config = try parseBattle(root, allocator),
        .avatar_config = ArrayList(Avatar).init(allocator),
        .lightcones = ArrayList(Lightcone).init(allocator),
        .relics = ArrayList(Relic).init(allocator),
    };
    errdefer config.deinit();

    var indexes = std.AutoHashMap(u32, usize).init(allocator);
    defer indexes.deinit();
    var it = entries.object.iterator();
    while (it.next()) |entry| {
        const value = entry.value_ptr.*;
        const id = resolveAvatarId(entry.key_ptr.*, value) orelse return error.InvalidAvatarId;
        if (id == 0 or indexes.contains(id)) return error.InvalidAvatarId;
        const data = getObject(value, "data") orelse return error.InvalidAvatarData;
        const techniques = getArray(value, "techniques");
        var avatar = Avatar{
            .id = id,
            .hp = 100,
            .sp = getInt(value, "sp_value", 5000) / SP_SCALE,
            .level = try requiredInt(value, "level"),
            .promotion = try requiredInt(value, "promotion"),
            .rank = try requiredInt(data, "rank"),
            .lightcone = .{ .id = 0, .rank = 1, .level = 1, .promotion = 0 },
            .relics = ArrayList(Relic).init(allocator),
            .use_technique = techniques != null and techniques.?.len != 0,
        };
        errdefer avatar.deinit(allocator);
        avatar.skills = try parseSkills(data, "skills", allocator);
        avatar.skills_by_anchor_type = try parseSkills(data, "skills_by_anchor_type", allocator);
        try indexes.put(id, config.avatar_config.items.len);
        try config.avatar_config.append(avatar);
    }

    var used = std.AutoHashMap(u32, void).init(allocator);
    defer used.deinit();
    var next = try maxItemId(equipment_entries);
    for (equipment_entries) |entry| {
        const lc = Lightcone{
            .unique_id = try nextItemId(entry, &used, &next),
            .equip_avatar = try requiredInt(entry, "equip_avatar"),
            .id = try requiredInt(entry, "item_id"),
            .rank = try requiredInt(entry, "rank"),
            .level = try requiredInt(entry, "level"),
            .promotion = try requiredInt(entry, "promotion"),
        };
        if (lc.id == 0) return error.InvalidLightcone;
        if (lc.equip_avatar != 0) {
            const idx = indexes.get(lc.equip_avatar) orelse return error.UnknownEquipAvatar;
            if (config.avatar_config.items[idx].lightcone.id != 0) return error.DuplicateEquippedLightcone;
            config.avatar_config.items[idx].lightcone = lc;
        }
        try config.lightcones.append(lc);
    }

    used.clearRetainingCapacity();
    next = try maxItemId(relic_entries);
    for (relic_entries) |entry| {
        var relic = parseRelic(entry);
        relic.id = try requiredInt(entry, "relic_id");
        relic.level = try requiredInt(entry, "level");
        relic.main_affix_id = try requiredInt(entry, "main_affix_id");
        relic.unique_id = try nextItemId(entry, &used, &next);
        relic.equip_avatar = try requiredInt(entry, "equip_avatar");
        const slot = relic.id % 10;
        if (slot < 1 or slot > 6) return error.InvalidRelicSlot;
        if (relic.equip_avatar != 0) {
            const idx = indexes.get(relic.equip_avatar) orelse return error.UnknownEquipAvatar;
            const equipped = &config.avatar_config.items[idx].relics;
            for (equipped.items) |other| {
                if (other.id % 10 == slot) return error.DuplicateEquippedRelic;
            }
            try equipped.append(relic);
        }
        try config.relics.append(relic);
    }
    return config;
}
