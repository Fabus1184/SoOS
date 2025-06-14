#include <stdint.h>

#include "types.h"

enum syscall_id_t {
    SYSCALL_PRINT = 0,
    SYSCALL_SLEEP = 1,
    SYSCALL_EXIT = 2,
    SYSCALL_LISTDIR = 3,
    SYSCALL_READ = 4,
    SYSCALL_FORK = 5,
    SYSCALL_OPEN = 6,
    SYSCALL_CLOSE = 7,
    SYSCALL_MMAP = 8,
    SYSCALL_MUNMAP = 9,
    SYSCALL_EXECVE = 10,
    SYSCALL_MAP_FRAMEBUFFER = 11,
    SYSCALL_WRITE = 12,
    SYSCALL_WAITPID = 13,
};

struct syscall_print_t {
    struct string_const_t message;
};

struct syscall_sleep_t {
    uint32_t milliseconds;
};

struct syscall_exit_t {
    uint32_t status;
};

static const uint32_t SYSCALL_LISTDIR_ENTRY_TYPE_FILE = 0;
static const uint32_t SYSCALL_LISTDIR_ENTRY_TYPE_DIR = 1;

struct syscall_listdir_entry_t {
    struct string_t name;
    uint32_t type;
};

typedef uint32_t syscall_listdir_error_t;
static const syscall_listdir_error_t SYSCALL_LISTDIR_ERROR_NONE = 0;
static const syscall_listdir_error_t SYSCALL_LISTDIR_ERROR_NOT_FOUND = 1;
static const syscall_listdir_error_t SYSCALL_LISTDIR_ERROR_BUFFER_TOO_SMALL = 2;
struct syscall_listdir_return_t {
    uint32_t entries_count;
    syscall_listdir_error_t error;
};

struct syscall_listdir_t {
    struct string_const_t path;
    struct syscall_listdir_entry_t *entries;
    uint32_t entries_len;
    struct syscall_listdir_return_t return_value;
};

typedef uint32_t syscall_read_error_t;
static const syscall_read_error_t SYSCALL_READ_ERROR_NONE = 0;
static const syscall_read_error_t SYSCALL_READ_ERROR_INVALID_FD = 1;
typedef uint32_t sysycall_read_option_t;
static const sysycall_read_option_t SYSCALL_READ_OPTION_NONE = 0;
static const sysycall_read_option_t SYSCALL_READ_OPTION_NON_BLOCKING = 1;
struct syscall_read_return_t {
    uint32_t bytes_read;
    syscall_read_error_t error;
};
struct syscall_read_t {
    fd_t fd;
    void *buf;
    uint32_t len;
    sysycall_read_option_t options;
    struct syscall_read_return_t return_value;
};

typedef uint32_t pid_t;

struct syscall_fork_return_t {
    pid_t child_pid;
};
struct syscall_fork_t {
    struct syscall_fork_return_t return_value;
};

typedef uint32_t syscall_open_error_t;
static const syscall_open_error_t SYSCALL_OPEN_ERROR_NONE = 0;
static const syscall_open_error_t SYSCALL_OPEN_ERROR_NOT_FOUND = 1;
struct syscall_open_return_t {
    fd_t fd;
    syscall_open_error_t error;
};
struct syscall_open_t {
    struct string_const_t path;
    struct syscall_open_return_t return_value;
};

typedef uint32_t syscall_close_error_t;
static const syscall_close_error_t SYSCALL_CLOSE_ERROR_NONE = 0;
static const syscall_close_error_t SYSCALL_CLOSE_ERROR_INVALID_FD = 1;
struct syscall_close_return_t {
    syscall_close_error_t error;
};
struct syscall_close_t {
    fd_t fd;
    struct syscall_close_return_t return_value;
};

typedef uint32_t syscall_mmap_error_t;
static const syscall_mmap_error_t SYSCALL_MMAP_ERROR_NONE = 0;
struct syscall_mmap_return_t {
    void *addr;
    syscall_mmap_error_t error;
};
struct syscall_mmap_t {
    uint32_t size;
    struct syscall_mmap_return_t return_value;
};

typedef uint32_t syscall_munmap_error_t;
static const syscall_munmap_error_t SYSCALL_MUNMAP_ERROR_NONE = 0;
static const syscall_munmap_error_t SYSCALL_MUNMAP_ERROR_INVALID_ADDR = 1;
struct syscall_munmap_return_t {
    syscall_munmap_error_t error;
};
struct syscall_munmap_t {
    void *addr;
    uint32_t size;
    struct syscall_munmap_return_t return_value;
};

typedef uint32_t syscall_execve_error_t;
static const syscall_execve_error_t SYSCALL_EXECVE_ERROR_NONE = 0;
static const syscall_execve_error_t SYSCALL_EXECVE_ERROR_NOT_FOUND = 1;
struct syscall_execve_return_t {
    syscall_execve_error_t error;
};
struct syscall_execve_t {
    struct string_const_t path;
    struct string_const_t *argv;
    uint32_t argv_len;
    struct string_const_t *envp;
    uint32_t envp_len;
    struct syscall_execve_return_t return_value;
};

struct syscall_map_framebuffer_return_t {
    void *addr;
    uint32_t width;
    uint32_t height;
};
struct syscall_map_framebuffer_t {
    struct syscall_map_framebuffer_return_t return_value;
};

typedef uint32_t syscall_write_error_t;
static const syscall_write_error_t SYSCALL_WRITE_ERROR_NONE = 0;
static const syscall_write_error_t SYSCALL_WRITE_ERROR_INVALID_FD = 1;
struct syscall_write_return_t {
    uint32_t bytes_written;
    syscall_write_error_t error;
};
struct syscall_write_t {
    fd_t fd;
    const void *buf;
    uint32_t len;
    struct syscall_write_return_t return_value;
};

typedef uint32_t syscall_waitpid_error_t;
static const syscall_waitpid_error_t SYSCALL_WAITPID_ERROR_NONE = 0;
static const syscall_waitpid_error_t SYSCALL_WAITPID_ERROR_INVALID_PID = 1;
struct syscall_waitpid_return_t {
    uint32_t status;
    syscall_waitpid_error_t error;
};
struct syscall_waitpid_t {
    pid_t pid;
    struct syscall_waitpid_return_t return_value;
};
