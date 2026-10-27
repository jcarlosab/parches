//! Resolution extension for the Spanish 1.00.0010 executable. No game files are patched on disk.
use std::{ffi::c_void, path::PathBuf, sync::OnceLock};
type Handle = *mut c_void;
#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: *const u16) -> Handle;
    fn GetModuleFileNameW(module: Handle, name: *mut u16, size: u32) -> u32;
    fn GetSystemDirectoryW(name: *mut u16, size: u32) -> u32;
    fn LoadLibraryW(name: *const u16) -> Handle;
    fn GetProcAddress(module: Handle, name: *const u8) -> Handle;
    fn VirtualAlloc(address: Handle, size: usize, kind: u32, protect: u32) -> Handle;
    fn VirtualProtect(address: Handle, size: usize, protect: u32, old: *mut u32) -> i32;
    fn FlushInstructionCache(process: Handle, address: *const c_void, size: usize) -> i32;
    fn GetCurrentProcess() -> Handle;
    fn GetTickCount() -> u32;
    fn DisableThreadLibraryCalls(module: Handle) -> i32;
}

static REAL_D3D: OnceLock<usize> = OnceLock::new();
static PATCH: OnceLock<bool> = OnceLock::new();
static MENU_SCENE: [u16;14] = [77,101,110,117,83,99,101,110,101,46,100,108,108,0];
const MODES: [(u32,u32,u32);3] = [(10,1920,1080),(11,1280,720),(12,2560,1440)];

