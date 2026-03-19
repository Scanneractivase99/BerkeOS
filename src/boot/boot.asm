; BerkeOS — boot.asm
; Multiboot2 header + 32-bit protected mode setup → Long Mode → Rust kernel
; NASM: nasm -f elf64 src/boot/boot.asm -o build/boot.o

global start
extern kernel_main

; ─────────────────────────────────────────────────────────────────────────────
; Multiboot2 Header — MUST appear in first 32 KiB of image
; ─────────────────────────────────────────────────────────────────────────────
section .multiboot_header
align 8
header_start:
    dd 0xe85250d6                   ; Multiboot2 magic
    dd 0                            ; arch: i386 protected mode
    dd header_end - header_start    ; total header length
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

    ; ── Framebuffer request tag (type 5) ─────────────────────────────────────
    align 8
    dw 5            ; tag type = framebuffer
    dw 0            ; flags = 0
    dd 20           ; size = 20 bytes
    dd 1920         ; preferred width
    dd 1080         ; preferred height
    dd 32           ; preferred bpp

    ; ── End tag ───────────────────────────────────────────────────────────────
    align 8
    dw 0
    dw 0
    dd 8
header_end:

; ─────────────────────────────────────────────────────────────────────────────
; BSS — page tables + stack
; Using 2 MiB huge pages: P4 + P3 + multiple P2 tables
; We cover 0x0000_0000 → 0xFFFF_FFFF (4 GiB) with 4 P2 tables
; Each P2 table covers 1 GiB (512 × 2 MiB entries)
; ─────────────────────────────────────────────────────────────────────────────
section .bss
align 4096
p4_table:   resb 4096       ; PML4
p3_table:   resb 4096       ; PDPT — 4 entries, one per GiB
p2_table0:  resb 4096       ; PD for GiB 0: 0x0000_0000–0x3FFF_FFFF
p2_table1:  resb 4096       ; PD for GiB 1: 0x4000_0000–0x7FFF_FFFF
p2_table2:  resb 4096       ; PD for GiB 2: 0x8000_0000–0xBFFF_FFFF
p2_table3:  resb 4096       ; PD for GiB 3: 0xC000_0000–0xFFFF_FFFF
stack_bottom:
            resb 65536      ; 64 KiB stack
stack_top:

; ─────────────────────────────────────────────────────────────────────────────
; GDT — in .data so it has a fixed address before .text
; ─────────────────────────────────────────────────────────────────────────────
section .data
align 16
gdt64:
    dq 0                                        ; null descriptor
    dq (1<<43)|(1<<44)|(1<<47)|(1<<53)          ; 64-bit code segment
gdt64_end:

align 4
gdt64_ptr:
    dw gdt64_end - gdt64 - 1
    dq gdt64

align 4
mb2_info_save: dd 0

; ─────────────────────────────────────────────────────────────────────────────
; 32-bit protected mode entry
; ─────────────────────────────────────────────────────────────────────────────
section .text.boot
bits 32

start:
    cli
    mov  esp, stack_top
    mov  [mb2_info_save], ebx
    
    ; Skip VGA debug writes - VGA doesn't exist on UEFI!
    ; Debug messages moved to early Rust kernel code where we can handle missing VGA
    
    ; Verify Multiboot2 magic
    cmp  eax, 0x36d76289
    jne  .bad_magic
    
    ; Zero ALL of BSS (page tables + stack) before using them
    mov  edi, p4_table
    mov  ecx, (stack_top - p4_table) / 4
    xor  eax, eax
    rep  stosd
    
    ; Skip VGA debug writes - VGA doesn't exist on UEFI!
    
    call setup_page_tables
    
    ; Skip VGA debug writes
    
    call enable_paging
    
    ; Skip VGA debug writes
    
    lgdt [gdt64_ptr]
    
    ; Skip VGA debug writes
    
    jmp  0x08:long_mode_start

.bad_magic:
    ; Try to show error on VGA if available, otherwise just halt
    ; Skip VGA writes on UEFI - they cause a fault
.hang32:
    hlt
    jmp  .hang32

