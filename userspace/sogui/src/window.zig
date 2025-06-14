const std = @import("std");

const zclay = @import("zclay");

const soos = @import("soos");

const Position = struct {
    x: u32,
    y: u32,
};

pub const Window = struct {
    title: []const u8,
    position: Position,
    state: enum {
        normal,
        minimized,
        maximized,
        closed,
    } = .normal,

    allocator: std.mem.Allocator,

    drawBuffer: soos.Framebuffer,

    content: *anyopaque,
    contentDrawFn: *const fn (self: *anyopaque, framebuffer: soos.Framebuffer) void,
    contentContext: *zclay.Context,

    dragStartOffset: ?zclay.Vector2 = null,

    pub fn new(
        allocator: std.mem.Allocator,
        title: []const u8,
        width: u32,
        height: u32,
        content: *anyopaque,
        context: *zclay.Context,
        contentDrawFn: *const fn (self: *anyopaque, framebuffer: soos.Framebuffer) void,
    ) !Window {
        const drawBuffer = soos.Framebuffer{
            .ptr = try allocator.alignedAlloc(u32, 128 * 4, width * height),
            .width = width,
            .height = height,
        };
        drawBuffer.clear(0xFF_FF00FF);

        return Window{
            .title = title,
            .position = Position{ .x = 100, .y = 100 },
            .drawBuffer = drawBuffer,
            .allocator = allocator,
            .content = content,
            .contentDrawFn = contentDrawFn,
            .contentContext = context,
        };
    }

    pub fn draw(self: *Window) void {
        zclay.UI()(zclay.ElementDeclaration{
            .layout = zclay.LayoutConfig{
                .direction = .top_to_bottom,
                .sizing = .{ .w = .fixed(@floatFromInt(self.drawBuffer.width)), .h = .fixed(@floatFromInt(self.drawBuffer.height)) },
                .child_alignment = .{ .x = .left, .y = .top },
            },
            .floating = .{
                .attach_to = .to_root,
                .offset = .{
                    .x = @floatFromInt(self.position.x),
                    .y = @floatFromInt(self.position.y),
                },
            },
            .background_color = [_]f32{ 255, 0, 255, 255 }, // Dark gray background
        })({
            // draw titlebar
            zclay.UI()(.{
                .layout = .{
                    .direction = .left_to_right,
                    .sizing = .{ .w = .grow, .h = .fixed(40.0) },
                    .padding = .all(4.0),
                    .child_alignment = .{ .x = .left, .y = .center },
                    .child_gap = 0,
                },
                .background_color = [_]f32{ 0.0, 0.0, 50, 255 }, // Blue titlebar
            })({
                zclay.onHover(*Window, self, struct {
                    fn f(
                        _: zclay.ElementId,
                        pointer_data: zclay.PointerData,
                        self_: *Window,
                    ) void {
                        if (pointer_data.state == .pressed) {
                            if (self_.dragStartOffset) |offset| {
                                // Continue dragging
                                self_.position.x = @as(u32, @intFromFloat(pointer_data.position.x - offset.x));
                                self_.position.y = @as(u32, @intFromFloat(pointer_data.position.y - offset.y));
                            } else {
                                // Start dragging
                                self_.dragStartOffset = .{
                                    .x = pointer_data.position.x - @as(f32, @floatFromInt(self_.position.x)),
                                    .y = pointer_data.position.y - @as(f32, @floatFromInt(self_.position.y)),
                                };
                            }
                        } else {
                            // Stop dragging
                            self_.dragStartOffset = null;
                        }
                    }
                }.f);

                zclay.UI()(.{
                    .layout = .{
                        .direction = .left_to_right,
                        .sizing = .{ .w = .grow, .h = .fit },
                    },
                })({
                    zclay.text(self.title, .{
                        .font_id = 0,
                        .font_size = 16,
                        .color = [_]f32{ 255, 255, 255, 255 }, // White text
                    });
                });

                // buttons at end
                zclay.UI()(.{
                    .layout = .{
                        .direction = .left_to_right,
                        .sizing = .{ .w = .grow, .h = .fit },
                        .child_alignment = .{ .x = .right, .y = .center },
                        .child_gap = 8,
                    },
                })({
                    zclay.text("[_]", .{
                        .font_id = 0,
                        .font_size = 16,
                        .color = [_]f32{ 0, 255, 0, 255 }, // Green minimize button
                    });
                    zclay.text("[+]", .{
                        .font_id = 0,
                        .font_size = 16,
                        .color = [_]f32{ 0, 0, 255, 255 }, // Blue maximize button
                    });
                    zclay.UI()(.{})({
                        zclay.onHover(*Window, self, struct {
                            fn f(
                                _: zclay.ElementId,
                                pointer_data: zclay.PointerData,
                                self_: *Window,
                            ) void {
                                if (pointer_data.state == .pressed_this_frame) {
                                    self_.state = .closed;
                                }
                            }
                        }.f);
                        zclay.text("[X]", .{
                            .font_id = 0,
                            .font_size = 16,
                            .color = [_]f32{ 255, 0, 0, 255 }, // Red close button
                        });
                    });
                });
            });

            zclay.UI()(.{
                .layout = .{
                    .direction = .top_to_bottom,
                    .sizing = .{ .w = .grow, .h = .grow },
                    .padding = .all(8),
                    .child_alignment = .{ .x = .left, .y = .top },
                    .child_gap = 8,
                },
                .custom = zclay.CustomElementConfig{
                    .custom_data = &self.drawBuffer,
                },
            })({});
        });
    }

    pub fn drawContent(self: *Window) void {
        self.drawBuffer.clear(0xFF_000000);

        const before = zclay.getCurrentContext();
        zclay.setCurrentContext(self.contentContext);
        self.contentDrawFn(self.content, self.drawBuffer);
        zclay.setCurrentContext(before);
    }
};
