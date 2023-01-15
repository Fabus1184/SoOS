[bits 32]

section .bss
align 4096

page_table_l4:
    resb 4096
page_table_l3:
    resb 4096
page_table_l2:
    resb 4096

section .text
align 4

global paging_and_long_mode
paging_and_long_mode:
    call setup_page_tables
    call enable_paging_and_long_mode
    ret

setup_page_tables:
    ; first entry in page table l4 is the address of page table l3
    mov eax, page_table_l3
    or eax, 0b11 ; present and writable
    mov [page_table_l4], eax
    
    ; first entry in page table l3 is the address of page table l2
    mov eax, page_table_l2
    or eax, 0b11 ; present and writable
    mov [page_table_l3], eax

    mov ecx, 0
    ; identity map the first 2GiB
setup_page_tables_loop:
    mov eax, 0x200000 ; 2MiB
    mul ecx
    or eax, 0b10000011 ; present, writable, huge page

    ; write the entry to page table l2
    mov [page_table_l2 + ecx * 8], eax

    inc ecx
    cmp ecx, 1024
    jne setup_page_tables_loop

    ret

enable_paging_and_long_mode:
    ; load the address of page table l4 into cr3
    mov eax, page_table_l4
    mov cr3, eax

    ; enable PAE
    mov eax, cr4
    or eax, (1 << 5)
    mov cr4, eax

    ; enable long mode
    mov ecx, 0xC0000080
    rdmsr
    or eax, (1 << 8)
    wrmsr

    ; enable paging
    mov eax, cr0
    or eax, (1 << 31)
    mov cr0, eax

    ret
