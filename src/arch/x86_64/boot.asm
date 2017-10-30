global start
extern long_mode_start                   ; this means that long_mode_start is coming from a seperate file

section .text
bits 32
start:
    mov esp, stack_top
	mov edi, ebx
    
    call check_multiboot
    call check_cpuid
    call check_long_mode

    call set_up_page_tables
    call enable_paging

    lgdt [gdt64.pointer]

    jmp gdt64.code:long_mode_start

    ; print `OK` to screen
    mov dword [0xb8000], 0x2f4b2f4f
    hlt

check_multiboot:
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret

.no_multiboot:
    mov al, "0"
    jmp error

check_cpuid:
    ; Check if CPUID is supported by attempting to flip the ID bit (bit 21)
    ; in the FLAGS register. If we can flip it, CPUID is available.

    ; Copy FLAGS in to EAX via stack
    pushfd
    pop eax

    ; Copy to ECX as well for comparing later on
    mov ecx, eax

    ; Flip the ID bit
    xor eax, 1 << 21

    ; Copy EAX to FLAGS via the stack
    push eax
    popfd

    ; Copy FLAGS back to EAX (with the flipped bit if CPUID is supported)
    pushfd
    pop eax

    ; Restore FLAGS from the old version stored in ECX (i.e. flipping the
    ; ID bit back if it was ever flipped).
    push ecx
    popfd

    ; Compare EAX and ECX. If they are equal then that means the bit
    ; wasn't flipped, and CPUID isn't supported.
    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
    mov al, "1"
    jmp error

check_long_mode:
    ; test if extended processor info in available
    mov eax, 0x80000000    ; implicit argument for cpuid
    cpuid                  ; get highest supported argument
    cmp eax, 0x80000001    ; it needs to be at least 0x80000001
    jb .no_long_mode       ; if it's less, the CPU is too old for long mode

    ; use extended info to test if long mode is available
    mov eax, 0x80000001    ; argument for extended processor info
    cpuid                  ; returns various feature b - its in ecx and edx
    test edx, 1 << 29      ; test if the LM-bit is set in the D-register
    jz .no_long_mode       ; If it's not set, there is no long mode
    ret
.no_long_mode:
    mov al, "2"
    jmp error

set_up_page_tables:
	mov eax, p4_table
	or eax, 0b11
	mov [p4_table + 511 * 8], eax ; map P4 recursively

	mov eax, p3_table             ; map first P4 entry to P3 table via eax
    or eax, 0b11                  ; set present + writable bits
    mov [p4_table], eax           ; put P3 with present + writable set into P4

    mov eax, p2_table             ; map first P3 entry to P2 table via eax
    or eax, 0b11                  ; set present + writable bits
    mov [p3_table], eax           ; same as P3 into P4 above
                                  ; 

    ; map each P2 entry to a huge 2MiB page
    mov ecx, 0                    ; counter variable

.map_p2_table:
    mov eax, 0x20000000           ; 2MiB
    mul ecx                       ; start address of ecx-th page - counter * 2MiB is the address of the start of this entry
    or eax, 0b10000011            ; present + writable + huge
    mov [p2_table + ecx * 8], eax ; map ecx-th entry - i-th entry is start of P2 + (counter * 8 bytes) 8B is size of entry

    inc ecx                       ; increase counter
    cmp ecx, 512                   ; if counter == 512 then the whole P2 table is mapped
    jne .map_p2_table             ; loop over all 512 entries in P2 and map them

    ret

enable_paging:
    mov eax, p4_table
    mov cr3, eax                  ; load P4 to cr3 via eax

    mov eax, cr4                  ; put cr4 into eax
    or eax, 1 << 5                ; flip the 5th bit of what was in cr4
    mov cr4, eax                   ; put the new value back into cr3

    mov ecx, 0xC0000080           ; set ecx to point to the EFER Model Specific Register (MSR)
    rdmsr                         ; read the value from the MSR into eax
    or eax, 1 << 8                ; flip the 8th bit of the msr, which is long mode
    wrmsr                         ; write the new value back to the MSR

    mov eax, cr0                  ; get cr0 into eax
    or eax, 1 << 31               ; flip bit 31 of cr0 to enable paging
    mov cr0, eax                  ; put the new value back into cr0

    ret

error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    hlt

section .rodata
gdt64: ; this is a special pointer that is passed to the lgdt instrction
    dq 0                                      ; 64bit zero entry
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53)  ; flip bits 43, 44, 47 and 53 to mark this segment as
                                              ;executable, a code or data segment, valid and 64bit, respectively
.code: equ $ - gdt64
    dq (1<<43) | (1<<44 ) | (1<<47) | (1<<53) ; label that always points to the start of the code segment even if the GDT changes
.pointer:
    dw $ - gdt64 - 1                          ; define a word as the current address minus the length of the GDT minus 1
    dq gdt64                                  ; define a quad as the length of the GDT

section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096
stack_bottom:
    resb 4096 * 4
stack_top:
