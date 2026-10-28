//! Resolution-only engine component for the exact StarCraft 1.16.1 executable.
//! Patch map provenance: ../../THIRD-PARTY.txt. No gameplay hooks are installed.
#![allow(non_snake_case, static_mut_refs)]
#[path="../../common.rs"] mod win;
use win::*;
use std::{ptr, mem, slice};

static mut WIDTH:usize=640;
static mut HEIGHT:usize=480;
static mut GRID:usize=0;
static mut GRID_LEN:usize=0;
static mut COMPOSE:usize=0;
static mut MOVIE_PLAY:usize=0;
static mut VIDEO_ACTIVE:bool=false;
static mut COPY:usize=0;
static mut STORM:usize=0;
static mut IN_GAME:bool=false;
static mut INITIALIZED:bool=false;
static mut PRESENTS:u32=0;
static mut RESTORE:usize=0;
static mut MENU_VERIFIED:bool=false;
static mut HUD_VERIFIED:bool=false;
static mut WINDOW_PROC:usize=0;
static mut MENU_PIXELS:usize=0;
static mut MENU_HAS_IMAGE:bool=false;
static mut MENU_FULL_ROOT:bool=false;
static mut MENU_XMAP:usize=0;
static mut MENU_YMAP:usize=0;
static mut MENU_SCALE_LOGGED:bool=false;
static mut MENU_STAGE:usize=0;
static mut STORM_LOCK:usize=0;
static mut STORM_UNLOCK:usize=0;
static mut MENU_CURSOR:usize=0;
static mut MENU_CAPTURED:bool=false;
static mut PRESENTED_FRAME:bool=false;
static mut DIAGNOSTICS:bool=false;
static mut DIAGNOSTIC_FRAME:u32=0;
static mut MENU_OUTPUT:usize=0;
static mut DIALOG_DRAW:usize=0;
static mut SKIN_LOGGED:bool=false;
static mut SKIN_STYLE:u8=0;
static mut GAME_FRAME_READY:bool=false;
static mut HUD_BACKGROUND:usize=0;
static mut HUD_SOURCE:usize=0;
static mut HUD_MASK:usize=0;
static mut UI_POINTER_DELTA:(i32,i32)=(0,0);
static mut UI_MODAL:bool=false;
static mut TOOLTIP_DRAW:usize=0;
static mut TOOLTIP_LOGGED:bool=false;
static mut HUD_ART:usize=0;
static mut UI_BUILDING:bool=false;
static mut BACKGROUND_LOAD:usize=0;
static mut AI_TITLE:bool=false;
static mut TITLE_ORIGINAL:usize=0;
static mut TITLE_SCALED:usize=0;
static mut TITLE_READY:bool=false;
static mut MODAL_SCALE:usize=100;
static TITLE_AI:&[u8]=include_bytes!("../../assets/title-ai-1440x1080.idx");

unsafe fn readable(at:usize,len:usize)->bool {
    if at<0x10000 || len==0 {return false}
    let mut m:MemoryInfo=mem::zeroed();
    VirtualQuery(at as Handle,&mut m,mem::size_of::<MemoryInfo>())!=0 && m.state==0x1000 && m.protect & 0x101 == 0 && at.checked_add(len).is_some_and(|end|end<=(m.base as usize).saturating_add(m.size))
}
unsafe fn read32(at:usize)->u32 { ptr::read_unaligned(at as *const u32) }
unsafe fn write32(at:usize,value:u32) {ptr::write_unaligned(at as *mut u32,value)}
unsafe fn alloc(n:usize,execute:bool)->Result<usize,String> {
    let p=VirtualAlloc(NULL,n,0x3000,if execute {0x40}else{4});
    if p.is_null() {Err("No hay memoria para el parche".into())}else{Ok(p as usize)}
}
unsafe fn write(at:usize,bytes:&[u8])->Result<(),String> {
    let mut old=0;
    if VirtualProtect(at as Handle,bytes.len(),0x40,&mut old)==0 {return Err(format!("VirtualProtect {at:08x}"))}
    ptr::copy_nonoverlapping(bytes.as_ptr(),at as *mut u8,bytes.len());
    let ok=FlushInstructionCache(GetCurrentProcess(),at as Handle,bytes.len());
    let mut ignored=0;
    if VirtualProtect(at as Handle,bytes.len(),old,&mut ignored)==0 || ok==0 {return Err("No se pudo finalizar la escritura".into())}
    if slice::from_raw_parts(at as *const u8,bytes.len())!=bytes {return Err("El parche no coincide después de escribirlo".into())}
    Ok(())
}
fn jump(from:usize,to:usize)->[u8;5] {
    let mut j=[0xe9,0,0,0,0]; j[1..].copy_from_slice(&(to.wrapping_sub(from+5) as u32).to_le_bytes()); j
}
unsafe fn hook(at:usize,dest:usize,expect:&[u8])->Result<usize,String> {
    if !readable(at,expect.len()) || slice::from_raw_parts(at as *const u8,expect.len())!=expect {return Err(format!("Firma del hook {at:08x} incompatible"))}
    let tramp=alloc(expect.len()+5,true)?;
    write(tramp,expect)?;
    write(tramp+expect.len(),&jump(tramp+expect.len(),at+expect.len()))?;
    let mut patch=vec![0x90;expect.len()]; patch[..5].copy_from_slice(&jump(at,dest));
    write(at,&patch)?; Ok(tramp)
}
unsafe fn callback_stub(callback:usize,continuation:usize)->Result<usize,String> {
    let stub=alloc(32,true)?;
    // VC6 game callers only promise four-byte stack alignment. Rust's i686
    // ABI needs 16 at a call site; EBX holds the saved stack across the callback.
    let mut code=vec![0x9c,0x60,0x89,0xe3,0x83,0xe4,0xf0,0xb8];
    code.extend_from_slice(&(callback as u32).to_le_bytes());
    code.extend_from_slice(&[0xff,0xd0,0x89,0xdc,0x61,0x9d]);
    code.extend_from_slice(&jump(stub+code.len(),continuation));
    write(stub,&code)?;Ok(stub)
}
unsafe fn abi_stub(callback:usize,args:u8,stdcall:bool)->Result<usize,String> {
    let stub=alloc(64,true)?;
    let mut code=vec![0x55,0x89,0xe5,0x83,0xe4,0xf0]; // save EBP; align ESP
    let pad=(16-(args*4)%16)%16;
    if pad>0 {code.extend_from_slice(&[0x83,0xec,pad]);}
    for i in (0..args).rev() {code.extend_from_slice(&[0xff,0x75,8+i*4]);}
    code.push(0xb8);code.extend_from_slice(&(callback as u32).to_le_bytes());
    code.extend_from_slice(&[0xff,0xd0,0x89,0xec,0x5d]); // restore caller frame
    if stdcall {code.extend_from_slice(&[0xc2,args*4,0]);}else{code.push(0xc3);}
    write(stub,&code)?;Ok(stub)
}
unsafe fn fastcall_stub(callback:usize,stack_args:u8)->Result<usize,String> {
    // ECX and EDX carry the first two arguments; the remaining ones stay on
    // the stack. Align the callback without altering those register arguments.
    let stub=alloc(64,true)?;
    let mut code=vec![0x55,0x89,0xe5,0x83,0xe4,0xf0];
    let pad=(16-(stack_args*4)%16)%16;
    if pad>0 {code.extend_from_slice(&[0x83,0xec,pad]);}
    for i in (0..stack_args).rev() {code.extend_from_slice(&[0xff,0x75,8+i*4]);}
    code.push(0xb8);code.extend_from_slice(&(callback as u32).to_le_bytes());
    code.extend_from_slice(&[0xff,0xd0,0x89,0xec,0x5d,0xc2,stack_args*4,0]);
    write(stub,&code)?;Ok(stub)
}
unsafe fn log(text:&str) {
    use std::io::Write;
    if let Ok(mut f)=std::fs::OpenOptions::new().create(true).append(true).open("rust-resolution.log") {let _=writeln!(f,"{text}");}
}
unsafe fn update(dlg:usize) {
    // The original function uses EAX for its argument and preserves EBX/ESI/EDI.
    std::arch::asm!("call {f}",f=in(reg) 0x0041c400usize,inlateout("eax") dlg=>_,lateout("ecx") _,lateout("edx") _);
}
unsafe fn name(dlg:usize)->[u8;20] {
    let mut out=[0u8;20]; let p=read32(dlg+0x14) as usize;
    for (i,ch) in out.iter_mut().enumerate().take(19) {
        if !readable(p+i,1) {break} let c=*(p as *const u8).add(i); if c==0 {break} *ch=c;
    }
    out
}
fn is(n:&[u8;20],s:&[u8])->bool {&n[..s.len()]==s && n[s.len()]==0}
fn scaled_root(n:&[u8;20])->bool {
    [b"Minimap".as_slice(),b"Stat_F10",b"StatBtn",b"StatData",b"StatPort",b"StatFluf",b"StatRes"].iter().any(|s|is(n,s))
}
fn modal_root(n:&[u8;20])->bool {!scaled_root(n) && !is(n,b"TextBox") && !is(n,b"StatLB")}
#[derive(Clone,Copy)]
struct Moved {at:usize,bounds:[i16;4],dx:i16,dy:i16}
static mut MOVED:[Moved;128]=[Moved{at:0,bounds:[0;4],dx:0,dy:0};128];

