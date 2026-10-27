// Windows Version ABI; system DLL is never redistributed.
pub const EXPORTS: &[(&[u8],u16)] = &[
    (b"GetFileVersionInfoA\0", 1),
    (b"GetFileVersionInfoByHandle\0", 2),
    (b"GetFileVersionInfoExA\0", 3),
    (b"GetFileVersionInfoExW\0", 4),
    (b"GetFileVersionInfoSizeA\0", 5),
    (b"GetFileVersionInfoSizeExA\0", 6),
    (b"GetFileVersionInfoSizeExW\0", 7),
    (b"GetFileVersionInfoSizeW\0", 8),
    (b"GetFileVersionInfoW\0", 9),
    (b"VerFindFileA\0", 10),
    (b"VerFindFileW\0", 11),
    (b"VerInstallFileA\0", 12),
    (b"VerInstallFileW\0", 13),
    (b"VerLanguageNameA\0", 14),
    (b"VerLanguageNameW\0", 15),
    (b"VerQueryValueA\0", 16),
    (b"VerQueryValueW\0", 17),
];
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_0() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 0]", table=sym FUNCTIONS);
}
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_1() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 4]", table=sym FUNCTIONS);
}
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_2() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 8]", table=sym FUNCTIONS);
}
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_3() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 12]", table=sym FUNCTIONS);
}
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_4() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 16]", table=sym FUNCTIONS);
}
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_5() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 20]", table=sym FUNCTIONS);
}
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_6() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 24]", table=sym FUNCTIONS);
}
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_7() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 28]", table=sym FUNCTIONS);
}
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_8() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 32]", table=sym FUNCTIONS);
}
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_9() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 36]", table=sym FUNCTIONS);
}
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_10() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 40]", table=sym FUNCTIONS);
}
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_11() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 44]", table=sym FUNCTIONS);
}
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_12() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 48]", table=sym FUNCTIONS);
}
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_13() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 52]", table=sym FUNCTIONS);
}
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_14() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 56]", table=sym FUNCTIONS);
}
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_15() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 60]", table=sym FUNCTIONS);
}
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn forward_16() {
    core::arch::naked_asm!("jmp dword ptr [{table} + 64]", table=sym FUNCTIONS);
}
