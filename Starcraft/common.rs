#![allow(dead_code, non_snake_case)]
use std::ffi::c_void;
pub type Handle = *mut c_void;
pub const NULL: Handle = std::ptr::null_mut();
pub fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(Some(0)).collect() }

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Rect { pub left: i32, pub top: i32, pub right: i32, pub bottom: i32 }
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Point { pub x: i32, pub y: i32 }
#[repr(C)]
pub struct Msg { pub hwnd: Handle, pub message: u32, pub wparam: usize, pub lparam: isize, pub time: u32, pub pt: Point, pub private: u32 }
#[repr(C)]
pub struct WndClass { pub style: u32, pub proc: Option<unsafe extern "system" fn(Handle,u32,usize,isize)->isize>, pub cls_extra:i32, pub wnd_extra:i32, pub instance:Handle, pub icon:Handle, pub cursor:Handle, pub background:Handle, pub menu:*const u16, pub class:*const u16 }
#[repr(C)]
pub struct StartupInfo { pub cb:u32, pub reserved:*mut u16, pub desktop:*mut u16, pub title:*mut u16, pub x:u32, pub y:u32, pub xsize:u32, pub ysize:u32, pub xcount:u32, pub ycount:u32, pub fill:u32, pub flags:u32, pub show:u16, pub reserved2:u16, pub reserved_ptr:*mut u8, pub stdin:Handle, pub stdout:Handle, pub stderr:Handle }
#[repr(C)]
pub struct ProcessInfo { pub process:Handle, pub thread:Handle, pub pid:u32, pub tid:u32 }
#[repr(C)]
pub struct MemoryInfo { pub base:Handle, pub allocation:Handle, pub protection:u32, pub size:usize, pub state:u32, pub protect:u32, pub kind:u32 }
#[repr(C)]
pub struct PaintInfo {pub dc:Handle,pub erase:i32,pub rect:Rect,pub restore:i32,pub update:i32,pub reserved:[u8;32]}
#[repr(C)]
pub struct DrawItem {pub kind:u32,pub id:u32,pub item:u32,pub action:u32,pub state:u32,pub window:Handle,pub dc:Handle,pub rect:Rect,pub data:usize}
#[repr(C)]
pub struct MeasureItem {pub kind:u32,pub id:u32,pub item:u32,pub width:u32,pub height:u32,pub data:usize}