unsafe fn move_root(dlg:usize,dx:i16,dy:i16,wait_surface:bool) {
    let bounds=ptr::read_unaligned((dlg+4) as *const [i16;4]);
    if let Some(i)=MOVED.iter().position(|s|s.at==dlg) {
        let old=MOVED[i]; let b=old.bounds;
        if bounds==[b[0]+old.dx,b[1]+old.dy,b[2]+old.dx,b[3]+old.dy] {return}
        MOVED[i].at=0;
    }
    if wait_surface && read32(dlg+0x3a)==0 && read32(dlg+0x10)==0 {return}
    if let Some(slot)=MOVED.iter_mut().find(|s|s.at==0) {
        *slot=Moved{at:dlg,bounds,dx,dy}; update(dlg);
        ptr::write_unaligned((dlg+4) as *mut [i16;4],[bounds[0]+dx,bounds[1]+dy,bounds[2]+dx,bounds[3]+dy]);
        update(dlg);
    }
}

unsafe fn frame() {
    if VIDEO_ACTIVE {return}
    let mut live=[0usize;128]; let mut count=0;
    let mut d=read32(0x006d5e34) as usize;
    let mut in_game=false;
    while d!=0 && count<128 && readable(d,0x56) {
        if live[..count].contains(&d) {break}
        live[count]=d; count+=1;
        if is(&name(d),b"StatBtn") {in_game=true}
        d=read32(d) as usize;
    }
    if in_game!=IN_GAME { IN_GAME=in_game; SKIN_LOGGED=false;GAME_FRAME_READY=false; MENU_HAS_IMAGE=false; MENU_FULL_ROOT=false; MOVED.fill(Moved{at:0,bounds:[0;4],dx:0,dy:0}); log(if in_game {"Entrando en partida · barra ampliada a la izquierda"}else{"Volviendo a los menús ampliados"}); }
    UI_MODAL=in_game && live[..count].iter().any(|&d| {
        let n=name(d);read32(d+0x18)&8!=0 && !scaled_root(&n) && !is(&n,b"TextBox") && !is(&n,b"StatLB")
    });
    MODAL_SCALE=if UI_MODAL && HEIGHT>=720 {150}else{100};
    for &dlg in &live[..count] {
        if UI_MODAL && read32(dlg+0x18)&8!=0 && modal_root(&name(dlg)) {
            let b=ptr::read_unaligned((dlg+4) as *const [i16;4]);
            let w=(b[2] as i32-b[0] as i32+1).max(1) as usize;
            let h=(b[3] as i32-b[1] as i32+1).max(1) as usize;
            MODAL_SCALE=MODAL_SCALE.min(WIDTH*100/w).min(HEIGHT*100/h).max(100);
        }
    }
    for slot in MOVED.iter_mut() {if slot.at!=0 && !live[..count].contains(&slot.at) {slot.at=0}}
    if !in_game {
        MENU_FULL_ROOT=false;
        let dx=0i16;let dy=0i16;
        for &dlg in &live[..count] {
            let b=ptr::read_unaligned((dlg+4) as *const [i16;4]);
            let full=b[2] as i32-b[0] as i32+1>=640 && b[3] as i32-b[1] as i32+1>=480;
            MENU_FULL_ROOT|=full;
            // Native menus remain at their original logical coordinates.
            // Only the final presentation is scaled and centred.
            if !AI_TITLE && read32(dlg+0x18)&8!=0 {update(dlg);}
            let moved=ptr::read_unaligned((dlg+4) as *const [i16;4]);
            if full && moved[0]==dx && moved[1]==dy && !MENU_VERIFIED {
                MENU_VERIFIED=true;
                log(&format!("MENU centro verificado: ({dx},{dy}) en {}x{}",WIDTH,HEIGHT));
            }
        }
        *(0x006cef51 as *mut u8)|=0x20;
        return
    }
    // Whole-frame presentation needs a fully composed frame, including fog and sprites.
    ptr::write_bytes(GRID as *mut u8,1,GRID_LEN);
    *(0x006cefb5 as *mut u8)|=1;
    for &dlg in &live[..count] {
        let flags=read32(dlg+0x18);
        if flags&0x10000000==0 {write32(dlg+0x18,flags|0x10000000); update(dlg)}
        let n=name(dlg);
        if ![b"Minimap".as_slice(),b"TextBox",b"Stat_F10",b"StatBtn",b"StatData",b"StatPort",b"StatFluf",b"StatRes"].iter().any(|s|is(&n,s)) {
            if !is(&n,b"StatLB") {
                let b=MOVED.iter().find(|s|s.at==dlg).map(|s|s.bounds).unwrap_or_else(||ptr::read_unaligned((dlg+4) as *const [i16;4]));
                if b[2]>b[0] && b[3]>b[1] {
                    let (dx,dy)=popup_offset(WIDTH,HEIGHT,b);
                    let known=MOVED.iter().any(|s|s.at==dlg);
                    move_root(dlg,dx,dy,true);
                    if !known && MOVED.iter().any(|s|s.at==dlg) {log(&format!("Ventana de partida centrada: {}",String::from_utf8_lossy(&n)));}
                }
            }
            continue
        }
        let bounds=ptr::read_unaligned((dlg+4) as *const [i16;4]);
        let original=MOVED.iter().find(|s|s.at==dlg).map(|s|s.bounds).unwrap_or(bounds);
        let end=n.iter().position(|&c|c==0).unwrap_or(n.len());
        let dx=hud_offset(WIDTH,&n[..end],original[0],original[2]);
        let dy=if is(&n,b"StatRes") {0}else{(HEIGHT-480) as i16};
        move_root(dlg,dx,dy,true);
    }
    // Native dropdowns can extend beyond the control's cached rectangle.
    // Redraw each visible root so those transient controls are included every
    // frame, even when the only change underneath is animated text.
    for &dlg in &live[..count] {if read32(dlg+0x18)&8!=0 {update(dlg);}}
    *(0x006cef51 as *mut u8)|=0x20; // cursor
    *(0x006cef65 as *mut u8)|=0x20; // tooltip
    if !HUD_VERIFIED {
        let card=MOVED.iter().find(|s|s.at!=0 && is(&name(s.at),b"StatBtn"));
        let mini=MOVED.iter().find(|s|s.at!=0 && is(&name(s.at),b"Minimap"));
        let data=MOVED.iter().find(|s|s.at!=0 && is(&name(s.at),b"StatData"));
        if let (Some(card),Some(mini),Some(data))=(card,mini,data) {
            let cb=ptr::read_unaligned((card.at+4) as *const [i16;4]);
            let mb=ptr::read_unaligned((mini.at+4) as *const [i16;4]);
            let db=ptr::read_unaligned((data.at+4) as *const [i16;4]);
            if cb[2] as usize==639 && mb[0]==0 && db[0] as usize==138 {
                HUD_VERIFIED=true;log(&format!("HUD anclas verificadas: minimapa {}, selección {}, órdenes {}..{}",mb[0],db[0],cb[0],cb[2]));
            }
        }
    }
}
unsafe extern "C" fn compose() {
    if VIDEO_ACTIVE {let f:unsafe extern "C" fn()=mem::transmute(COMPOSE);f();return}
    frame();
    MENU_CAPTURED=false;
    GAME_FRAME_READY=false;
    let cursor=ptr::read_unaligned(0x006cef52 as *const [i16;2]);
    let tooltip=ptr::read_unaligned(0x006cef66 as *const [i16;2]);
    let f:unsafe extern "C" fn()=mem::transmute(COMPOSE);f();
    // The input router uses logical HUD coordinates. Presentation alone moves
    // the cursor/tooltip back to their physical positions for this layer walk.
    if IN_GAME {
        ptr::write_unaligned(0x006cef52 as *mut [i16;2],cursor);
        ptr::write_unaligned(0x006cef66 as *mut [i16;2],tooltip);
    }
    // Scale after the complete layer walk, not during cursor restore-under:
    // that helper can run before every menu surface has been composited.
    if IN_GAME {if GAME_FRAME_READY {publish_frame();}}else if MENU_FULL_ROOT || PRESENTED_FRAME {
        // With a hidden cursor the native restore-under callback does not run.
        // Capture the clean foreground now instead of reusing its last cursor.
        if !MENU_CAPTURED {capture_menu_foreground();}
        menu_primary();
    }
    if DIAGNOSTICS {
        DIAGNOSTIC_FRAME=DIAGNOSTIC_FRAME.wrapping_add(1);
        if DIAGNOSTIC_FRAME%120==0 {
            let fb=read32(0x006ceff4) as usize;
            let pixels=|at:usize,n:usize|if readable(at,n) {slice::from_raw_parts(at as *const u8,n).iter().filter(|&&p|p!=0).count()}else{0};
            log(&format!("DIAG full={} game={} cursor={} capture={} stage={} foreground={} fb={} force={}",MENU_FULL_ROOT,IN_GAME,*(0x6cef50 as *const u8),MENU_CAPTURED,pixels(MENU_STAGE,WIDTH*HEIGHT),pixels(MENU_CURSOR,640*480),pixels(fb,WIDTH*HEIGHT),*(0x51a0e9 as *const u8)));
        }
    }
}

