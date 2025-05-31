#include "../libsoos/libsoos.h"

void _start() {
    print("Hello from userspace!\n");

    int a = 0, b = 1;
    for (int i = 0; i < 20; i++) {
        char str[32];
        itoa(a, str);
        print(str);
        print("\n");

        int tmp = a;
        a = b;
        b = tmp + b;

        sleep(500);
    }

    print("Goodbye from userspace!\n");

    exit(69420);
}
