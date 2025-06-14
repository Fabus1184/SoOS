const std = @import("std");

const zclay = @import("zclay");

const soos = @import("soos");

const font = @import("font.zig");

pub const ClayBufferBackend = struct {
    buffer: soos.Framebuffer,

    pub fn init(buffer: soos.Framebuffer) ClayBufferBackend {
        return ClayBufferBackend{
            .buffer = buffer,
        };
    }

    pub fn draw(self: ClayBufferBackend, mainfont: *const font.Font, commands: zclay.ClayArray(zclay.RenderCommand)) void {
        for (0..commands.length) |i| {
            const command = commands.internal_array[i];
            switch (command.command_type) {
                .none => {},
                .text => {
                    const color = @as(u32, @intFromFloat(command.render_data.text.text_color[3])) << 24 |
                        @as(u32, @intFromFloat(command.render_data.text.text_color[0])) << 16 |
                        @as(u32, @intFromFloat(command.render_data.text.text_color[1])) << 8 |
                        @as(u32, @intFromFloat(command.render_data.text.text_color[2]));
                    const str = command.render_data.text.string_contents.chars[0..@intCast(command.render_data.text.string_contents.length)];
                    mainfont.blitText(color, str, @intFromFloat(command.bounding_box.x), @intFromFloat(command.bounding_box.y), 2, self.buffer);
                },
                .rectangle => {
                    self.buffer.fillRect(@intFromFloat(command.bounding_box.x), @intFromFloat(command.bounding_box.y), @intFromFloat(command.bounding_box.width), @intFromFloat(command.bounding_box.height), @as(u32, @intFromFloat(command.render_data.rectangle.background_color[3])) << 24 |
                        @as(u32, @intFromFloat(command.render_data.rectangle.background_color[0])) << 16 |
                        @as(u32, @intFromFloat(command.render_data.rectangle.background_color[1])) << 8 |
                        @as(u32, @intFromFloat(command.render_data.rectangle.background_color[2])), [_]usize{
                        @intFromFloat(command.render_data.rectangle.corner_radius.top_left),
                        @intFromFloat(command.render_data.rectangle.corner_radius.top_right),
                        @intFromFloat(command.render_data.rectangle.corner_radius.bottom_left),
                        @intFromFloat(command.render_data.rectangle.corner_radius.bottom_right),
                    });
                },
                .custom => {
                    if (command.render_data.custom.custom_data) |windowPtr| {
                        const src: *soos.Framebuffer = @alignCast(@ptrCast(windowPtr));
                        // copy window contents
                        const height: usize = @intFromFloat(command.bounding_box.height);
                        const width: usize = @intFromFloat(command.bounding_box.width);
                        const x: usize = @intFromFloat(command.bounding_box.x);
                        const y: usize = @intFromFloat(command.bounding_box.y);
                        for (0..height) |y_| {
                            @memcpy(
                                self.buffer.ptr[(y + y_) * self.buffer.width + x .. (y + y_) * self.buffer.width + x + width],
                                src.ptr[(y_ * src.width)..(y_ * src.width + width)],
                            );
                        }
                    } else {
                        std.log.err("custom command with no custom data", .{});
                    }
                },
                else => |cmd| {
                    std.log.err("unhandled command type: {s}", .{@tagName(cmd)});
                    @panic("unhandled command type");
                },
            }
        }
    }
};