// Runs before the dialog layer, underneath controls, portrait and cursor.
unsafe extern "C" fn hud_backing() {
    if !IN_GAME {return}
    let buf=read32(0x006ceff4) as usize;
    let art=HUD_ART;
    if !readable(art,640*480) || !readable(buf,WIDTH*HEIGHT) {return}
    let count=overlay_console(slice::from_raw_parts_mut(buf as *mut u8,WIDTH*HEIGHT),WIDTH,HEIGHT,
                              slice::from_raw_parts(art as *const u8,640*480),0,HEIGHT-480);
    SKIN_STYLE=console_style(count);
    finish_console_sides(slice::from_raw_parts_mut(buf as *mut u8,WIDTH*HEIGHT),WIDTH,HEIGHT,
                         slice::from_raw_parts(art as *const u8,640*480),0,HEIGHT-480,SKIN_STYLE);
    if !SKIN_LOGGED {
        SKIN_LOGGED=true;
        let race=match count {86727=>"Terran",89893=>"Zerg",89346=>"Protoss",_=>"arte cargado"};
        log(&format!("Barra completa: {race}, {count} píxeles opacos, transparencia original"));
    }
}

unsafe extern "system" fn draw_dialogs(param:usize,clip:usize) {
    let f:unsafe extern "system" fn(usize,usize)=mem::transmute(DIALOG_DRAW);
    if !IN_GAME {f(param,clip);return}
    let buf=read32(0x006ceff4) as usize;
    if !readable(buf,WIDTH*HEIGHT) {f(param,clip);return}
    let mut roots=Vec::new();let mut d=read32(0x006d5e34) as usize;
    while d!=0 && roots.len()<128 && readable(d,0x56) {
        if roots.iter().any(|&(at,_,_)|at==d) {break}
        roots.push((d,read32(d+0x18)&8,scaled_root(&name(d))));
        d=read32(d) as usize;
    }
    // First compose only the native HUD. Keep its terrain separately, so the
    // logical-size copy is completely removed before the enlarged one is drawn.
    // Build the UI on a clean transparent-key canvas. Drawing it over terrain
    // would bake world pixels into otherwise transparent control rectangles.
    ptr::write_bytes(HUD_SOURCE as *mut u8,0,WIDTH*HEIGHT);
    ptr::write_bytes(HUD_MASK as *mut u8,0,WIDTH*HEIGHT);
    for &(at,visible,hud) in &roots {
        if !hud {write32(at+0x18,read32(at+0x18)&!8);}
        else if visible!=0 {update(at);}
    }
    UI_BUILDING=true;
    write32(0x006ceff4,HUD_SOURCE as u32);
    hud_backing();f(param,clip);
    write32(0x006ceff4,buf as u32);
    // Art transparency and control rectangles form an explicit alpha mask;
    // palette index zero inside a control remains an opaque black pixel.
    let art=HUD_ART;
    let mut silhouette=[480usize;640];
    if readable(art,640*480) {
        for y in 0..480 {for x in 0..640 {
            if *((art+y*640+x) as *const u8)!=0 {
                silhouette[x]=silhouette[x].min(y);
                *((HUD_MASK+(HEIGHT-480+y)*WIDTH+x) as *mut u8)=1;
            }
        }}
        for y in 280..480 {
            for x in 0..console_cap_width(slice::from_raw_parts(art as *const u8,640*480),true,y,SKIN_STYLE) {
                *((HUD_MASK+(HEIGHT-480+y)*WIDTH+640+x) as *mut u8)=1;
            }
        }
    }
    for &(at,visible,hud) in &roots {
        if hud && visible!=0 {
            let b=ptr::read_unaligned((at+4) as *const [i16;4]);
            let resource=is(&name(at),b"StatRes");
            for y in b[1].max(0) as usize..(b[3] as i32+1).clamp(0,HEIGHT as i32) as usize {
                let left=b[0].max(0) as usize;let right=(b[2] as i32+1).clamp(0,WIDTH as i32) as usize;
                if right>left {for x in left..right {
                    // Resource controls are rectangular. Bottom controls follow
                    // the race artwork's silhouette, including black interiors.
                    let local_y=y as i32-(HEIGHT as i32-480);
                    let inside=if resource {*((HUD_SOURCE+y*WIDTH+x) as *const u8)!=0}
                    else if x<640 && (0..480).contains(&local_y) {
                        local_y as usize>=silhouette[x]
                    }else{false};
                    if inside {*((HUD_MASK+y*WIDTH+x) as *mut u8)=1;}
                }}
            }
        }
    }
    if readable(0x006ceb40,1024) {
        let mut pixels=vec![0u8;420*32];
        for y in 0..32 {
            ptr::copy_nonoverlapping((HUD_SOURCE+y*WIDTH+WIDTH-420) as *const u8,pixels.as_mut_ptr().add(y*420),420);
        }
        let alpha=resource_alpha(&pixels,slice::from_raw_parts(0x006ceb40 as *const u8,1024),420,32);
        for y in 0..32 {
            ptr::copy_nonoverlapping(alpha.as_ptr().add(y*420),(HUD_MASK+y*WIDTH+WIDTH-420) as *mut u8,420);
        }
    }
    let scale=hud_scale(HEIGHT);
    let target=slice::from_raw_parts_mut(buf as *mut u8,WIDTH*HEIGHT);
    let source=slice::from_raw_parts(HUD_SOURCE as *const u8,WIDTH*HEIGHT);
    let mask=slice::from_raw_parts(HUD_MASK as *const u8,WIDTH*HEIGHT);
    scale_ui_region(target,source,mask,WIDTH,HEIGHT,0,HEIGHT-480,652,480,
                    0,HEIGHT-480*scale/100,652*scale/100,480*scale/100);
    scale_ui_region(target,source,mask,WIDTH,HEIGHT,WIDTH-420,0,420,32,
                    WIDTH-420*scale/100,0,420*scale/100,32*scale/100);
    // Menus and dropdowns are composed afterwards at their actual screen
    // coordinates. They therefore cover the enlarged HUD and retain native order.
    for &(at,visible,hud) in &roots {
        write32(at+0x18,(read32(at+0x18)&!8)|if hud {0}else{visible});
        if !hud && visible!=0 {update(at);}
    }
    if UI_MODAL {
        // Chat/leaderboard stay at their native screen positions behind popups.
        for &(at,visible,hud) in &roots {
            let show=!hud && !modal_root(&name(at));
            write32(at+0x18,(read32(at+0x18)&!8)|if show {visible}else{0});
            if show && visible!=0 {update(at);}
        }
        f(param,clip);
        ptr::write_bytes(MENU_STAGE as *mut u8,0,WIDTH*HEIGHT);
        ptr::write_bytes(HUD_MASK as *mut u8,0,WIDTH*HEIGHT);
        for &(at,visible,_) in &roots {
            let show=modal_root(&name(at));
            write32(at+0x18,(read32(at+0x18)&!8)|if show {visible}else{0});
            if show && visible!=0 {
                update(at);
                let b=ptr::read_unaligned((at+4) as *const [i16;4]);
                for y in b[1].max(0) as usize..(b[3] as i32+1).clamp(0,HEIGHT as i32) as usize {
                    let l=b[0].max(0) as usize;let r=(b[2] as i32+1).clamp(0,WIDTH as i32) as usize;
                    if r>l {ptr::write_bytes((HUD_MASK+y*WIDTH+l) as *mut u8,1,r-l);}
                }
            }
        }
        write32(0x006ceff4,MENU_STAGE as u32);f(param,clip);write32(0x006ceff4,buf as u32);
        let mut bounds=[WIDTH,HEIGHT,0,0];
        for y in 0..HEIGHT {for x in 0..WIDTH {
            let at=y*WIDTH+x;
            if *((MENU_STAGE+at) as *const u8)!=0 {*((HUD_MASK+at) as *mut u8)=1;}
            if *((HUD_MASK+at) as *const u8)!=0 {
                bounds[0]=bounds[0].min(x);bounds[1]=bounds[1].min(y);
                bounds[2]=bounds[2].max(x);bounds[3]=bounds[3].max(y);
            }
        }}
        if bounds[2]>=bounds[0] && bounds[3]>=bounds[1] {
            scale_modal(slice::from_raw_parts_mut(buf as *mut u8,WIDTH*HEIGHT),
                        slice::from_raw_parts(MENU_STAGE as *const u8,WIDTH*HEIGHT),
                        slice::from_raw_parts(HUD_MASK as *const u8,WIDTH*HEIGHT),WIDTH,HEIGHT,MODAL_SCALE,bounds);
        }
    }else{f(param,clip);}
    UI_BUILDING=false;
    for &(at,visible,_) in &roots {write32(at+0x18,(read32(at+0x18)&!8)|visible);}
    let (dx,dy)=UI_POINTER_DELTA;
    for rect in [0x006cef52usize,0x006cef66] {
        let p=ptr::read_unaligned(rect as *const [i16;2]);
        ptr::write_unaligned(rect as *mut [i16;2],[p[0].saturating_add(dx as i16),p[1].saturating_add(dy as i16)]);
    }
}

