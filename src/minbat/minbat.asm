section .data
    bat0_cap  db "/sys/class/power_supply/BAT0/capacity", 0
    bat0_stat db "/sys/class/power_supply/BAT0/status", 0
    bat1_cap  db "/sys/class/power_supply/BAT1/capacity", 0
    bat1_stat db "/sys/class/power_supply/BAT1/status", 0
    
    notify_bin db "/usr/bin/notify-send", 0
    arg1 db "notify-send", 0
    arg2 db "-u", 0
    arg3 db "critical", 0
    arg4 db "-i", 0
    arg5 db "battery-caution-symbolic", 0
    arg6 db "Battery Warning", 0
    
    msg_prefix db "Battery dropped to "
    msg_prefix_len equ $ - msg_prefix
    msg_suffix db "%!"
    msg_suffix_len equ $ - msg_suffix

    argv dq arg1, arg2, arg3, arg4, arg5, arg6, dyn_msg, 0

    sleep_ts dq 60      ; tv_sec
             dq 0       ; tv_nsec

section .bss
    saved_envp      resq 1
    active_bat_cap  resq 1
    active_bat_stat resq 1
    read_buf        resb 16
    stat_buf        resb 16
    dyn_msg         resb 64
    notified_20     resb 1
    notified_15     resb 1

section .text
global _start

_start:
    ; Save parent envp pointer to pass to execve
    mov rdi, [rsp]
    lea rdx, [rsp + 8 + rdi*8 + 8]
    mov [saved_envp], rdx

    ; Discover BAT0
    mov rdi, bat0_cap
    mov rsi, 4          ; R_OK
    mov rax, 21         ; SYS_ACCESS
    syscall
    test rax, rax
    jz .found_bat0

    ; Discover BAT1
    mov rdi, bat1_cap
    mov rsi, 4
    mov rax, 21
    syscall
    test rax, rax
    jz .found_bat1

    ; Exit if no battery found
    mov rax, 60
    mov rdi, 1
    syscall

.found_bat0:
    mov qword [active_bat_cap], bat0_cap
    mov qword [active_bat_stat], bat0_stat
    jmp main_loop

.found_bat1:
    mov qword [active_bat_cap], bat1_cap
    mov qword [active_bat_stat], bat1_stat

main_loop:
    ; SYS_OPEN(active_bat_cap, O_RDONLY)
    mov rdi, [active_bat_cap]
    xor rsi, rsi
    mov rax, 2
    syscall
    test rax, rax
    js sleep_60
    mov rbx, rax

    ; SYS_READ(fd, read_buf, 15)
    mov rdi, rbx
    mov rsi, read_buf
    mov rdx, 15
    mov rax, 0
    syscall

    ; SYS_CLOSE(fd)
    mov rdi, rbx
    mov rax, 3
    syscall

    ; Parse string to integer
    mov rsi, read_buf
    xor eax, eax
    xor ecx, ecx
.parse_loop:
    mov cl, byte [rsi]
    cmp cl, '0'
    jl .done_parse
    cmp cl, '9'
    jg .done_parse
    sub cl, '0'
    imul eax, eax, 10
    add eax, ecx
    inc rsi
    jmp .parse_loop

.done_parse:
    cmp eax, 15
    jle .check_15
    cmp eax, 20
    jle .check_20

    ; Capacity > 20, reset state
    mov byte [notified_15], 0
    mov byte [notified_20], 0
    jmp sleep_60

.check_15:
    cmp byte [notified_15], 1
    je sleep_60
    call is_discharging
    test rax, rax
    jz sleep_60
    mov r12, 15
    jmp do_notify

.check_20:
    cmp byte [notified_20], 1
    je sleep_60
    call is_discharging
    test rax, rax
    jz sleep_60
    mov r12, 20
    jmp do_notify

do_notify:
    ; Stitch dynamic message
    mov rdi, dyn_msg
    mov rsi, msg_prefix
    mov rcx, msg_prefix_len
    cld
    rep movsb
    
    mov rsi, read_buf
.copy_num:
    mov al, [rsi]
    cmp al, '0'
    jl .copy_suffix
    cmp al, '9'
    jg .copy_suffix
    mov [rdi], al
    inc rsi
    inc rdi
    jmp .copy_num

.copy_suffix:
    mov rsi, msg_suffix
    mov rcx, msg_suffix_len
    rep movsb
    mov byte [rdi], 0

    ; SYS_FORK
    mov rax, 57
    syscall
    test rax, rax
    js sleep_60
    jz .child

.parent:
    ; SYS_WAIT4
    mov rdi, rax
    sub rsp, 8
    mov rsi, rsp
    xor rdx, rdx
    xor r10, r10
    mov rax, 61
    syscall
    
    ; Check child exit status
    mov ebx, dword [rsp]
    add rsp, 8
    test ebx, ebx
    jnz sleep_60
    
    ; Success, mark notified
    cmp r12, 15
    je .mark_15
    mov byte [notified_20], 1
    jmp sleep_60
.mark_15:
    mov byte [notified_15], 1
    mov byte [notified_20], 1
    jmp sleep_60

.child:
    ; SYS_EXECVE
    mov rdi, notify_bin
    mov rsi, argv
    mov rdx, [saved_envp]
    mov rax, 59
    syscall
    
    mov rax, 60
    mov rdi, 1
    syscall

sleep_60:
    ; SYS_NANOSLEEP
    mov rdi, sleep_ts
    xor rsi, rsi
    mov rax, 35
    syscall
    jmp main_loop

is_discharging:
    ; SYS_OPEN(active_bat_stat, O_RDONLY)
    mov rdi, [active_bat_stat]
    xor rsi, rsi
    mov rax, 2
    syscall
    test rax, rax
    js .not_discharging
    mov rbx, rax

    ; SYS_READ
    mov rdi, rbx
    mov rsi, stat_buf
    mov rdx, 15
    mov rax, 0
    syscall

    ; SYS_CLOSE
    mov rdi, rbx
    mov rax, 3
    syscall

    mov al, byte [stat_buf]
    cmp al, 'D'
    jne .not_discharging
    mov rax, 1
    ret
.not_discharging:
    xor rax, rax
    ret