#[link(name="kernel32")]
extern "system" {
    pub fn GetModuleHandleW(name:*const u16)->Handle;
    pub fn LoadLibraryExW(name:*const u16,file:Handle,flags:u32)->Handle;
    pub fn FreeLibrary(module:Handle)->i32;
    pub fn GetProcAddress(module:Handle,name:*const u8)->Handle;
    pub fn GetCurrentProcess()->Handle;
    pub fn OpenProcess(access:u32,inherit:i32,pid:u32)->Handle;
    pub fn GetLastError()->u32;
    pub fn GetTickCount()->u32;
    pub fn DisableThreadLibraryCalls(module:Handle)->i32;
    pub fn VirtualAlloc(address:Handle,size:usize,kind:u32,protect:u32)->Handle;
    pub fn VirtualProtect(address:Handle,size:usize,protect:u32,old:*mut u32)->i32;
    pub fn VirtualQuery(address:Handle,info:*mut MemoryInfo,len:usize)->usize;
    pub fn FlushInstructionCache(process:Handle,address:Handle,size:usize)->i32;
    pub fn CreateProcessW(app:*const u16,command:*mut u16,pa:Handle,ta:Handle,inherit:i32,flags:u32,environment:Handle,cwd:*const u16,start:*mut StartupInfo,info:*mut ProcessInfo)->i32;
    pub fn VirtualAllocEx(process:Handle,address:Handle,size:usize,kind:u32,protect:u32)->Handle;
    pub fn VirtualFreeEx(process:Handle,address:Handle,size:usize,kind:u32)->i32;
    pub fn WriteProcessMemory(process:Handle,address:Handle,data:*const u8,size:usize,written:*mut usize)->i32;
    pub fn ReadProcessMemory(process:Handle,address:Handle,data:*mut u8,size:usize,read:*mut usize)->i32;
    pub fn CreateRemoteThread(process:Handle,attr:Handle,stack:usize,start:Handle,param:Handle,flags:u32,id:*mut u32)->Handle;
    pub fn WaitForSingleObject(handle:Handle,ms:u32)->u32;
    pub fn GetExitCodeThread(thread:Handle,code:*mut u32)->i32;
    pub fn GetExitCodeProcess(process:Handle,code:*mut u32)->i32;
    pub fn CloseHandle(handle:Handle)->i32;
    pub fn ResumeThread(thread:Handle)->u32;
    pub fn TerminateProcess(process:Handle,code:u32)->i32;
    pub fn CreateMutexW(attr:Handle,initial:i32,name:*const u16)->Handle;
}
#[link(name="user32")]
extern "system" {
    pub fn SetProcessDPIAware()->i32;
    pub fn RegisterClassW(class:*const WndClass)->u16;
    pub fn CreateWindowExW(ex:u32,class:*const u16,text:*const u16,style:u32,x:i32,y:i32,w:i32,h:i32,parent:Handle,menu:Handle,instance:Handle,param:Handle)->Handle;
    pub fn DefWindowProcW(hwnd:Handle,msg:u32,w:usize,l:isize)->isize;
    pub fn ShowWindow(hwnd:Handle,cmd:i32)->i32;
    pub fn UpdateWindow(hwnd:Handle)->i32;
    pub fn GetMessageW(msg:*mut Msg,hwnd:Handle,min:u32,max:u32)->i32;
    pub fn TranslateMessage(msg:*const Msg)->i32;
    pub fn DispatchMessageW(msg:*const Msg)->isize;
    pub fn PostQuitMessage(code:i32);
    pub fn PostMessageW(hwnd:Handle,msg:u32,w:usize,l:isize)->i32;
    pub fn SendMessageW(hwnd:Handle,msg:u32,w:usize,l:isize)->isize;
    pub fn SetWindowTextW(hwnd:Handle,text:*const u16)->i32;
    pub fn GetWindowTextW(hwnd:Handle,text:*mut u16,max:i32)->i32;
    pub fn GetDlgItem(hwnd:Handle,id:i32)->Handle;
    pub fn EnableWindow(hwnd:Handle,enable:i32)->i32;
    pub fn DestroyWindow(hwnd:Handle)->i32;
    pub fn MessageBoxW(hwnd:Handle,text:*const u16,title:*const u16,kind:u32)->i32;
    pub fn LoadCursorW(instance:Handle,name:*const u16)->Handle;
    pub fn LoadIconW(instance:Handle,name:*const u16)->Handle;
    pub fn GetSysColorBrush(index:i32)->Handle;
    pub fn GetSystemMetrics(index:i32)->i32;
    pub fn FindWindowW(class:*const u16,title:*const u16)->Handle;
    pub fn FillRect(dc:Handle,rect:*const Rect,brush:Handle)->i32;
    pub fn DrawTextW(dc:Handle,text:*const u16,len:i32,rect:*mut Rect,format:u32)->i32;
    pub fn InvalidateRect(hwnd:Handle,rect:*const Rect,erase:i32)->i32;
    pub fn BeginPaint(hwnd:Handle,paint:*mut PaintInfo)->Handle;
    pub fn EndPaint(hwnd:Handle,paint:*const PaintInfo)->i32;
}
#[link(name="gdi32")]
extern "system" {
    pub fn CreateSolidBrush(color:u32)->Handle;
    pub fn CreateFontW(h:i32,w:i32,esc:i32,orient:i32,weight:i32,italic:u32,under:u32,strike:u32,charset:u32,out:u32,clip:u32,quality:u32,pitch:u32,face:*const u16)->Handle;
    pub fn SetTextColor(dc:Handle,color:u32)->u32;
    pub fn SetBkColor(dc:Handle,color:u32)->u32;
    pub fn SetBkMode(dc:Handle,mode:i32)->i32;
    pub fn SelectObject(dc:Handle,obj:Handle)->Handle;
    pub fn DeleteObject(obj:Handle)->i32;
}
#[link(name="shell32")]
extern "system" { pub fn ShellExecuteW(hwnd:Handle,op:*const u16,file:*const u16,args:*const u16,dir:*const u16,show:i32)->Handle; }
#[link(name="bcrypt")]
extern "system" {
    pub fn BCryptOpenAlgorithmProvider(alg:*mut Handle,name:*const u16,implementation:*const u16,flags:u32)->i32;
    pub fn BCryptHash(alg:Handle,key:*const u8,key_len:u32,data:*const u8,len:u32,out:*mut u8,out_len:u32)->i32;
    pub fn BCryptCloseAlgorithmProvider(alg:Handle,flags:u32)->i32;
}