unsafe extern "system" fn draw_tooltip(param:usize,clip:usize) {
    let f:unsafe extern "system" fn(usize,usize)=mem::transmute(TOOLTIP_DRAW);
    if !IN_GAME || read32(0x00655c48)==0 || !readable(param,8) {f(param,clip);return}
    let sw=ptr::read_unaligned(param as *const u16) as usize;
    let sh=ptr::read_unaligned((param+2) as *const u16) as usize;
    let pixels=read32(param+4) as usize;
    let r=ptr::read_unaligned(0x006cef66 as *const [i16;4]);
    let (w,h)=((r[2].max(0) as usize).min(sw),(r[3].max(0) as usize).min(sh));
    let buf=read32(0x006ceff4) as usize;
    if w==0 || h==0 || !readable(pixels,sw*sh) || !readable(buf,WIDTH*HEIGHT) {f(param,clip);return}
    UI_BUILDING=true;
    write32(0x006ceff4,HUD_SOURCE as u32);
    f(param,clip); // Preserve native bookkeeping without presenting the small copy.
    write32(0x006ceff4,buf as u32);
    UI_BUILDING=false;
    let (dw,dh)=((w*2).min(WIDTH),(h*2).min(HEIGHT));
    let x=(r[0].max(0) as usize).min(WIDTH-dw);
    let y=(r[1] as i32+h as i32-dh as i32).clamp(0,(HEIGHT-dh) as i32) as usize;
    for row in 0..dh {for col in 0..dw {
        *((buf+(y+row)*WIDTH+x+col) as *mut u8)=*((pixels+(row*h/dh)*sw+col*w/dw) as *const u8);
    }}
    if !TOOLTIP_LOGGED {TOOLTIP_LOGGED=true;log(&format!("Ayuda emergente al 200%: {w}x{h} -> {dw}x{dh}"));}
}

