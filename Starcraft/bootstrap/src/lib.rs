//! Local Version compatibility loader. Never opens or writes another process.
#![allow(non_snake_case,static_mut_refs)]
use std::{ffi::c_void,ptr};
type Handle=*mut c_void;
#[link(name="kernel32")]
extern "system" {
    fn GetSystemDirectoryW(path:*mut u16,size:u32)->u32;
    fn LoadLibraryW(path:*const u16)->Handle;
    fn GetProcAddress(module:Handle,name:*const u8)->Handle;
    fn GetModuleFileNameW(module:Handle,path:*mut u16,size:u32)->u32;
    fn DisableThreadLibraryCalls(module:Handle)->i32;
    fn VirtualProtect(address:Handle,size:usize,protect:u32,old:*mut u32)->i32;
    fn FlushInstructionCache(process:Handle,address:Handle,size:usize)->i32;
    fn GetCurrentProcess()->Handle;
    fn GetPrivateProfileIntW(section:*const u16,key:*const u16,default:i32,file:*const u16)->u32;
    fn ExitProcess(code:u32)->!;
}
#[link(name="user32")]
extern "system" {fn MessageBoxW(window:Handle,text:*const u16,title:*const u16,flags:u32)->i32;}
static mut FUNCTIONS:[usize;17]=[0;17];
include!("exports.rs");
const ENTRY:usize=0x404c21;
const ORIGINAL:[u8;7]=[0x6a,0x60,0x68,0xf0,0xe5,0x4f,0x00];
static mut SIZE:[u32;2]=[640,480];
static mut DLL_PATH:[u16;32768]=[0;32768];
fn wide(s:&str)->Vec<u16>{s.encode_utf16().chain(Some(0)).collect()}
unsafe fn replace_entry(bytes:&[u8])->bool {
    let mut old=0;
    if VirtualProtect(ENTRY as Handle,7,0x40,&mut old)==0{return false}
    ptr::copy_nonoverlapping(bytes.as_ptr(),ENTRY as *mut u8,bytes.len());
    let mut ignored=0;VirtualProtect(ENTRY as Handle,7,old,&mut ignored);
    FlushInstructionCache(GetCurrentProcess(),ENTRY as Handle,7);true
}
// The callback runs at the EXE entry point, after Windows has initialized all
// dependencies and released its loader lock. Preserve the original entry ABI.
#[unsafe(naked)]
unsafe extern "C" fn entry_thunk(){core::arch::naked_asm!(
    "pushfd","pushad","mov ebp, esp","and esp, -16",
    "call {initialize}","mov esp, ebp","popad","popfd",
    "push 0x404c21","ret",initialize=sym initialize
);}
unsafe extern "C" fn initialize(){
    let restored=replace_entry(&ORIGINAL);
    let module=if restored {LoadLibraryW(DLL_PATH.as_ptr())}else{ptr::null_mut()};
    let proc=if module.is_null(){ptr::null_mut()}else{GetProcAddress(module,b"InitializeResolution\0".as_ptr())};
    if !proc.is_null(){
        let f:unsafe extern "system" fn(*const u32)->u32=std::mem::transmute(proc);
        if f(SIZE.as_ptr())==1{return}
    }
    MessageBoxW(ptr::null_mut(),wide("No se pudo iniciar StarCraft HD. Consulta rust-resolution.log en la carpeta del juego.").as_ptr(),wide("StarCraft HD").as_ptr(),0x10);
    ExitProcess(1);
}
#[no_mangle]
pub unsafe extern "system" fn DllMain(module:Handle,reason:u32,_reserved:Handle)->i32 {
    if reason!=1{return 1}
    DisableThreadLibraryCalls(module);
    // Explicit system path prevents recursive loading of this proxy.
    let mut system=[0u16;32768];let n=GetSystemDirectoryW(system.as_mut_ptr(),system.len() as u32) as usize;
    if n==0 || n+13>=system.len(){return 0}
    for (i,c) in "\\version.dll\0".encode_utf16().enumerate(){system[n+i]=c;}
    let real=LoadLibraryW(system.as_ptr());if real.is_null(){return 0}
    for (i,&(name,ordinal)) in EXPORTS.iter().enumerate(){
        let key=if name[0]==0 {ordinal as usize as *const u8}else{name.as_ptr()};
        FUNCTIONS[i]=GetProcAddress(real,key) as usize;
        if FUNCTIONS[i]==0{return 0}
    }
    let mut exe=[0u16;32768];let n=GetModuleFileNameW(ptr::null_mut(),exe.as_mut_ptr(),exe.len() as u32) as usize;
    if n==0 || n>=exe.len(){return 0}
    let path=String::from_utf16_lossy(&exe[..n]);
    let Some((dir,name))=path.rsplit_once('\\')else{return 1};
    if !name.eq_ignore_ascii_case("StarCraft.exe"){return 1}
    let config=wide(&format!("{dir}\\sc-hd-session.ini"));
    SIZE=[GetPrivateProfileIntW(wide("HD").as_ptr(),wide("width").as_ptr(),640,config.as_ptr()),
          GetPrivateProfileIntW(wide("HD").as_ptr(),wide("height").as_ptr(),480,config.as_ptr())];
    if SIZE==[640,480]{return 1}
    if std::slice::from_raw_parts(ENTRY as *const u8,7)!=ORIGINAL{return 0}
    let dll=wide(&format!("{dir}\\sc_resolution.dll"));if dll.len()>DLL_PATH.len(){return 0}
    DLL_PATH[..dll.len()].copy_from_slice(&dll);
    let mut jump=[0x90u8;7];jump[0]=0xe9;
    jump[1..5].copy_from_slice(&((entry_thunk as *const () as usize).wrapping_sub(ENTRY+5) as u32).to_le_bytes());
    if replace_entry(&jump){1}else{0}
}
