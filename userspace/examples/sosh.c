#include "../libsoos/libsoos.h"

// SoOS Shell (sosh)

#define BANNER                                                                                                                                                                     \
    ".d88888b            .88888.  .d88888b\n"                                                                                                                                      \
    "88.    \"'          d8'   `8b 88.    \"'\n"                                                                                                                                   \
    "`Y88888b. .d8888b. 88     88 `Y88888b.\n"                                                                                                                                     \
    "      `8b 88'  `88 88     88       `8b\n"                                                                                                                                     \
    "d8'   .8P 88.  .88 Y8.   .8P d8'   .8P\n"                                                                                                                                     \
    " Y88888P  `88888P'  `8888P'   Y88888P\n"

#define ANSI_FG_GREEN "\033[32m"
#define ANSI_FG_RED "\033[31m"
#define ANSI_FG_YELLOW "\033[33m"
#define ANSI_FG_BLUE "\033[34m"
#define ANSI_FG_MAGENTA "\033[35m"
#define ANSI_FG_CYAN "\033[36m"
#define ANSI_FG_WHITE "\033[37m"

#define PROMPT ANSI_FG_BLUE "sosh> "

void run_command(const char *cmd);

void _start() {
    // clear the screen
    print("\033[2J\033[H");

    print(ANSI_FG_GREEN BANNER);
    print(ANSI_FG_YELLOW "\nWelcome to SoOS Shell (sosh)!\n");
    print("Type 'exit' to quit.\n");

    print(PROMPT);

    char command[512] = {0};
    uint64_t command_len = 0;

    while (1) {
        char buffer[64] = {0};
        uint64_t n = read(0, buffer, sizeof(buffer) - 1);
        if (n == 0) {
            print(ANSI_FG_RED "\nEOF or error reading input.\n");
            exit(1);
        }

        for (uint64_t i = 0; i < n; ++i) {
            switch (buffer[i]) {
            case '\n':
                command[command_len] = '\0';

                print("\n");
                if (command_len > 0) {
                    run_command(command);
                }

                command_len = 0;
                memset(command, 0, sizeof(command)); // Clear the command buffer

                print(PROMPT);

                break;
            case '\b': // Handle backspace
                if (command_len > 0) {
                    command_len--;
                    print("\b \b"); // Move cursor back, print space, move cursor back again
                }
                break;
            default:
                command[command_len++] = buffer[i];
                char display_char[2] = {buffer[i], '\0'};
                print(display_char);
                break;
            }
        }
    }
}

struct Command {
    const char *name;
    void (*func)(uint64_t argc, const char **argv);
};

void command_exit(uint64_t argc, const char **argv);
void command_help(uint64_t argc, const char **argv);
void command_ls(uint64_t argc, const char **argv);
void command_fork(uint64_t argc, const char **argv);
void command_clear(uint64_t argc, const char **argv);
void command_cat(uint64_t argc, const char **argv);

const struct Command commands[] = {
    {.name = "exit", .func = command_exit}, {.name = "help", .func = command_help},   {.name = "ls", .func = command_ls},
    {.name = "fork", .func = command_fork}, {.name = "clear", .func = command_clear}, {.name = "cat", .func = command_cat},
};

void run_command(const char *cmd) {
    // Tokenize the command input by spaces, replace spaces with null terminators
    const char *argv[64] = {0};
    uint64_t argc = 0;
    const char *start = cmd;
    while (*start != '\0' && argc < 64) {
        // Skip leading spaces
        while (*start == ' ') {
            start++;
        }

        if (*start == '\0') {
            break; // End of command
        }

        // Find the end of the token
        const char *end = start;
        while (*end != ' ' && *end != '\0') {
            end++;
        }

        // Null-terminate the token
        char *token = (char *)start;
        if (end - start > 0) {
            argv[argc++] = token;
            if (*end != '\0') {
                *(char *)end = '\0'; // Replace space with null terminator
            }
        }

        start = end + 1; // Move to the next token
    }

    for (uint64_t i = 0; i < sizeof(commands) / sizeof(commands[0]); i++) {
        if (strcmp(argv[0], commands[i].name) == 0) {
            commands[i].func(argc, argv);
            return;
        }
    }

    print(ANSI_FG_RED "Unknown command: '" ANSI_FG_WHITE);
    print(cmd);
    print(ANSI_FG_RED "'\n");
}

void command_exit(uint64_t argc, const char **argv) {
    print("Exiting SoOS Shell...\n");
    exit(0);
}

void command_help(uint64_t argc, const char **argv) {
    if (argc == 1) {
        print("Available commands:\n");
        for (uint64_t i = 0; i < sizeof(commands) / sizeof(commands[0]); i++) {
            print(" - ");
            print(commands[i].name);
            print("\n");
        }
    } else if (argc == 2) {
        print("Help for command: ");
        print(argv[1]);
        print("\n");
        print("No specific help available for this command.\n");
    } else {
        print(ANSI_FG_RED "Usage: help [command]\n");
        print("Type 'help' for a list of commands.\n");
    }
}

void command_ls(uint64_t argc, const char **argv) {
    if (argc != 2) {
        print(ANSI_FG_RED "Usage: ls <directory>\n");
        return;
    }

    uint64_t i;
    for (i = 0;; ++i) {
        char buffer[1024] = {0};
        uint64_t length = listdir(argv[1], i, buffer);
        if (length == 0) {
            break; // No more entries
        }

        print(buffer);
        print("\n");
    }

    if (i == 0) {
        print("directory is empty or does not exist.\n");
    }
}

void command_fork(uint64_t argc, const char **argv) {
    uint32_t pid = fork();
    if (pid == 0) {
        // Child process
        print(ANSI_FG_GREEN "Hello from the child process!\n");
        exit(0);
    } else {
        // Parent process
        print(ANSI_FG_GREEN "Forked child process with PID: ");
        char buffer[32];
        itoa(pid, buffer);
        print(buffer);
        print("\n");
    }
}

void command_clear(uint64_t argc, const char **argv) {
    // Clear the screen
    print("\033[2J\033[H");
}

void command_cat(uint64_t argc, const char **argv) {
    if (argc != 2) {
        print(ANSI_FG_RED "Usage: cat <file>\n");
        return;
    }

    char buffer[4096] = {0};
    int64_t fd = open(argv[1]);
    if (fd < 0) {
        print(ANSI_FG_RED "Error opening file: ");
        print(argv[1]);
        print("\n");
        return;
    }

    uint64_t bytes_read = read(fd, buffer, sizeof(buffer) - 1);
    if (bytes_read < 0) {
        print(ANSI_FG_RED "Error reading file: ");
        print(argv[1]);
        print("\n");
        return;
    }
    buffer[bytes_read] = '\0'; // Null-terminate the string

    int64_t ret = close(fd);
    if (ret < 0) {
        print(ANSI_FG_RED "Error closing file: ");
        print(argv[1]);
        print("\n");
        return;
    }

    print(buffer);
    print("\n");
}