const std = @import("std");

const process = @import("process.zig");
const idt = @import("../idt.zig");

pub const Scheduler = struct {
    processes: std.ArrayList(process.Process),
    currentProcess: ?process.Process = null,

    pub fn init(self: *Scheduler, allocator: std.mem.Allocator) void {
        self.processes = std.ArrayList(process.Process).init(allocator);
    }

    pub fn add(self: *Scheduler, p: process.Process) !void {
        try self.processes.append(p);
    }

    pub fn schedule(self: *Scheduler) !noreturn {
        while (true) {
            if (self.processes.items.len == 0) {
                return error.NoProcessesLeft;
            }

            for (self.processes.items, 0..) |*ptr, i| {
                if (ptr.ready()) {
                    self.currentProcess = self.processes.orderedRemove(i);
                    self.currentProcess.?.run();
                }
            }
        }
    }

    pub fn storeState(
        self: *Scheduler,
        state: *const idt.State,
        stackFrame: *const idt.InterruptStackFrame,
        state_: process.State,
    ) !void {
        if (self.currentProcess) |*p| {
            p.savedState[0] = state.*;
            p.savedState[1] = stackFrame.*;
            p.state = state_;

            try self.processes.append(p.*);
            self.currentProcess = null;
        } else {
            return error.NoCurrentProcess;
        }
    }

    pub fn abort(self: *Scheduler) !void {
        if (self.currentProcess) |*p| {
            p.state = .aborted;
            try self.processes.append(p.*);
            self.currentProcess = null;
        } else {
            return error.NoCurrentProcess;
        }
    }
};
