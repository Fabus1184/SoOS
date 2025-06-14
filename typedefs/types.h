#include <stdint.h>

struct string_t {
    char *ptr;
    uint32_t len;
};

struct string_const_t {
    const char *ptr;
    uint32_t len;
};

typedef int32_t fd_t;
static const fd_t FD_STDIN = 0;

// a pointer to this struct is on the stack at the entry point of the program
struct entry_t {
    uint32_t argc;
    struct string_const_t *argv;
};