unsafe extern "system" fn ui_hit(x:i32,y:i32)->u32 {
    if !IN_GAME {return 0}
    let art_y=y-(HEIGHT as i32-480);
    let Some(art_x)=hud_art_x(WIDTH,x) else {
        let local=x;
        if (0..480).contains(&art_y) && (-12..652).contains(&local) {
            let art=HUD_ART;
            if readable(art,640*480) {
                let right=local>=640;
                let width=console_cap_width(slice::from_raw_parts(art as *const u8,640*480),right,art_y as usize,SKIN_STYLE);
                let distance=if right {local-639}else{-local};
                return u32::from(distance>0 && distance as usize<=width)
            }
        }
        return 0
    };
    if art_y>=400 {return 1}
    if art_y<read32(0x00596b6c) as i32 {return 0}
    let region=read32(0x006d5e14);
    if region==0 {return 0}
    let f:unsafe extern "system" fn(u32,i32,i32)->i32=mem::transmute(0x00410166usize);
    u32::from(f(region,art_x,art_y)==0)
}

// Redirect all menu primary-surface locks to a private offscreen canvas.
// The real primary receives exactly one final scaled image per composed frame.
unsafe extern "system" fn menu_lock(surface:i32,rect:*const Rect,out:*mut usize,pitch:*mut i32,flags:i32)->i32 {
    if surface==0 && !VIDEO_ACTIVE && (IN_GAME || MENU_FULL_ROOT || PRESENTED_FRAME) && MENU_STAGE!=0 {
        if out.is_null() || pitch.is_null() {return 0}
        let (x,y)=if rect.is_null() {(0,0)}else{
            let r=*rect;
            if r.left<0 || r.top<0 || r.right>WIDTH as i32 || r.bottom>HEIGHT as i32 || r.right<=r.left || r.bottom<=r.top {return 0}
            (r.left as usize,r.top as usize)
        };
        let target=if IN_GAME && !VIDEO_ACTIVE {read32(0x006ceff4) as usize}else{MENU_STAGE};
        if target==0 {return 0}
        *out=target+y*WIDTH+x;*pitch=WIDTH as i32;return 1
    }
    let f:unsafe extern "system" fn(i32,*const Rect,*mut usize,*mut i32,i32)->i32=mem::transmute(STORM_LOCK);
    f(surface,rect,out,pitch,flags)
}
unsafe extern "system" fn menu_unlock(surface:i32,pixels:usize,a:i32,b:i32)->i32 {
    if !VIDEO_ACTIVE && MENU_STAGE!=0 && pixels>=MENU_STAGE && pixels<MENU_STAGE+WIDTH*HEIGHT {return 1}
    if IN_GAME && !VIDEO_ACTIVE {let fb=read32(0x006ceff4) as usize;if fb!=0 && pixels>=fb && pixels<fb+WIDTH*HEIGHT {return 1}}
    let f:unsafe extern "system" fn(i32,usize,i32,i32)->i32=mem::transmute(STORM_UNLOCK);
    f(surface,pixels,a,b)
}
// Native movie dispatcher receives its identifier in EAX rather than the stack.
unsafe fn eax_stub(callback:usize)->Result<usize,String> {
    let p=alloc(48,true)?;
    let mut code=vec![0x55,0x89,0xe5,0x83,0xe4,0xf0,0x83,0xec,0x0c,0x50,0xb8];
    code.extend_from_slice(&(callback as u32).to_le_bytes());
    code.extend_from_slice(&[0xff,0xd0,0x89,0xec,0x5d,0xc3]);write(p,&code)?;Ok(p)
}
unsafe extern "C" fn play_movie(id:u32)->u32 {
    VIDEO_ACTIVE=true;IN_GAME=false;GAME_FRAME_READY=false;MENU_FULL_ROOT=false;
    MENU_HAS_IMAGE=false;AI_TITLE=false;TITLE_READY=false;
    ptr::write_bytes(MENU_STAGE as *mut u8,0,WIDTH*HEIGHT);
    log(&format!("Video nativo: {id}"));
    let result:u32;
    std::arch::asm!("call {target}",target=in(reg) MOVIE_PLAY,inlateout("eax") id=>result,clobber_abi("C"));
    VIDEO_ACTIVE=false;MENU_FULL_ROOT=false;MENU_HAS_IMAGE=false;
    log(&format!("Video terminado: {id}, resultado {result}"));result
}
unsafe fn menu_primary() {
    // Backgrounds and roots can be absent while the next screen/map loads.
    // Keep the last complete primary rather than publishing the cleared canvas.
    if !AI_TITLE && !menu_has_content() {return}
    let (left,top,w,h)=menu_viewport(WIDTH,HEIGHT);
    ptr::write_bytes(MENU_OUTPUT as *mut u8,0,WIDTH*HEIGHT);
    if AI_TITLE {
        if !TITLE_READY {
            ptr::write_bytes(TITLE_SCALED as *mut u8,0,WIDTH*HEIGHT);
            for y in 0..h {
                let sy=y*1080/h;
                for x in 0..w {*((TITLE_SCALED+(top+y)*WIDTH+left+x) as *mut u8)=TITLE_AI[sy*1440+x*1440/w];}
            }
            TITLE_READY=true;
        }
        ptr::copy_nonoverlapping(TITLE_SCALED as *const u8,MENU_OUTPUT as *mut u8,WIDTH*HEIGHT);
        overlay_loading_caption(slice::from_raw_parts_mut(MENU_OUTPUT as *mut u8,WIDTH*HEIGHT),WIDTH,HEIGHT,
                                slice::from_raw_parts(MENU_STAGE as *const u8,WIDTH*HEIGHT),
                                slice::from_raw_parts(TITLE_ORIGINAL as *const u8,640*480));
    }else{
        for y in 0..480 {
            ptr::copy_nonoverlapping((MENU_STAGE+y*WIDTH) as *const u8,(MENU_PIXELS+y*640) as *mut u8,640);
        }
        for i in 0..640*480 {let cursor=*((MENU_CURSOR+i) as *const u8);if cursor!=0 {*((MENU_PIXELS+i) as *mut u8)=cursor;}}
        for y in 0..h {
            let source=MENU_PIXELS+*((MENU_YMAP+y*4) as *const u32) as usize*640;
            let output=MENU_OUTPUT+(top+y)*WIDTH+left;
            for x in 0..w {*((output+x) as *mut u8)=*((source+*((MENU_XMAP+x*4) as *const u32) as usize) as *const u8);}
        }
    }
    publish_frame();MENU_HAS_IMAGE=true;
    if !MENU_SCALE_LOGGED {MENU_SCALE_LOGGED=true;log("Presentación única: menú lógico original, desplegables incluidos");}
}
unsafe fn publish_frame() {
    let lock:unsafe extern "system" fn(i32,*const Rect,*mut usize,*mut i32,i32)->i32=mem::transmute(STORM_LOCK);
    let unlock:unsafe extern "system" fn(i32,usize,i32,i32)->i32=mem::transmute(STORM_UNLOCK);
    let mut dst=0usize;let mut pitch=0;
    if lock(0,ptr::null(),&mut dst,&mut pitch,0)==0 || dst==0 {return}
    if pitch>=WIDTH as i32 && primary_rows()>=HEIGHT {
        for y in 0..HEIGHT {ptr::copy_nonoverlapping((MENU_OUTPUT+y*WIDTH) as *const u8,(dst+y*pitch as usize) as *mut u8,WIDTH);}
        PRESENTED_FRAME=true;
    }
    unlock(0,dst,0,0);
}
unsafe fn capture_menu_foreground() {
    let buf=read32(0x006ceff4) as usize;
    if MENU_CURSOR!=0 && readable(buf,WIDTH*HEIGHT) {
        copy_menu_foreground(slice::from_raw_parts(buf as *const u8,WIDTH*HEIGHT),WIDTH,
                             slice::from_raw_parts_mut(MENU_CURSOR as *mut u8,640*480));
        MENU_CAPTURED=true;
    }
}
unsafe fn menu_has_content()->bool {
    let cursor=if *(0x006cef50 as *const u8)!=0 {
        Some(ptr::read_unaligned(0x006cef40 as *const [i16;4]))
    }else{None};
    menu_content_visible(slice::from_raw_parts(MENU_STAGE as *const u8,WIDTH*HEIGHT),WIDTH,
                         slice::from_raw_parts(MENU_CURSOR as *const u8,640*480),cursor)
}
unsafe extern "system" fn restore_under(bitmap:u32) {
    if !VIDEO_ACTIVE && !IN_GAME && MENU_FULL_ROOT && bitmap==0x006ceff0 {capture_menu_foreground();}
    let f:unsafe extern "system" fn(u32)=mem::transmute(RESTORE);f(bitmap);
}
unsafe extern "system" fn game_window(hwnd:Handle,msg:u32,w:usize,mut l:isize)->isize {
    if !VIDEO_ACTIVE && IN_GAME && (0x200..=0x209).contains(&msg) {
        let x=l as u32 as u16 as i16 as i32;
        let y=(l as u32>>16) as u16 as i16 as i32;
        let (mx,my)=if UI_MODAL {modal_input(WIDTH,HEIGHT,MODAL_SCALE,x,y)}else{hud_pointer(WIDTH,HEIGHT,x,y)};
        UI_POINTER_DELTA=(x-mx,y-my);
        l=((mx as u16 as u32)|((my as u16 as u32)<<16)) as isize;
    }
    if !VIDEO_ACTIVE && !IN_GAME && MENU_FULL_ROOT && MENU_HAS_IMAGE && (0x200..=0x209).contains(&msg) {
        let x=(l as u32 as u16 as i16) as i32;
        let y=((l as u32>>16) as u16 as i16) as i32;
        let (left,top,vw,vh)=menu_viewport(WIDTH,HEIGHT);
        if msg!=0x200 && (x<left as i32 || y<top as i32 || x>=(left+vw) as i32 || y>=(top+vh) as i32) {return 0}
        let (mx,my)=menu_input(WIDTH,HEIGHT,x,y);
        l=((mx as u16 as u32)|((my as u16 as u32)<<16)) as isize;
    }
    let f:unsafe extern "system" fn(Handle,u32,usize,isize)->isize=mem::transmute(WINDOW_PROC);
    f(hwnd,msg,w,l)
}