fn game_exe() -> PathBuf {
    use std::os::windows::ffi::OsStringExt;
    let mut name = [0u16;32768];
    let n=unsafe { GetModuleFileNameW(std::ptr::null_mut(),name.as_mut_ptr(),name.len() as u32) };
    std::ffi::OsString::from_wide(&name[..n as usize]).into()
}
fn log(message: &str) {
    use std::io::Write;
    if let Some(dir)=game_exe().parent() {
        if let Ok(mut file)=std::fs::OpenOptions::new().create(true).append(true).open(dir.join("WotR-HD.log")) {
            let _=writeln!(file,"{message}");
        }
    }
}
unsafe fn real_d3d() -> usize {
    *REAL_D3D.get_or_init(|| {
        let mut path=[0u16;32768];
        let n=GetSystemDirectoryW(path.as_mut_ptr(),path.len() as u32) as usize;
        if n==0 || n+10>=path.len() { return 0; }
        let tail:Vec<u16>="\\d3d9.dll\0".encode_utf16().collect();
        path[n..n+tail.len()].copy_from_slice(&tail);
        LoadLibraryW(path.as_ptr()) as usize
    })
}
struct Code { bytes: Vec<u8>, base: usize }
impl Code {
    fn new() -> Self {
        let base=unsafe { VirtualAlloc(std::ptr::null_mut(),4096,0x3000,0x04) } as usize;
        Self { bytes:Vec::new(),base }
    }
    fn raw(&mut self,b:&[u8]) {self.bytes.extend_from_slice(b);}
    fn u32(&mut self,v:u32) {self.raw(&v.to_le_bytes());}
    fn jump(&mut self,target:usize) {self.raw(&[0xe9]);let next=self.base+self.bytes.len()+4;self.u32(target.wrapping_sub(next) as u32);}
    fn conditional(&mut self,opcode:u8,target:usize) {self.raw(&[0x0f,opcode]);let next=self.base+self.bytes.len()+4;self.u32(target.wrapping_sub(next) as u32);}
    fn finish(self) -> Result<usize,&'static str> {
        if self.base==0 { return Err("No se pudo reservar memoria para el parche"); }
        unsafe {
            std::ptr::copy_nonoverlapping(self.bytes.as_ptr(),self.base as *mut u8,self.bytes.len());
            let mut old=0;
            if VirtualProtect(self.base as Handle,4096,0x20,&mut old)==0 {return Err("No se pudo preparar el código");}
            FlushInstructionCache(GetCurrentProcess(),self.base as *const c_void,self.bytes.len());
        }
        Ok(self.base)
    }
}
unsafe fn write(address:usize,bytes:&[u8]) -> Result<(), &'static str> {
    let mut old=0;
    if VirtualProtect(address as Handle,bytes.len(),0x40,&mut old)==0 {return Err("No se pudo aplicar el parche");}
    std::ptr::copy_nonoverlapping(bytes.as_ptr(),address as *mut u8,bytes.len());
    let mut discard=0;
    VirtualProtect(address as Handle,bytes.len(),old,&mut discard);
    FlushInstructionCache(GetCurrentProcess(),address as *const c_void,bytes.len());
    Ok(())
}
fn jump_bytes(from:usize,to:usize,length:usize) -> Vec<u8> {
    let mut b=vec![0x90;length];b[0]=0xe9;b[1..5].copy_from_slice(&(to.wrapping_sub(from+5) as u32).to_le_bytes());b
}
fn renderer_code(base: usize) -> Result<usize,&'static str> {
    let mut c=Code::new();
    for (id,w,h) in MODES {
        c.raw(&[0x83,0xf8,id as u8,0x75,13,0xc7,0x07]);c.u32(w);
        c.raw(&[0xc7,0x06]);c.u32(h);c.raw(&[0xc3]);
    }
    c.raw(&[0x83,0xf8,9]);c.conditional(0x87,base+0x1a31af);c.jump(base+0x1a3100);c.finish()
}
fn ui_code(base: usize) -> Result<usize,&'static str> {
    let mut c=Code::new();
    for (id,w,h) in MODES {
        c.raw(&[0x83,0x3a,id as u8,0x75,13,0xc7,0x01]);c.u32(w);
        c.raw(&[0xc7,0x00]);c.u32(h);c.raw(&[0xc3]);
    }
    c.raw(&[0x8b,0x12,0x83,0xc2,0xfe]);c.jump(base+0x08bdf4);c.finish()
}
fn inverse_code(base: usize) -> Result<usize,&'static str> {
    let mut c=Code::new();
    for (id,w,h) in MODES {
        c.raw(&[0x81,0xfa]);c.u32(w);c.raw(&[0x75,15,0x81,0xfe]);c.u32(h);
        c.raw(&[0x75,7,0xc7,0x00]);c.u32(id);c.raw(&[0xc3]);
    }
    c.raw(&[0x83,0x08,0xff,0x81,0xfa,0x80,2,0,0]);c.jump(base+0x1a31e7);c.finish()
}
fn animation_clock(clock:usize)->Result<usize,&'static str> {
    let mut c=Code::new();
    // Campaign selection pulses have a 1500 ms period. The original code feeds
    // system uptime directly into a signed 32-bit trig-table index, overflowing
    // after about 10.36 days. Reduce ONLY this animation's clock, not game time.
    c.raw(&[0xb8]);c.u32(clock as u32);c.raw(&[0xff,0xd0,0x31,0xd2,0xb9]);
    c.u32(1500);c.raw(&[0xf7,0xf1,0x89,0xd0,0xc3]);c.finish()
}
fn camera_bridge(mode:usize,fov:usize,get_module:usize,angle:u32,continuation:usize)->Result<usize,&'static str> {
    let mut c=Code::new();
    // No Rust, allocation, formatting, or floating-point arithmetic in the render hook.
    // The game can call this with a live x87 stack. Save it, give the Win32 call a
    // clean FPU environment, then restore the complete original x87/SSE state.
    c.raw(&[0x9c,0x60,0x89,0xe3,0x81,0xec,0x10,2,0,0,0x83,0xe4,0xf0]);
    c.raw(&[0x0f,0xae,0x04,0x24,0xdb,0xe3,0x83,0xec,12,0x68]);
    c.u32(MENU_SCENE.as_ptr() as usize as u32);
    c.raw(&[0xb8]);c.u32(get_module as u32);c.raw(&[0xff,0xd0,0x83,0xc4,12]);
    let mut original_branches=Vec::new();
    c.raw(&[0x85,0xc0]);original_branches.push(c.bytes.len());c.conditional(0x85,0);
    // The loading ring renders before MenuScene.dll is loaded. A missing menu
    // module alone does NOT imply a running battle: require an enabled camera.
    c.raw(&[0x80,0x3e,0]);original_branches.push(c.bytes.len());c.conditional(0x84,0);
    c.raw(&[0x80,0x7e,0x2a,0]);original_branches.push(c.bytes.len());c.conditional(0x85,0);
    c.raw(&[0xa1]);c.u32(mode as u32);
    c.raw(&[0x83,0xf8,8]);let wide8=c.bytes.len();c.conditional(0x84,0);
    c.raw(&[0x83,0xe8,10,0x83,0xf8,2]);original_branches.push(c.bytes.len());c.conditional(0x87,0);
    let wide_label=c.bytes.len();c.raw(&[0xba]);c.u32(angle);
    let to_store=c.bytes.len();c.jump(0);
    let original_label=c.bytes.len();c.raw(&[0xba]);c.u32(std::f32::consts::FRAC_PI_4.to_bits());
    let store_label=c.bytes.len();c.raw(&[0x89,0x15]);c.u32(fov as u32);
    c.raw(&[0x0f,0xae,0x0c,0x24,0x89,0xdc,0x61,0x9d]);
    c.raw(&[0x55,0x8b,0xec,0x83,0xec,0x24]);c.jump(continuation);
    for branch in original_branches {
        c.bytes[branch+2..branch+6].copy_from_slice(&((original_label as i32)-(branch as i32+6)).to_le_bytes());
    }
    c.bytes[wide8+2..wide8+6].copy_from_slice(&((wide_label as i32)-(wide8 as i32+6)).to_le_bytes());
    c.bytes[to_store+1..to_store+5].copy_from_slice(&((store_label as i32)-(to_store as i32+5)).to_le_bytes());
    c.finish()
}
unsafe fn install() -> Result<(), &'static str> {
    let file=std::fs::read(game_exe()).map_err(|_|"No se pudo verificar Rings.exe")?;
    let hash=file.iter().fold(0xcbf29ce484222325u64,|h,b|(h^(*b as u64)).wrapping_mul(0x100000001b3));
    if file.len()!=3481600 || hash!=0x3017eefba0bda303 {return Err("Versión no compatible: se requiere Rings.exe español 1.00.0010");}
    let base=GetModuleHandleW(std::ptr::null()) as usize;
    // Validate every site before changing any instruction.
    let sites:&[(usize,&[u8])]=&[
        (0x1a30f7,&[0x83,0xf8,9,0x0f,0x87,0xaf,0,0,0]),
        (0x08bdef,&[0x8b,0x12,0x83,0xc2,0xfe]),
        (0x1a31de,&[0x83,0x08,0xff,0x81,0xfa,0x80,2,0,0]),
        (0x1a51d6,&[0x83,0xfb,10]),(0x1a51dd,&[0x83,0xfb,10]),
        (0x1547e1,&[0x55,0x8b,0xec,0x83,0xec,0x24]),
        (0x0870b5,&[0xff,0x15,0x8c,0x31,0x6f,0]),
    ];
    for (rva,expected) in sites {
        if std::slice::from_raw_parts((base+rva) as *const u8,expected.len())!=*expected {return Err("Las instrucciones no coinciden; no se aplicó el parche");}
    }
    // Rasterizer: EAX=mode, EDI=&width, ESI=&height. Preserve the special window modes 0 and 1.
    let renderer=renderer_code(base)?;
    // Options UI: EDX=&mode, ECX=&width, EAX=&height.
    let ui=ui_code(base)?;
    // INI and resize conversion: EDX=width, ESI=height, EAX=&mode.
    let inverse=inverse_code(base)?;
    let clock=animation_clock(GetTickCount as *const () as usize)?;
    let mut clock_call=jump_bytes(base+0x0870b5,clock,6);
    clock_call[0]=0xe8; // Replace CALL [GetTickCount], retaining its return address.
    let ini=game_exe().with_file_name("WotR-HD.ini");
    let text=std::fs::read_to_string(ini).unwrap_or_default();
    let degrees=text.lines().find_map(|line| {
        let (name,value)=line.split_once('=')?;
        if name.trim().eq_ignore_ascii_case("CameraDegrees") {value.trim().parse::<f32>().ok()} else {None}
    }).filter(|x|x.is_finite()).unwrap_or(60.0).clamp(45.0,75.0);
    let camera=camera_bridge(base+0x349a58,base+0x746b30,GetModuleHandleW as *const () as usize,
        degrees.to_radians().to_bits(),base+0x1547e7)?;
    // Display enumeration visits modes 2..13, retaining every original mode (2..10).
    let mut changes=vec![
        (base+sites[0].0,jump_bytes(base+sites[0].0,renderer,9)),
        (base+sites[1].0,jump_bytes(base+sites[1].0,ui,5)),
        (base+sites[2].0,jump_bytes(base+sites[2].0,inverse,9)),
        (base+0x1a51d8,vec![13]),(base+0x1a51df,vec![13]),
        (base+0x0870b5,clock_call),
    ];
    let camera_enabled=text.lines().find_map(|line| {
        let (name,value)=line.split_once('=')?;
        name.trim().eq_ignore_ascii_case("EnableCamera").then(||value.trim()!="0")
    }).unwrap_or(false);
    if camera_enabled { changes.push((base+0x1547e1,jump_bytes(base+0x1547e1,camera,6))); }
    let mut applied:Vec<(usize,Vec<u8>)>=Vec::new();
    for (address,bytes) in changes {
        let original=std::slice::from_raw_parts(address as *const u8,bytes.len()).to_vec();
        if let Err(error)=write(address,&bytes) {
            for (address,original) in applied.into_iter().rev() {let _=write(address,&original);}
            return Err(error);
        }
        applied.push((address,original));
    }
    log(&format!("WotR HD v0.1.4: 1080p/720p/1440p; cámara={camera_enabled}, grados={degrees:.0}; cámara original durante carga y menús; contador de campaña corregido."));
    Ok(())
}
fn ensure_patch() {
    PATCH.get_or_init(||unsafe { match install() {Ok(())=>true,Err(e)=>{log(e);false}} });
}
#[no_mangle]
pub unsafe extern "system" fn Direct3DCreate9(sdk:u32) -> Handle {
    ensure_patch();
    let module=real_d3d();if module==0 {log("No se pudo cargar Direct3D 9 del sistema");return std::ptr::null_mut();}
    let f=GetProcAddress(module as Handle,b"Direct3DCreate9\0".as_ptr());if f.is_null(){return std::ptr::null_mut();}
    let f:unsafe extern "system" fn(u32)->Handle=std::mem::transmute(f);f(sdk)
}
#[no_mangle]
pub unsafe extern "system" fn Direct3DCreate9Ex(sdk:u32,out:*mut Handle) -> i32 {
    ensure_patch();let module=real_d3d();if module==0{return 0x80004005u32 as i32;}
    let f=GetProcAddress(module as Handle,b"Direct3DCreate9Ex\0".as_ptr());if f.is_null(){return 0x80004001u32 as i32;}
    let f:unsafe extern "system" fn(u32,*mut Handle)->i32=std::mem::transmute(f);f(sdk,out)
}
#[no_mangle]
pub unsafe extern "system" fn DllMain(module:Handle,reason:u32,_reserved:Handle)->i32 {
    if reason==1 {
        DisableThreadLibraryCalls(module);
        // The game parses Rings.ini BEFORE its first Direct3DCreate9 call. Patch the
        // converters now so a saved widescreen mode is recognized on the next launch.
        // No graphics DLL or worker thread is loaded from this entry point.
        ensure_patch();
    }1
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::arch::asm;
    static MOCK_MENU: std::sync::atomic::AtomicBool=std::sync::atomic::AtomicBool::new(false);
    static MOCK_TICKS: std::sync::atomic::AtomicU32=std::sync::atomic::AtomicU32::new(0);
    unsafe extern "system" fn mock_ticks()->u32 {MOCK_TICKS.load(std::sync::atomic::Ordering::Relaxed)}
    #[test]
    fn campaign_animation_clock_handles_long_uptime_and_wraparound() {
        let code=animation_clock(mock_ticks as *const () as usize).unwrap();
        let clock:unsafe extern "system" fn()->u32=unsafe{std::mem::transmute(code)};
        for ticks in [0u32,1499,1500,864_000_000,1_073_015_466,u32::MAX] {
            MOCK_TICKS.store(ticks,std::sync::atomic::Ordering::Relaxed);
            let t=unsafe{clock()};assert_eq!(t,ticks%1500);
            // The actual failure used a negative signed trig-table index.
            let table_index=((t as f32)*0.00418879045*572.9578247) as i32;
            assert!((0..3600).contains(&table_index));
        }
    }
    unsafe extern "system" fn mock_module(_: *const u16)->Handle {
        MOCK_MENU.load(std::sync::atomic::Ordering::Relaxed) as usize as Handle
    }
    #[repr(align(16))]
    struct Fpu([u8;512]);
    #[test]
    fn native_camera_preserves_live_fpu_and_selects_the_correct_view() {
        let mut ending=Code::new();ending.raw(&[0xc9,0xc3]);let ending=ending.finish().unwrap();
        let wide=60f32.to_radians().to_bits();let original=std::f32::consts::FRAC_PI_4.to_bits();
        for (mode,menu,cinematic,enabled,expected) in [(10u32,false,false,true,wide),(10,true,false,true,original),
            (10,false,true,true,original),(9,false,false,true,original),(12,false,false,true,wide),
            (8,false,false,true,wide),(10,false,false,false,original)] {
            MOCK_MENU.store(menu,std::sync::atomic::Ordering::Relaxed);
            let mut fov=0u32;let mut camera=[0u8;64];camera[0]=enabled as u8;camera[0x2a]=cinematic as u8;
            let hook=camera_bridge(&mode as *const u32 as usize,&mut fov as *mut u32 as usize,
                mock_module as *const () as usize,wide,ending).unwrap();
            let mut saved=Fpu([0;512]);let mut before=Fpu([0;512]);let mut after=Fpu([0;512]);
            unsafe {
                asm!("fxsave [{saved}]","fninit","fld1","fld1","fld1","fld1","fld1","fld1",
                    "fxsave [{before}]",saved=in(reg)saved.0.as_mut_ptr(),before=in(reg)before.0.as_mut_ptr());
                asm!("push {hook}","push esi","mov esi, ecx","call dword ptr [esp + 4]","pop esi","add esp,4",
                    hook=in(reg)hook,inout("ecx")camera.as_ptr()=>_,clobber_abi("C"));
                asm!("fxsave [{after}]","fxrstor [{saved}]",after=in(reg)after.0.as_mut_ptr(),saved=in(reg)saved.0.as_ptr());
            }
            assert_eq!(&before.0[..5],&after.0[..5],"FPU control/status/tag");
            assert_eq!(&before.0[32..160],&after.0[32..160],"live x87 values");
            assert_eq!(fov,expected,"mode={mode} menu={menu} cinematic={cinematic}");
        }
    }
    /// Execute the actual generated x86 instructions with the game's register ABI.
    /// This catches incorrect conditional branch lengths and reversed width/height pointers.
    #[test]
    fn generated_hooks_round_trip_new_modes() {
        let renderer=renderer_code(0).unwrap();
        let ui=ui_code(0).unwrap();
        let inverse=inverse_code(0).unwrap();
        for (id,width,height) in MODES {
            let (mut w,mut h)=(0u32,0u32);
            unsafe {
                asm!("push {fun}","push esi","push edi","mov edi, ecx","mov esi, edx",
                    "call dword ptr [esp + 8]","pop edi","pop esi","add esp, 4",
                    fun=in(reg)renderer,inout("eax")id=>_,inout("ecx")&mut w=>_,inout("edx")&mut h=>_,clobber_abi("C"));
            }
            assert_eq!((w,h),(width,height),"renderer mode {id}");
            w=0;h=0;
            unsafe {
                asm!("push {fun}","call dword ptr [esp]","add esp, 4",
                    fun=in(reg)ui,inout("eax")&mut h=>_,inout("ecx")&mut w=>_,inout("edx")&id=>_,clobber_abi("C"));
            }
            assert_eq!((w,h),(width,height),"options mode {id}");
            let mut result=u32::MAX;
            unsafe {
                asm!("push {fun}","push esi","mov esi, ecx","call dword ptr [esp + 4]","pop esi","add esp, 4",
                    fun=in(reg)inverse,inout("eax")&mut result=>_,inout("ecx")height=>_,inout("edx")width=>_,clobber_abi("C"));
            }
            assert_eq!(result,id,"INI mode {id}");
        }
    }
}
