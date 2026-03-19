; BerkeOS - kernel.asm
; 64-bit kernel entry point
; Prints "Welcome to BerkeOS" on the VGA text buffer

global long_mode_start

section .text
bits 64
long_mode_start:
    ; Clear segment registers
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; Clear the screen (VGA text mode: 80x25, base 0xb8000)
    mov rdi, 0xb8000
    mov rcx, 80 * 25
    mov rax, 0x0f200f20   ; Two spaces with white-on-black attr
    rep stosd

    ; Print "Welcome to BerkeOS"
    mov rsi, msg
    mov rdi, 0xb8000
    mov ah, 0x0f          ; White on black attribute
.print_loop:
    lodsb
    test al, al
    jz .done
    mov [rdi], al
    mov [rdi+1], ah
    add rdi, 2
    jmp .print_loop
.done:
    ; Halt forever
    cli
.halt:
    hlt
    jmp .halt

section .rodata
msg db "Welcome to BerkeOS", 0