pub const PRESETS: &[(u32,u32)] = &[(640,480),(960,540),(1280,720),(1280,800),(1280,880),(1536,864),(1600,900),(1920,1080)];
pub fn table(w:u32,h:u32)->Option<&'static [u8]> {
    match (w,h) {
        (960,540)=>Some(include_bytes!("resolution/data/960x540.bin")),
        (1280,720)=>Some(include_bytes!("resolution/data/1280x720.bin")),
        (1280,800)=>Some(include_bytes!("resolution/data/1280x800.bin")),
        (1280,880)=>Some(include_bytes!("resolution/data/1280x880.bin")),
        (1536,864)=>Some(include_bytes!("resolution/data/1536x864.bin")),
        (1600,900)=>Some(include_bytes!("resolution/data/1600x900.bin")),
        (1920,1080)=>Some(include_bytes!("resolution/data/1920x1080.bin")),
        _=>None
    }
}
#[derive(Clone)]
pub struct Patch { pub va:usize, pub fix:usize, pub add:usize, pub expect:Vec<u8>, pub bytes:Vec<u8>, pub cave:Vec<u8> }
pub fn parse(data:&[u8])->Result<(usize,Vec<Patch>),String> {
    fn take<'a>(data:&mut &'a [u8],n:usize)->Result<&'a [u8],String> { if data.len()<n { return Err("Tabla truncada".into()) } let (a,b)=data.split_at(n); *data=b; Ok(a) }
    fn u32le(data:&[u8])->usize { u32::from_le_bytes(data.try_into().unwrap()) as usize }
    if data.len()<16 || &data[..4]!=b"SCR1" { return Err("Tabla no válida".into()) }
    let grid=u32le(&data[8..12]); let count=u32le(&data[12..16]);
    if count>1000 || grid>1_000_000 {return Err("Tabla fuera de límites".into())}
    let mut rest=&data[16..]; let mut out=Vec::new();
    for _ in 0..count {
        let h=take(&mut rest,11)?; let n=h[4] as usize; let c=h[6] as usize;
        if n==0 || n>32 || c>32 || (h[5]!=255 && h[5] as usize+4>n) || (c>0 && n<5) {return Err("Registro no válido".into())}
        out.push(Patch {va:u32le(&h[..4]),fix:h[5] as usize,add:u32le(&h[7..11]),expect:take(&mut rest,n)?.to_vec(),bytes:take(&mut rest,n)?.to_vec(),cave:take(&mut rest,c)?.to_vec()});
    }
    if !rest.is_empty() { return Err("Datos extra en la tabla".into()) }
    Ok((grid,out))
}
pub fn sha256(data:&[u8])->Result<String,String> { unsafe {
    let mut alg=NULL; let mut digest=[0u8;32];
    if BCryptOpenAlgorithmProvider(&mut alg,wide("SHA256").as_ptr(),std::ptr::null(),0)!=0 {return Err("No se pudo iniciar SHA256".into())}
    let result=BCryptHash(alg,std::ptr::null(),0,data.as_ptr(),data.len() as u32,digest.as_mut_ptr(),32);
    BCryptCloseAlgorithmProvider(alg,0);
    if result!=0 {return Err("No se pudo calcular SHA256".into())}
    Ok(digest.iter().map(|b|format!("{b:02x}")).collect())
}}
pub const GAME_HASH:&str="ad6b58b27b8948845ccfa69bcfcc1b10d6aa7a27a371ee3e61453925288c6a46";

// Native controls stay in a logical canvas; the dialog layer scales their
// completed pixels, and mouse input uses the inverse of that same transform.
pub fn hud_offset(width:usize,name:&[u8],_left:i16,_right:i16)->i16 {
    if name==b"StatRes" {(width-640) as i16}else{0}
}
pub fn hud_art_x(_width:usize,x:i32)->Option<i32> {
    let local=x;
    (0..640).contains(&local).then_some(local)
}
pub fn hud_scale(height:usize)->usize {if height>=864 {150}else if height>=720 {125}else{100}}
pub fn hud_pointer(width:usize,height:usize,x:i32,y:i32)->(i32,i32) {
    let s=hud_scale(height) as i32;
    // The bottom canvas includes the race-specific right-hand closure.
    if x>=0 && x<652*s/100 && y>=height as i32-200*s/100 && y<height as i32 {
        return (x*100/s,height as i32-((height as i32-y)*100+s-1)/s)
    }
    if x>=width as i32-420*s/100 && x<width as i32 && y>=0 && y<32*s/100 {
        return (width as i32-((width as i32-x)*100+s-1)/s,y*100/s)
    }
    (x,y)
}