// Storm ordinal 325 takes eight stack arguments (ret 0x20), including the
// optional output and allocator callback after the dimensions. Forward all of
// them unchanged: truncating this signature corrupts the original call frame.
unsafe extern "system" fn background_loaded(file:*const u8,palette:usize,bits:usize,width:*const u32,height:*const u32,extra:usize,optional:usize,allocator:usize)->i32 {
    let f:unsafe extern "system" fn(*const u8,usize,usize,*const u32,*const u32,usize,usize,usize)->i32=mem::transmute(BACKGROUND_LOAD);
    let result=f(file,palette,bits,width,height,extra,optional,allocator);
    if result==0 || !readable(width as usize,4) || !readable(height as usize,4) {return result}
    if ptr::read_unaligned(width)!=640 || ptr::read_unaligned(height)!=480 {return result}
    let mut name=Vec::new();
    for i in 0..260 {
        if !readable(file as usize+i,1) {break}
        let c=*file.add(i);if c==0 {break}name.push(c.to_ascii_lowercase());
    }
    if name.ends_with(b"title.pcx") || name.ends_with(b"backgnd.pcx") {
        let buf=read32(0x006ceff4) as usize;
        if readable(buf,WIDTH*HEIGHT) {ptr::write_bytes(buf as *mut u8,0,WIDTH*HEIGHT);}
        if MENU_CURSOR!=0 {ptr::write_bytes(MENU_CURSOR as *mut u8,0,640*480);}
        if MENU_STAGE!=0 {ptr::write_bytes(MENU_STAGE as *mut u8,0,WIDTH*HEIGHT);}
        log(&format!("Fondo de menu: {}",String::from_utf8_lossy(&name)));
    }
    AI_TITLE=name.ends_with(b"title.pcx");
    if AI_TITLE {TITLE_READY=false;}
    if AI_TITLE {log("Pantalla inicial restaurada con IA: 1440x1080, paleta original");}
    if AI_TITLE && readable(bits,4) {
        let source=read32(bits) as usize;
        if readable(source,640*480) {ptr::copy_nonoverlapping(source as *const u8,TITLE_ORIGINAL as *mut u8,640*480);}
    }
    if name.ends_with(b"console.pcx") && readable(bits,4) && HUD_ART!=0 {
        let source=read32(bits) as usize;
        if readable(source,640*480) {
            // The game frees its original descriptor after creating the roots.
            // Retain this copy while the loaded race's art is still alive.
            ptr::copy_nonoverlapping(source as *const u8,HUD_ART as *mut u8,640*480);
            log(&format!("Arte de barra conservado: {}",String::from_utf8_lossy(&name)));
        }
    }
    result
}

