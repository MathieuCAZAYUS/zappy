const std = @import("std");
const allocator = std.heap.wasm_allocator;

const c = @cImport({
    @cInclude("ui_world.h");
});

var initialized: bool = false;
var last_tick_time: f32 = 0.0;
var time_scale: u8 = 1;

var g_heap: [65536]u8 = undefined;
var g_heap_idx: usize = 0;

// stdlib.h functions:
export fn malloc(size: usize) callconv(.c) ?*anyopaque {
    const aligned_size = (size + 7) & ~@as(usize, 7);
    if (g_heap_idx + aligned_size > g_heap.len) return null;

    const ptr = &g_heap[g_heap_idx];
    g_heap_idx += aligned_size;
    return ptr;
}

export fn free(ptr: ?*anyopaque) callconv(.c) void {
    _ = ptr;
}

export fn realloc(ptr: ?*anyopaque, size: usize) callconv(.c) ?*anyopaque {
    if (size == 0) return null;
    if (ptr) |p| {
        const new_ptr = malloc(size);
        if (new_ptr) |n_ptr| {
            @memcpy(@as([*]u8, @ptrCast(n_ptr))[0..size], @as([*]u8, @ptrCast(p))[0..size]);
            return n_ptr;
        }
        return null;
    }
    return malloc(size);
}

export fn abort() callconv(.c) noreturn {
    @trap();
}

fn initModule() void {
    if (initialized) return;

    var val = [_]u8{time_scale};
    const key_str = "sys:timescale";

    var key = c.ui_world_string_t{
        .ptr = @constCast(key_str.ptr),
        .len = key_str.len,
    };
    var value_list = c.ui_world_list_u8_t{
        .ptr = &val,
        .len = 1,
    };

    c.local_zappy_host_api_host_set_state(&key, &value_list);

    const msg = "Metronome Central (Zig via C) Initialized";
    var msg_str = c.ui_world_string_t{
        .ptr = @constCast(msg.ptr),
        .len = msg.len,
    };
    c.local_zappy_host_api_host_log(&msg_str);

    initialized = true;
}

export fn exports_ui_world_update_module(time: f32, dt: f32, w: f32, h: f32, ret_ptr: ?*c.ui_world_list_render_command_t) callconv(.c) void {
    _ = dt;
    _ = w;
    _ = h;

    g_heap_idx = 0;

    initModule();

    const ret = ret_ptr orelse return;

    if (time - last_tick_time >= 1.0) {
        const ev_name_str = "env:tick";
        const ev_payload_str = "1s";

        var ev_name = c.ui_world_string_t{
            .ptr = @constCast(ev_name_str.ptr),
            .len = ev_name_str.len,
        };
        var ev_payload = c.ui_world_string_t{
            .ptr = @constCast(ev_payload_str.ptr),
            .len = ev_payload_str.len,
        };

        c.local_zappy_host_api_emit_event(&ev_name, &ev_payload);
        last_tick_time = time;
    }

    ret.ptr = null;
    ret.len = 0;
}

export fn exports_ui_world_run_command(cmd_ptr: ?*c.ui_world_string_t, args_ptr: ?*c.ui_world_list_string_t, ret_ptr: ?*c.ui_world_response_command_t) callconv(.c) void {
    const cmd = cmd_ptr orelse return;
    const args = args_ptr orelse return;
    const ret = ret_ptr orelse return;

    const cmd_str = cmd.ptr[0..cmd.len];

    if (std.mem.eql(u8, cmd_str, "timescale") and args.len > 0) {
        const first_arg = args.ptr[0];
        const arg_str = first_arg.ptr[0..first_arg.len];

        const val_f = std.fmt.parseFloat(f32, arg_str) catch {
            ret.tag = 1;
            return;
        };

        var bytes: [4]u8 = undefined;
        @memcpy(&bytes, std.mem.asBytes(&val_f));

        const key_str = "sys:timescale";
        var key = c.ui_world_string_t{
            .ptr = @constCast(key_str.ptr),
            .len = key_str.len,
        };
        var value_list = c.ui_world_list_u8_t{
            .ptr = &bytes,
            .len = 4,
        };

        c.local_zappy_host_api_host_set_state(&key, &value_list);

        var ev_name = c.ui_world_string_t{
            .ptr = @constCast("sys:timescale_changed".ptr),
            .len = 21,
        };
        var ev_payload = c.ui_world_string_t{
            .ptr = first_arg.ptr,
            .len = first_arg.len,
        };

        c.local_zappy_host_api_emit_event(&ev_name, &ev_payload);

        ret.tag = 0;
        return;
    }

    ret.tag = 2;
}

export fn exports_ui_world_get_commands(ret_ptr: ?*c.ui_world_list_command_desc_t) callconv(.c) void {
    const ret = ret_ptr orelse return;

    const Storage = struct {
        var desc: [1]c.ui_world_command_desc_t = undefined;
    };

    const mod_name = "time_manager";
    const cmd_name = "timescale";
    const cmd_opts = "<float_value>";
    const cmd_help = "Adjust game tick speed (e.g.: 1.0 = normal, 0.5 = slow, 2.0 = fast, 0.0 = pause).";

    Storage.desc[0] = c.ui_world_command_desc_t{
        .module = .{ .ptr = @constCast(mod_name.ptr), .len = mod_name.len },
        .name = .{ .ptr = @constCast(cmd_name.ptr), .len = cmd_name.len },
        .options = .{ .ptr = @constCast(cmd_opts.ptr), .len = cmd_opts.len },
        .help = .{ .ptr = @constCast(cmd_help.ptr), .len = cmd_help.len },
    };

    ret.ptr = &Storage.desc;
    ret.len = 1;
}

export fn exports_ui_world_handle_event(event_ptr: ?*anyopaque, payload_ptr: ?*anyopaque) callconv(.c) void {
    _ = event_ptr;
    _ = payload_ptr;
}
export fn exports_ui_world_handle_input(state_ptr: ?*anyopaque) callconv(.c) void {
    _ = state_ptr;
}
export fn exports_ui_world_accept_log(segments_ptr: ?*anyopaque) callconv(.c) void {
    _ = segments_ptr;
}
export fn exports_ui_world_serialize(ret_ptr: ?*c.ui_world_list_u8_t) callconv(.c) void {
    if (ret_ptr) |ret| {
        ret.ptr = null;
        ret.len = 0;
    }
}
export fn exports_ui_world_deserialize(state_ptr: ?*c.ui_world_list_u8_t) callconv(.c) void {
    _ = state_ptr;
}