// All reads use an immutable snapshot: the enlarged destination overlaps the
// native source. Masked pixels preserve the terrain, including black HUD pixels.
pub fn scale_ui_region(dst:&mut [u8],src:&[u8],mask:&[u8],w:usize,h:usize,
                       sx:usize,sy:usize,sw:usize,sh:usize,dx:usize,dy:usize,dw:usize,dh:usize) {
    assert!(dst.len()>=w*h && src.len()>=w*h && mask.len()>=w*h);
    assert!(sx+sw<=w && sy+sh<=h && dx+dw<=w && dy+dh<=h && dw>0 && dh>0);
    for y in 0..dh {for x in 0..dw {
        let at=(sy+y*sh/dh)*w+sx+x*sw/dw;
        if mask[at]!=0 {dst[(dy+y)*w+dx+x]=src[at];}
    }}
}

// Native resource surfaces can use several palette entries for black. Remove
// only the dark background connected to the surface edge; enclosed icon details
// and dark interiors stay opaque. Palette index alone cannot identify the key.
pub fn resource_alpha(pixels:&[u8],palette:&[u8],w:usize,h:usize)->Vec<u8> {
    assert!(w>0 && h>0 && pixels.len()==w*h && palette.len()>=1024);
    let mut alpha=vec![1u8;w*h];let mut queue=Vec::with_capacity(w*h);
    let dark=|at:usize| {
        let color=pixels[at] as usize;
        color==0 || palette[color*4..color*4+3].iter().all(|&channel|channel<=24)
    };
    for y in 0..h {for x in 0..w {
        let at=y*w+x;
        if (x==0 || y==0 || x+1==w || y+1==h) && dark(at) {alpha[at]=0;queue.push(at);}
    }}
    let mut head=0;
    while head<queue.len() {
        let at=queue[head];head+=1;let (x,y)=(at%w,at/w);
        for next in [if x>0 {Some(at-1)}else{None},if x+1<w {Some(at+1)}else{None},
                     if y>0 {Some(at-w)}else{None},if y+1<h {Some(at+w)}else{None}].into_iter().flatten() {
            if alpha[next]!=0 && dark(next) {alpha[next]=0;queue.push(next);}
        }
    }
    alpha
}
pub fn overlay_loading_caption(dst:&mut [u8],width:usize,height:usize,
                               stage:&[u8],original:&[u8])->usize {
    assert!(dst.len()>=width*height && stage.len()>=width*480 && original.len()==640*480);
    let (left,top,w,h)=menu_viewport(width,height);
    let mut changes=Vec::with_capacity(4096);
    for y in 0..480 {for x in 0..640 {
        let pixel=stage[y*width+x];
        if pixel!=original[y*640+x] {
            changes.push((x,y,pixel));
            // An uninitialised native surface is not a loading caption.
            if changes.len()>=640*480/5 {return 0}
        }
    }}
    let count=changes.len();
    for (x,y,pixel) in changes {
        let x1=(x*w+639)/640;let x2=((x+1)*w+639)/640;
        let y1=(y*h+479)/480;let y2=((y+1)*h+479)/480;
        for row in y1..y2 {dst[(top+row)*width+left+x1..(top+row)*width+left+x2].fill(pixel);}
    }
    count
}