unsafe fn primary_rows()->usize {
    let primary=read32(STORM+0x5ea84) as usize;
    if !readable(primary,4) {return 0}
    let vt=read32(primary) as usize;
    if !readable(vt+0x58,4) {return 0}
    let f:unsafe extern "system" fn(usize,*mut u8)->i32=mem::transmute(read32(vt+0x58) as usize);
    let mut desc=[0u8;108]; desc[..4].copy_from_slice(&108u32.to_le_bytes());
    if f(primary,desc.as_mut_ptr())!=0 {return 0}
    u32::from_le_bytes(desc[8..12].try_into().unwrap()) as usize
}
unsafe extern "system" fn copy(dst:usize,src:usize,dp:usize,sp:usize,region:usize)->i32 {
    let framebuffer=read32(0x006ceff4) as usize;
    if IN_GAME && dst==framebuffer && src==framebuffer && dp==WIDTH && sp==WIDTH {
        if UI_BUILDING {return 1}
        // Capture while cursor and tooltip are still present. Native restore-under
        // runs afterwards; the completed snapshot is published when compose returns.
        ptr::copy_nonoverlapping(src as *const u8,MENU_OUTPUT as *mut u8,WIDTH*HEIGHT);
        GAME_FRAME_READY=true;PRESENTS=PRESENTS.wrapping_add(1);
        if PRESENTS==1 {log("Partida: fotograma completo único, incluidas escrituras directas y desplegables");}
        return 1
    }
    let f:unsafe extern "system" fn(usize,usize,usize,usize,usize)->i32=mem::transmute(COPY);
    f(dst,src,dp,sp,region)
}

unsafe fn install(w:u32,h:u32)->Result<(),String> {
    DIAGNOSTICS=std::env::var_os("SC_HD_DIAGNOSTICS").is_some();
    if INITIALIZED {return Err("El componente ya está inicializado".into())}
    let base=GetModuleHandleW(ptr::null()) as usize;
    if base!=0x400000 || read32(0x006ceff4)!=0 {return Err("EXE inesperado o vídeo ya inicializado".into())}
    let data=table(w,h).ok_or("Resolución sin tabla verificada")?;
    let (grid_len,mut patches)=parse(data)?;
    // Check every original signature, including hooks, before any game-code write.
    for p in &patches {if !readable(p.va,p.expect.len()) || slice::from_raw_parts(p.va as *const u8,p.expect.len())!=p.expect {return Err(format!("Firma incompatible en {:08x}",p.va))}}
    STORM=GetModuleHandleW(wide("storm.dll").as_ptr()) as usize;
    if STORM==0 {return Err("Storm.dll no está cargado".into())}
    for (at,sig) in [(0x4d4470usize,&[0x55,0x8b,0xec,0x51,0x56][..]),
                    (0x41e280usize,&[0x55,0x8b,0xec,0x83,0xec,0x14][..]),
                    (STORM+0x1a520,&[0x55,0x8b,0xec,0x8b,0x45,0x18][..]),
                    (0x41cb50,&[0x55,0x8b,0xec,0x83,0xec,0x2c][..]),
                    (0x41cf3c,&[0xc2,0x08,0x00][..]),
                      (0x4810f0,&[0x55,0x8b,0xec,0xa1,0x48,0x5c,0x65,0x00][..]),
                    (0x41deb0,&[0x55,0x8b,0xec,0x83,0xec,0x0c][..]),
                    (0x4d1d70,&[0x55,0x8b,0xec,0x83,0xec,0x58][..]),
                    (STORM+0x27c50,&[0x55,0x8b,0xec,0x83,0xec,0x40][..]),
                    (STORM+0x34cd0,&[0x55,0x8b,0xec,0x83,0xec,0x6c][..]),
                    (STORM+0x34740,&[0x55,0x8b,0xec,0x33,0xc0][..]),
                    (0x4d1140,&[0x3b,0x05,0x6c,0x6b,0x59,0x00][..])] {
        if !readable(at,sig.len()) || slice::from_raw_parts(at as *const u8,sig.len())!=sig {return Err(format!("Hook incompatible en {at:08x}"))}
    }
    let grid=alloc(0x20000+grid_len,false)?+0x10000;
    // Build every code cave up front. The game remains suspended throughout installation.
    for p in &mut patches {
        if p.fix!=255 {p.bytes[p.fix..p.fix+4].copy_from_slice(&((grid+p.add) as u32).to_le_bytes());}
        if !p.cave.is_empty() {
            let cave=alloc(p.cave.len()+5,true)?;
            write(cave,&p.cave)?;
            write(cave+p.cave.len(),&jump(cave+p.cave.len(),p.va+p.expect.len()))?;
            p.bytes[..5].copy_from_slice(&jump(p.va,cave));
        }
    }
    WIDTH=w as usize; HEIGHT=h as usize; GRID=grid; GRID_LEN=grid_len;
    MENU_PIXELS=alloc(640*480,false)?;
    MENU_CURSOR=alloc(640*480,false)?;
    MENU_STAGE=alloc(WIDTH*HEIGHT,false)?;
    MENU_OUTPUT=alloc(WIDTH*HEIGHT,false)?;
    HUD_BACKGROUND=alloc(WIDTH*HEIGHT,false)?;
    HUD_SOURCE=alloc(WIDTH*HEIGHT,false)?;
    HUD_MASK=alloc(WIDTH*HEIGHT,false)?;
    HUD_ART=alloc(640*480,false)?;
    TITLE_ORIGINAL=alloc(640*480,false)?;
    TITLE_SCALED=alloc(WIDTH*HEIGHT,false)?;
    let (_,_,mw,mh)=menu_viewport(WIDTH,HEIGHT);
    MENU_XMAP=alloc(mw*4,false)?;MENU_YMAP=alloc(mh*4,false)?;
    for x in 0..mw {write32(MENU_XMAP+x*4,(x*640/mw) as u32);}
    for y in 0..mh {write32(MENU_YMAP+y*4,(y*480/mh) as u32);}
    for p in &patches {write(p.va,&p.bytes)?;}
    let movie_entry=eax_stub(play_movie as *const () as usize)?;
    MOVIE_PLAY=hook(0x4d4470,movie_entry,&[0x55,0x8b,0xec,0x51,0x56])?;
    let compose_entry=abi_stub(compose as *const () as usize,0,false)?;
    let copy_entry=abi_stub(copy as *const () as usize,5,true)?;
    let restore_entry=abi_stub(restore_under as *const () as usize,1,true)?;
    COMPOSE=hook(0x41e280,compose_entry,&[0x55,0x8b,0xec,0x83,0xec,0x14])?;
    COPY=hook(STORM+0x1a520,copy_entry,&[0x55,0x8b,0xec,0x8b,0x45,0x18])?;
    RESTORE=hook(0x41deb0,restore_entry,&[0x55,0x8b,0xec,0x83,0xec,0x0c])?;
    let window_entry=abi_stub(game_window as *const () as usize,4,true)?;
    WINDOW_PROC=hook(0x4d1d70,window_entry,&[0x55,0x8b,0xec,0x83,0xec,0x58])?;
    let background_entry=abi_stub(background_loaded as *const () as usize,8,true)?;
    BACKGROUND_LOAD=hook(STORM+0x27c50,background_entry,&[0x55,0x8b,0xec,0x83,0xec,0x40])?;
    let lock_entry=abi_stub(menu_lock as *const () as usize,5,true)?;
    let unlock_entry=abi_stub(menu_unlock as *const () as usize,4,true)?;
    STORM_LOCK=hook(STORM+0x34cd0,lock_entry,&[0x55,0x8b,0xec,0x83,0xec,0x6c])?;
    STORM_UNLOCK=hook(STORM+0x34740,unlock_entry,&[0x55,0x8b,0xec,0x33,0xc0])?;
    let dialog_entry=abi_stub(draw_dialogs as *const () as usize,2,true)?;
    DIALOG_DRAW=hook(0x41cb50,dialog_entry,&[0x55,0x8b,0xec,0x83,0xec,0x2c])?;
    let tooltip_entry=abi_stub(draw_tooltip as *const () as usize,2,true)?;
    TOOLTIP_DRAW=hook(0x4810f0,tooltip_entry,&[0x55,0x8b,0xec,0xa1,0x48,0x5c,0x65,0x00])?;
    let ui_stub=alloc(32,true)?;
    let mut code=vec![0x55,0x89,0xe5,0x83,0xe4,0xf0,0x83,0xec,0x08,0x50,0x51,0xb8];
    code.extend_from_slice(&(ui_hit as *const () as u32).to_le_bytes());
    code.extend_from_slice(&[0xff,0xd0,0x89,0xec,0x5d,0xc3]);
    write(ui_stub,&code)?;
    hook(0x4d1140,ui_stub,&[0x3b,0x05,0x6c,0x6b,0x59,0x00])?;
    INITIALIZED=true;
    log(&format!("Rust v13: resolución {w}x{h}; {} firmas comprobadas; cursor actualizado con visibilidad nativa y transición sin primarias vacías",patches.len()));
    Ok(())
}

