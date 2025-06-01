#include "../libsoos/libsoos.h"

void _start() {
    print("Hello from userspace!\n");

    print("Listing contents of root directory...:\n");
    for (int i = 0;; i++) {
        char buffer[1024];

        uint64_t n = listdir("/", i, buffer);
        if (n == 0) {
            print("No more entries.\n");
            break;
        }
        buffer[n] = '\0'; // Null-terminate the string

        print("-> '");
        print(buffer);
        print("'\n");
    }

    print("Goodbye from userspace!\n");

    exit(69420);
}