pub fn menu_viewport(width:usize,height:usize)->(usize,usize,usize,usize) {
    let (w,h)=if width*3>=height*4 {(height*4/3,height)}else{(width,width*3/4)};
    ((width-w)/2,(height-h)/2,w,h)
}
pub fn menu_input(width:usize,height:usize,x:i32,y:i32)->(i32,i32) {
    let (left,top,w,h)=menu_viewport(width,height);
    (((x-left as i32)*640).div_euclid(w as i32),
     ((y-top as i32)*480).div_euclid(h as i32))
}
pub fn popup_offset(width:usize,height:usize,b:[i16;4])->(i16,i16) {
    let w=b[2] as i32-b[0] as i32+1;
    let h=b[3] as i32-b[1] as i32+1;
    (((width as i32-w)/2-b[0] as i32) as i16,((height as i32-h)/2-b[1] as i32) as i16)
}
pub fn modal_input(w:usize,h:usize,scale:usize,x:i32,y:i32)->(i32,i32) {
    ((x-w as i32/2)*100/(scale as i32)+w as i32/2,
     (y-h as i32/2)*100/(scale as i32)+h as i32/2)
}
pub fn scale_modal(dst:&mut [u8],src:&[u8],mask:&[u8],w:usize,h:usize,scale:usize,b:[usize;4]) {
    let p=scale as i32;let (cx,cy)=(w as i32/2,h as i32/2);
    let l=(cx+(b[0] as i32-cx)*p/100-2).clamp(0,w as i32) as usize;
    let t=(cy+(b[1] as i32-cy)*p/100-2).clamp(0,h as i32) as usize;
    let r=(cx+(b[2] as i32-cx)*p/100+3).clamp(0,w as i32) as usize;
    let bottom=(cy+(b[3] as i32-cy)*p/100+3).clamp(0,h as i32) as usize;
    for y in t..bottom {for x in l..r {
        let (sx,sy)=modal_input(w,h,scale,x as i32,y as i32);
        if sx>=0 && sy>=0 && sx<w as i32 && sy<h as i32 {
            let at=sy as usize*w+sx as usize;
            if mask[at]!=0 {dst[y*w+x]=src[at];}
        }
    }}
}

// Reuse the loaded race's indexed art. Index 0 is the console's transparent
// key, so the terrain remains visible through its silhouette and upper gaps.
pub fn overlay_console(dst:&mut [u8],dw:usize,dh:usize,art:&[u8],left:usize,top:usize)->usize {
    assert!(dst.len()>=dw*dh && art.len()>=640*480 && left+640<=dw && top+480<=dh);
    let mut painted=0;
    for y in 0..480 {
        for x in 0..640 {
            let pixel=art[y*640+x];
            if pixel!=0 {dst[(top+y)*dw+left+x]=pixel;painted+=1;}
        }
    }
    painted
}

pub fn console_style(opaque:usize)->u8 {match opaque {89893=>1,89346=>2,_=>0}}
pub fn console_cap_width(art:&[u8],right:bool,row:usize,style:u8)->usize {
    let edge=if right {639}else{0};
    if row>=480 || art[row*640+edge]==0 {return 0}
    let wave=row%14;
    let base=match style {1=>8+wave.min(14-wave)/3,2=>8,_=>6};
    let mut run=1;
    while run<base && row>=run && art[(row-run)*640+edge]!=0 {run+=1;}
    base.min(run)
}
pub fn finish_console_sides(dst:&mut [u8],dw:usize,dh:usize,art:&[u8],left:usize,top:usize,style:u8)->usize {
    assert!(dst.len()>=dw*dh && art.len()>=640*480 && left+652<=dw && top+480<=dh);
    let mut painted=0;
    for row in 280..480 {
        for right in [false,true] {
            let width=console_cap_width(art,right,row,style);
            let edge=if right {639}else{0};
            for distance in 1..=width {
                if !right && distance>left {continue}
                let sample=if right {640-distance}else{distance-1};
                let ink=art[row*640+sample];
                let pixel=if ink==0 {art[row*640+edge]}else{ink};
                let x=if right {left+639+distance}else{left-distance};
                dst[(top+row)*dw+x]=pixel;painted+=1;
            }
        }
    }
    painted
}

// Opaque modal surfaces cover the text underneath them. Clip source and target
// together so a popup near an edge cannot spill outside the framebuffer.
pub fn overlay_dialog(dst:&mut [u8],dw:usize,dh:usize,src:&[u8],sw:usize,sh:usize,x:i32,y:i32) {
    assert!(dst.len()>=dw*dh && src.len()>=sw*sh);
    let sx=(-x).max(0) as usize;let sy=(-y).max(0) as usize;
    let dx=x.max(0) as usize;let dy=y.max(0) as usize;
    if sx>=sw || sy>=sh || dx>=dw || dy>=dh {return}
    let width=(sw-sx).min(dw-dx);let height=(sh-sy).min(dh-dy);
    for row in 0..height {
        let s=(sy+row)*sw+sx;let d=(dy+row)*dw+dx;
        dst[d..d+width].copy_from_slice(&src[s..s+width]);
    }
}