#[no_mangle]
pub unsafe extern "system" fn InitializeResolution(config:*const u32)->u32 {
    if config.is_null() {return 0}
    match install(*config,*config.add(1)) {Ok(())=>1,Err(e)=>{log(&format!("ERROR: {e}"));0}}
}
#[no_mangle]
pub unsafe extern "system" fn DllMain(module:Handle,reason:u32,_reserved:Handle)->i32 {
    if reason==1 {DisableThreadLibraryCalls(module);} 1
}

#[no_mangle]
pub unsafe extern "system" fn ResolutionSelfTest(_p:Handle)->u32 {
    unsafe extern "C" fn movie_id(id:u32)->u32 {id*17+3}
    unsafe extern "C" fn replacement()->u32 {11}
    unsafe extern "system" fn sum5(a:u32,b:u32,c:u32,d:u32,e:u32)->u32 {a+2*b+3*c+4*d+5*e}
    unsafe extern "system" fn sum2(a:u32,b:u32)->u32 {3*a+7*b}
    unsafe extern "system" fn sum8(a:u32,b:u32,c:u32,d:u32,e:u32,f:u32,g:u32,h:u32)->u32 {a+2*b+3*c+4*d+5*e+6*f+7*g+8*h}
    unsafe extern "fastcall" fn sum_fast(a:u32,b:u32,c:u32,d:u32,e:u32)->u32 {a+2*b+3*c+4*d+5*e}
    unsafe extern "C" fn scratch() {std::arch::asm!("mov eax, 999",out("eax") _,out("ecx") _,out("edx") _);}
    let test=(||->Result<(),String>{
        let movie=eax_stub(movie_id as *const () as usize)?;
        for id in 0..27u32 {
            let result:u32;
            std::arch::asm!("call {target}",target=in(reg) movie,inlateout("eax") id=>result,clobber_abi("C"));
            if result!=id*17+3 {return Err("El puente de video perdii el identificador EAX".into())}
        }
        let p=alloc(32,true)?;
        write(p,&[0xb8,7,0,0,0,0x90,0xc3])?;
        let orig=hook(p,replacement as *const () as usize,&[0xb8,7,0,0,0,0x90])?;
        let a:unsafe extern "C" fn()->u32=mem::transmute(p);
        let b:unsafe extern "C" fn()->u32=mem::transmute(orig);
        if a()!=11 || b()!=7 {return Err("Trampolín incorrecto".into())}
        let wrapped=callback_stub(scratch as *const () as usize,orig)?;
        let c:unsafe extern "C" fn()->u32=mem::transmute(wrapped);
        if c()!=7 {return Err("El callback no preservó los registros".into())}
        let entry=abi_stub(sum5 as *const () as usize,5,true)?;
        let sum:unsafe extern "system" fn(u32,u32,u32,u32,u32)->u32=mem::transmute(entry);
        if sum(1,2,3,4,5)!=55 {return Err("El puente ABI no conservó los argumentos".into())}
        let entry=abi_stub(sum2 as *const () as usize,2,true)?;
        let draw:unsafe extern "system" fn(u32,u32)->u32=mem::transmute(entry);
        if draw(13,17)!=158 {return Err("El puente de composición de diálogos no conservó sus argumentos".into())}
        let entry=abi_stub(sum8 as *const () as usize,8,true)?;
        let bitmap:unsafe extern "system" fn(u32,u32,u32,u32,u32,u32,u32,u32)->u32=mem::transmute(entry);
        for _ in 0..128 {
            if bitmap(1,2,3,4,5,6,7,8)!=204 {return Err("La llamada de carga no conservó sus ocho parámetros".into())}
        }
        let entry=fastcall_stub(sum_fast as *const () as usize,3)?;
        let load:unsafe extern "fastcall" fn(u32,u32,u32,u32,u32)->u32=mem::transmute(entry);
        if load(1,2,3,4,5)!=55 {return Err("El puente de carga de la imagen no conservó los argumentos".into())}
        for &(w,h) in &PRESETS[1..] {
            parse(table(w,h).unwrap())?;
            WIDTH=w as usize;HEIGHT=h as usize;IN_GAME=true;
            let mid=0;
            if ui_hit(mid-1,h as i32-1)!=0 || ui_hit(mid,h as i32-1)!=1 ||
               ui_hit(mid+639,h as i32-1)!=1 || ui_hit(mid+640,h as i32-1)!=0 {
                return Err("Las esquinas inferiores no permiten interactuar con el mapa".into())
            }
        }
        WIDTH=640;HEIGHT=480;IN_GAME=false;
        Ok(())
    })();
    if test.is_ok() {1}else{0}
}