; ─────────────────────────────────────────────────────────────────────────────
; setup_page_tables — 2 MiB huge pages
; Key insight: use a RUNNING ADDRESS in esi, increment by 0x200000 each step
; Never do (ecx * 0x200000 + base) — that overflows for large ecx + large base
;
; For GiB 0: esi starts at 0x00000000, increments to 0x3FE00000
; For GiB 1: esi starts at 0x40000000, increments to 0x7FE00000
; For GiB 2: esi starts at 0x80000000, increments to 0xBFE00000
; For GiB 3: esi starts at 0xC0000000, increments to 0xFFE00000
;            (0xFFE00000 + 0x200000 = 0x00000000 — wrap, but we stop at 512)
;
; Each entry: [table + ecx*8] = esi | 0x83 (present+writable+2MB)
;             [table + ecx*8 + 4] = 0       (high 32 bits = 0, phys < 4GiB)
; ─────────────────────────────────────────────────────────────────────────────
setup_page_tables:
    ; ── P4[0] → p3_table ─────────────────────────────────────────────────────
    mov  eax, p3_table
    or   eax, 0x03
    mov  [p4_table], eax
    mov  dword [p4_table + 4], 0

    ; ── P3[0] → p2_table0 ────────────────────────────────────────────────────
    mov  eax, p2_table0
    or   eax, 0x03
    mov  [p3_table + 0], eax
    mov  dword [p3_table + 4], 0

    ; ── P3[1] → p2_table1 ────────────────────────────────────────────────────
    mov  eax, p2_table1
    or   eax, 0x03
    mov  [p3_table + 8], eax
    mov  dword [p3_table + 12], 0

    ; ── P3[2] → p2_table2 ────────────────────────────────────────────────────
    mov  eax, p2_table2
    or   eax, 0x03
    mov  [p3_table + 16], eax
    mov  dword [p3_table + 20], 0

    ; ── P3[3] → p2_table3 ────────────────────────────────────────────────────
    mov  eax, p2_table3
    or   eax, 0x03
    mov  [p3_table + 24], eax
    mov  dword [p3_table + 28], 0

    ; ── Fill p2_table0: 512 entries starting at physical 0x00000000 ──────────
    mov  ecx, 0
    mov  esi, 0x00000000
.fill0:
    cmp  ecx, 512
    jge  .fill0_done
    mov  eax, esi
    or   eax, 0x83
    mov  [p2_table0 + ecx*8],     eax
    mov  dword [p2_table0 + ecx*8 + 4], 0
    add  esi, 0x200000
    inc  ecx
    jmp  .fill0
.fill0_done:

    ; ── Fill p2_table1: 512 entries starting at physical 0x40000000 ──────────
    mov  ecx, 0
    mov  esi, 0x40000000
.fill1:
    cmp  ecx, 512
    jge  .fill1_done
    mov  eax, esi
    or   eax, 0x83
    mov  [p2_table1 + ecx*8],     eax
    mov  dword [p2_table1 + ecx*8 + 4], 0
    add  esi, 0x200000
    inc  ecx
    jmp  .fill1
.fill1_done:

    ; ── Fill p2_table2: 512 entries starting at physical 0x80000000 ──────────
    mov  ecx, 0
    mov  esi, 0x80000000
.fill2:
    cmp  ecx, 512
    jge  .fill2_done
    mov  eax, esi
    or   eax, 0x83
    mov  [p2_table2 + ecx*8],     eax
    mov  dword [p2_table2 + ecx*8 + 4], 0
    add  esi, 0x200000
    inc  ecx
    jmp  .fill2
.fill2_done:

    ; ── Fill p2_table3: 512 entries starting at physical 0xC0000000 ──────────
    ; Covers 0xC0000000 → 0xFFFFFFFF — includes QEMU framebuffer at 0xFD000000
    ; When esi reaches 0xFFE00000, adding 0x200000 wraps to 0x00000000
    ; This is fine — we stop at ecx=512 before writing the wrapped entry
    mov  ecx, 0
    mov  esi, 0xC0000000
.fill3:
    cmp  ecx, 512
    jge  .fill3_done
    mov  eax, esi
    or   eax, 0x83
    mov  [p2_table3 + ecx*8],     eax
    mov  dword [p2_table3 + ecx*8 + 4], 0
    add  esi, 0x200000
    inc  ecx
    jmp  .fill3
.fill3_done:

    ret

; ─────────────────────────────────────────────────────────────────────────────
enable_paging:
    ; CR3 = physical address of P4
    mov  eax, p4_table
    mov  cr3, eax

    ; Enable PAE (Physical Address Extension) — required for Long Mode
    mov  eax, cr4
    or   eax, (1 << 5)     ; PAE bit
    mov  cr4, eax

    ; Enable Long Mode in EFER MSR
    mov  ecx, 0xC0000080
    rdmsr
    or   eax, (1 << 8)     ; LME bit
    wrmsr

    ; Enable paging (PG) — this activates Long Mode
    mov  eax, cr0
    or   eax, (1 << 31) | (1 << 0)  ; PG + PE
    mov  cr0, eax

    mov  cr0, eax
    
    ; Skip VGA debug - VGA may not exist on UEFI
    
    ret

; ─────────────────────────────────────────────────────────────────────────────
; 64-bit long mode
; ─────────────────────────────────────────────────────────────────────────────
section .text
bits 64

long_mode_start:
    ; Clear segment registers
    xor  ax, ax
    mov  ss, ax
    mov  ds, ax
    mov  es, ax
    mov  fs, ax
    mov  gs, ax
    
    ; Skip VGA debug - VGA may not exist on UEFI
    
    ; Pass MB2 info pointer in edi (first arg, System V AMD64 ABI)
    mov  edi, dword [mb2_info_save]
    
    ; Skip VGA debug
    
    call kernel_main
    
    ; Should never return — halt forever

    ; Should never return — halt forever
    cli
.halt:
    hlt
    jmp  .halt
