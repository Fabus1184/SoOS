# SoOS Syscalls

| Name    | Description | rax | Arguments                                     | Return Value            |
| ------- | ----------- | --- | --------------------------------------------- | ----------------------- |
| print   |             | 0   | rbx: pointer to string, rcx: length of string | -                       |
| sleep   |             | 1   | rbx: time in milliseconds                     | -                       |
| exit    |             | 2   | rbx: exit code                                | -                       |
| listdir |             | 3   |                                               |                         |
| read    |             | 4   |                                               |                         |
| fork    |             | 5   |                                               |                         |
| open    |             | 6   |                                               |                         |
| close   |             | 7   |                                               |                         |
| mmap    |             | 8   | rbx: ?address                                 | rax: address, rbx: size |
| munmap  |             | 9   | rbx: address                                  | -                       |