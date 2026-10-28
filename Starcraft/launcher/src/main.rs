#![windows_subsystem="windows"]
#![allow(non_snake_case,static_mut_refs)]
#[path="../../common.rs"] mod win;
use win::*;
use std::{path::{Path,PathBuf},fs,ptr,mem};

const DONE:u32=0x8001;
const BG:u32=0x100b07;
static mut FONT:Handle=NULL;
static mut TITLE:Handle=NULL;
static mut BRUSH:Handle=NULL;
static mut BUSY:bool=false;
static mut SMALL:Handle=NULL;
static mut FIELD_BRUSH:Handle=NULL;

#[derive(Clone,Debug)]
struct Config {game:PathBuf,preset:usize,mode:usize,output:usize,filter:usize}
fn bundle()->PathBuf {std::env::current_exe().unwrap().parent().unwrap().to_owned()}
fn find_game()->PathBuf {
    let args:Vec<String>=std::env::args().collect();
    if let Some(i)=args.iter().position(|a|a=="--game") {
        if let Some(game)=args.get(i+1) {return PathBuf::from(game)}
    }
    let b=bundle();
    for d in b.ancestors() {if d.join("StarCraft.exe").is_file() {return d.to_owned()}}
    std::env::current_dir().unwrap_or(b)
}
fn load()->Config {
    let mut c=Config{game:find_game(),preset:2,mode:0,output:0,filter:0};
    if let Ok(s)=fs::read_to_string(bundle().join("launcher.ini")) {
        for l in s.lines() {if let Some((k,v))=l.split_once('=') {match k {
            "game"=>c.game=PathBuf::from(v),"preset"=>c.preset=v.parse::<usize>().unwrap_or(2).min(PRESETS.len()-1),
            "mode"=>c.mode=v.parse::<usize>().unwrap_or(0).min(1),"output"=>c.output=v.parse::<usize>().unwrap_or(0).min(3),
            "filter"=>c.filter=v.parse::<usize>().unwrap_or(0).min(3),_=>()
        }}}
    }
    // A package copied beside StarCraft.exe always belongs to that installation.
    // A stale saved path from another machine must not override its own folder.
    if bundle().join("StarCraft.exe").is_file() {c.game=bundle();}
    c
}
fn save(c:&Config)->Result<(),String> {
    fs::write(bundle().join("launcher.ini"),format!("game={}\npreset={}\nmode={}\noutput={}\nfilter={}\n",c.game.display(),c.preset,c.mode,c.output,c.filter)).map_err(|e|e.to_string())
}
fn log(s:&str) {use std::io::Write; if let Ok(mut f)=fs::OpenOptions::new().append(true).create(true).open(bundle().join("launcher.log")) {let _=writeln!(f,"{s}");}}
fn io(e:std::io::Error)->String {e.to_string()}
fn verify(game:&Path)->Result<Vec<u8>,String> {
    let exe=fs::read(game.join("StarCraft.exe")).map_err(|e|format!("No se puede leer StarCraft.exe: {e}"))?;
    if sha256(&exe)?!=GAME_HASH {return Err("Este launcher necesita el EXE original de StarCraft 1.16.1. El archivo elegido es otra versión o está modificado.".into())}
    let storm=fs::read(game.join("storm.dll")).map_err(io)?;
    if sha256(&storm)?!=STORM_HASH {return Err("Storm.dll no coincide con la versión verificada. Se ha cancelado el arranque.".into())}
    Ok(exe)
}
const STORM_HASH:&str="706ff2164ca472f27c44235ed55586644e5c86e68cd69b62d76f5a78778bff25";

fn display_filter(filter:usize)->Result<String,String> {
    match filter {
        0|1=>Ok(format!("renderer=direct3d9\nd3d9_filter={filter}\n")),
        2|3=>{
            let file=bundle().join("filters").join(if filter==2 {"crt-tv.glsl"}else{"pc-tubo.glsl"});
            if !file.is_file() {return Err(format!("Falta el filtro {}",file.display()))}
            Ok(format!("renderer=opengl\nshader={}\n",file.display()))
        },
        _=>Err("Filtro desconocido".into())
    }
}

// A durable journal makes a killed launcher recoverable on its next start.
struct Session {dir:PathBuf,game:PathBuf,active:bool}
impl Session {
    fn restore(dir:&Path,game:&Path)->Result<(),String> {
        if !dir.join("ready").exists() {return Ok(())}
        let names:&[&str]=if fs::read(dir.join("ready")).map_err(io)?==b"2" {
            &["ddraw.dll","ddraw.ini","version.dll","sc_resolution.dll","sc-hd-session.ini"]
        }else{&["ddraw.dll","ddraw.ini"]}; // Recover older launchers' journals too.
        for &n in names {
            let target=game.join(n);
            if dir.join(format!("{n}.present")).exists() {fs::copy(dir.join(n),target).map_err(io)?;}
            else if target.exists() {fs::remove_file(target).map_err(io)?;}
        }
        fs::remove_file(dir.join("ready")).map_err(io)?;
        Ok(())
    }
    fn recover()->Result<(),String> {
        let dir=bundle().join("session-backup");
        if !dir.join("ready").exists() {return Ok(())}
        let game=PathBuf::from(fs::read_to_string(dir.join("game.txt")).map_err(io)?);
        if let Ok(s)=fs::read_to_string(dir.join("pid.txt")) {if let Ok(pid)=s.trim().parse::<u32>() {unsafe {
            let p=OpenProcess(0x100000,0,pid);
            if !p.is_null() {let running=WaitForSingleObject(p,0)==258; CloseHandle(p); if running {return Err("Hay una partida iniciada por este launcher que sigue abierta. Ciérrala antes de recuperar la configuración.".into())}}
        }}}
        Self::restore(&dir,&game)
    }
    fn begin(game:&Path,c:&Config)->Result<Self,String> {
        Self::recover()?;
        let dir=bundle().join("session-backup"); fs::create_dir_all(&dir).map_err(io)?;
        for n in ["ddraw.dll","ddraw.ini","version.dll","sc_resolution.dll","sc-hd-session.ini"] {
            let marker=dir.join(format!("{n}.present")); if marker.exists() {fs::remove_file(&marker).map_err(io)?;}
            if game.join(n).exists() {fs::copy(game.join(n),dir.join(n)).map_err(io)?;fs::write(marker,b"1").map_err(io)?;}
        }
        if dir.join("pid.txt").exists() {fs::remove_file(dir.join("pid.txt")).map_err(io)?;}
        fs::write(dir.join("game.txt"),game.to_string_lossy().as_bytes()).map_err(io)?;
        fs::write(dir.join("ready"),b"2").map_err(io)?;
        let mut s=Self{dir,game:game.to_owned(),active:true};
        let result=(||{
            for name in ["ddraw.dll","version.dll","sc_resolution.dll"] {
            let source=bundle().join(name);let target=game.join(name);
            if fs::canonicalize(&source).map_err(io)?!=fs::canonicalize(&target).unwrap_or_else(|_|target.clone()) {
                fs::copy(source,target).map_err(io)?;
            }
            }
            let (w,h)=PRESETS[c.preset];
            fs::write(game.join("sc-hd-session.ini"),format!("[HD]\nwidth={w}\nheight={h}\n")).map_err(io)?;
            let (ow,oh)=match c.output {1=>(1920,1080),2=>(2560,1440),3=>(3840,2160),_=>(0,0)};
            let (w,h)=PRESETS[c.preset];
            let (ow,oh)=if c.mode==1 && c.output==0 {(w,h)}else{(ow,oh)};
            let effect=display_filter(c.filter)?;
            let ini=format!("; Generated for this session by the Rust launcher\n[ddraw]\nwidth={ow}\nheight={oh}\nwindowed=true\nfullscreen={}\nmaintas=true\nboxing=false\n{effect}lock_surfaces=true\nmaxfps=60\nvsync=false\nadjmouse=true\nsavesettings=0\nborder={}\nresizable=false\nnonexclusive=true\nsinglecpu=true\nmaxgameticks=0\nfixchilds=2\nkeytogglefullscreen=0x0D\n[StarCraft]\ngame_handles_close=true\n",if c.mode==0 {"true"}else{"false"},if c.mode==1 {"true"}else{"false"});
            fs::write(game.join("ddraw.ini"),ini).map_err(io)?;Ok::<(),String>(())
        })();
        if let Err(e)=result {s.finish()?;return Err(e)}
        Ok(s)
    }
    fn finish(&mut self)->Result<(),String> {if self.active {Self::restore(&self.dir,&self.game)?; self.active=false;}Ok(())}
}
impl Drop for Session {fn drop(&mut self) {if self.active {if let Err(e)=self.finish() {log(&format!("Restauración pendiente: {e}"));}}}}

struct GameProcess {info:ProcessInfo,finished:bool}
#[repr(C)]
struct ShellInfo {
    size:u32,mask:u32,window:Handle,verb:*const u16,file:*const u16,args:*const u16,
    directory:*const u16,show:i32,instance:Handle,idlist:Handle,class:*const u16,
    class_key:Handle,hotkey:u32,icon:Handle,process:Handle,
}
#[link(name="shell32")]
extern "system" {fn ShellExecuteExW(info:*mut ShellInfo)->i32;}
impl Drop for GameProcess {fn drop(&mut self) {unsafe {
    if !self.finished {TerminateProcess(self.info.process,1);WaitForSingleObject(self.info.process,5000);}
    if !self.info.thread.is_null() {CloseHandle(self.info.thread);} CloseHandle(self.info.process);
}}}
use std::os::windows::ffi::OsStrExt;
fn launch(c:Config,notify:Option<usize>)->Result<(),String> {
    let smoke=std::env::args().any(|a|a=="--smoke-test");
    unsafe {if !FindWindowW(wide("SWarClass").as_ptr(),ptr::null()).is_null() {return Err("Ya hay una ventana de StarCraft abierta. Ciérrala antes de iniciar otra partida.".into())}}
    verify(&c.game)?;
    for name in ["sc_resolution.dll","version.dll","ddraw.dll"] {
        if !bundle().join(name).exists() {return Err(format!("Falta {name} junto al launcher"))}
    }
    if !smoke {save(&c)?;}
    let (w,h)=PRESETS[c.preset];
    log(&format!("Iniciando {w}x{h}, modo={}, salida={}",c.mode,c.output));
    let mut session=Session::begin(&c.game,&c)?;
    unsafe {
        let mut si:StartupInfo=mem::zeroed(); si.cb=mem::size_of::<StartupInfo>() as u32;
        let mut pi:ProcessInfo=mem::zeroed();
        let app:Vec<u16>=c.game.join("StarCraft.exe").as_os_str().encode_wide().chain(Some(0)).collect();
        let cwd:Vec<u16>=c.game.as_os_str().encode_wide().chain(Some(0)).collect();
        if CreateProcessW(app.as_ptr(),ptr::null_mut(),NULL,NULL,0,0,NULL,cwd.as_ptr(),&mut si,&mut pi)==0 {
            let error=GetLastError();
            if error!=740 {return Err(format!("No se pudo abrir StarCraft ({error})"))}
            // Respect Windows' elevation/compatibility choice. The shell owns
            // any UAC prompt; do not alter compatibility or protection settings.
            let mut shell:ShellInfo=mem::zeroed();shell.size=mem::size_of::<ShellInfo>() as u32;
            shell.mask=0x40|0x100; shell.file=app.as_ptr();shell.directory=cwd.as_ptr();shell.show=1;
            if ShellExecuteExW(&mut shell)==0 || shell.process.is_null() {return Err(format!("Windows no pudo abrir el juego ({})",GetLastError()))}
            pi.process=shell.process;
            pi.pid=GetProcessId(shell.process);
        }
        let mut process=GameProcess{info:pi,finished:false};
        fs::write(session.dir.join("pid.txt"),process.info.pid.to_string()).map_err(io)?;
        if let Some(hwnd)=notify {PostMessageW(hwnd as Handle,0x8002,0,0);}
        if smoke {
            if WaitForSingleObject(process.info.process,12000)==258 {
                log("SMOKE: arranque estable durante 12 segundos; cerrando solo el proceso de prueba");
                TerminateProcess(process.info.process,0);WaitForSingleObject(process.info.process,5000);
            }else{return Err("La prueba terminó antes de los 12 segundos".into())}
        }else{WaitForSingleObject(process.info.process,u32::MAX);}
        let mut exit=0;GetExitCodeProcess(process.info.process,&mut exit);process.finished=true;
        session.finish()?;
        log(&format!("Juego cerrado; código {exit:#x}; archivos originales restaurados"));
        if exit!=0 {return Err(format!("StarCraft se cerró con el código {exit:#x}. La configuración original se ha restaurado. Consulta los registros para localizar el fallo."))}
    } Ok(())
}

fn pe_offset(exe:&[u8],va:usize)->Option<usize> {
    fn u16at(x:&[u8],i:usize)->usize {u16::from_le_bytes(x[i..i+2].try_into().unwrap()) as usize}
    fn u32at(x:&[u8],i:usize)->usize {u32::from_le_bytes(x[i..i+4].try_into().unwrap()) as usize}
    let pe=u32at(exe,0x3c);let count=u16at(exe,pe+6);let first=pe+24+u16at(exe,pe+20);let rva=va.checked_sub(0x400000)?;
    for i in 0..count {let p=first+i*40;let start=u32at(exe,p+12);let size=u32at(exe,p+16);if rva>=start && rva<start+size {return Some(u32at(exe,p+20)+rva-start)}} None
}
fn self_test()->Result<(),String> {
    let mut foreground=vec![0u8;640*480];let mut frame=vec![0u8;1920*1080];
    let background=vec![0u8;1920*1080];let cursor=[100i16,100,119,119];
    frame[105*1920+105]=31;
    copy_menu_foreground(&frame,1920,&mut foreground);
    if menu_content_visible(&background,1920,&foreground,Some(cursor)) {
        return Err("Una transición vacía solo con cursor sustituye el último fotograma".into())
    }
    frame[200*1920+300]=42;frame[105*1920+105]=0;
    copy_menu_foreground(&frame,1920,&mut foreground);
    if foreground[105*640+105]!=0 || foreground[200*640+300]!=42 ||
       !menu_content_visible(&background,1920,&foreground,None) {
        return Err("El cursor oculto deja restos o elimina el texto directo del briefing".into())
    }
    frame.fill(0);copy_menu_foreground(&frame,1920,&mut foreground);
    if menu_content_visible(&background,1920,&foreground,None) {
        return Err("Se publica una superficie vacía durante la carga".into())
    }
    log("TEST transición: cursor oculto renovado, briefing conservado y primarias vacías retenidas");
    let mut title=vec![91u8;1920*1080];let original=vec![17u8;640*480];
    let mut stage=vec![17u8;1920*1080];stage[400*1920+320]=0;stage[400*1920+321]=42;
    let now=std::time::Instant::now();
    for _ in 0..20 {
        if overlay_loading_caption(&mut title,1920,1080,&stage,&original)!=2 {return Err("El texto de carga incluye píxeles ajenos a su capa".into())}
    }
    if title[900*1920+960]!=0 || title[900*1920+963]!=42 || title[900*1920+959]!=91 {
        return Err("La imagen inicial o el texto de carga cambian de posición".into())
    }
    log(&format!("TEST carga: 20 composiciones incrementales en {} ms; fondo preparado una vez",now.elapsed().as_millis()));

    let (w,h)=(1280usize,720usize);
    let mut modal=vec![0u8;w*h];let mut mask=vec![0u8;w*h];let mut output=vec![77u8;w*h];
    for y in 260..460 {for x in 540..740 {modal[y*w+x]=5;mask[y*w+x]=1;}}
    modal[360*w+640]=0;
    scale_modal(&mut output,&modal,&mask,w,h,150,[540,260,739,459]);
    if output[360*w+640]!=0 || output[215*w+495]!=5 || output[200*w+480]!=77 ||
       modal_input(w,h,150,565,285)!=(590,310) {
        return Err("El menú ampliado no conserva su fondo, bordes o coordenadas de clic".into())
    }
    // A non-zero black palette entry must disappear at the background edge,
    // while a black detail enclosed inside a coloured icon remains intact.
    let mut palette=vec![255u8;1024];palette[7*4..7*4+3].fill(0);
    let mut resources=vec![7u8;49];
    for y in 2..5 {for x in 2..5 {resources[y*7+x]=5;}}
    resources[3*7+3]=7;
    let alpha=resource_alpha(&resources,&palette,7,7);
    if alpha[0]!=0 || alpha[1*7+3]!=0 || alpha[3*7+3]!=1 || alpha[2*7+2]!=1 {
        return Err("El fondo de recursos no se separa de los detalles oscuros del icono".into())
    }
    if sha256(b"abc")?!="ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad" {return Err("SHA256 incorrecto".into())}
    for filter in 0..4 {
        let effect=display_filter(filter)?;
        if filter<2 {
            if !effect.contains("renderer=direct3d9") {return Err("Presentador de píxel incorrecto".into())}
        }else if !effect.contains("renderer=opengl") || !effect.contains(if filter==2 {"crt-tv.glsl"}else{"pc-tubo.glsl"}) {
            return Err("El filtro no selecciona su archivo y presentador".into())
        }
    }
    let exe=verify(&find_game())?;
    for &(w,h) in &PRESETS[1..] {
        let data=table(w,h).unwrap();let (_,patches)=parse(data)?;
        for p in &patches {let off=pe_offset(&exe,p.va).ok_or("Dirección fuera del EXE")?;
            if exe.get(off..off+p.expect.len())!=Some(p.expect.as_slice()) {return Err(format!("Firma errónea en {:08x}",p.va))}
        }
        if parse(&data[..data.len()-1]).is_ok() {return Err("El parser aceptó una tabla truncada".into())}
        let (ml,mt,mw,mh)=menu_viewport(w as usize,h as usize);
        let (cx,cy)=menu_input(w as usize,h as usize,(ml+mw/2) as i32,(mt+mh/2) as i32);
        if (cx-320).abs()>1 || (cy-240).abs()>1 || mw>w as usize || mh>h as usize {return Err("Escala o clics de menú incorrectos".into())}
        let b=[180,32,444,319];let (px,py)=popup_offset(w as usize,h as usize,b);
        if (b[0] as i32+px as i32)*2+265>w as i32 || (b[1] as i32+py as i32)*2+288>h as i32 {return Err("Ventana de pausa fuera del centro".into())}
        let width=w as usize;let mid=0i16;
        if hud_offset(width,b"StatBtn",496,639)!=mid ||
           hud_offset(width,b"StatRes",220,639) as usize+639!=width-1 ||
           hud_offset(width,b"Minimap",0,137)!=mid ||
           hud_offset(width,b"StatData",138,407)!=mid ||
           hud_art_x(width,639+mid as i32)!=Some(639) ||
           hud_art_x(width,138+mid as i32)!=Some(138) ||
           hud_art_x(width,mid as i32)!=Some(0) ||
           hud_art_x(width,mid as i32-1).is_some() ||
           hud_art_x(width,mid as i32+640).is_some() {return Err("Las anclas de la interfaz no coinciden".into())}
        log(&format!("TEST {w}x{h}: {} firmas verificadas",patches.len()));
    }
    let qa=find_game().join("rust-launcher/qa");
    for &(w,h) in &PRESETS[1..] {
        let (w,h)=(w as usize,h as usize);let s=hud_scale(h);
        // Display-to-native coordinates must sample the same logical pixel as
        // the renderer, including fractional 125%/150% scales and the last row.
        for y in h-200*s/100..h {for x in 0..652*s/100 {
            let (mx,my)=hud_pointer(w,h,x as i32,y as i32);
            let expected=(x*652/(652*s/100),h-480+(y-(h-480*s/100))*480/(480*s/100));
            // 652*1.25 and 652*1.5 are integral; both axes align exactly.
            if (mx as usize,my as usize)!=expected {return Err("El ratón no coincide con la interfaz ampliada".into())}
        }}
        if hud_pointer(w,h,w as i32-1,0)!=(w as i32-1,0) ||
           hud_pointer(w,h,w as i32-1,h as i32-1)!=(w as i32-1,h as i32-1) {
            return Err("El escalado invade el terreno de la derecha".into())
        }
    }
    for (race,count) in [("t",86727),("z",89893),("p",89346)] {
        if let Ok(art)=fs::read(qa.join(format!("{race}console.raw"))) {
            if art.len()!=640*480 {return Err("Dimensiones de arte incorrectas".into())}
            let mut image=vec![77u8;1920*1080];
            if overlay_console(&mut image,1920,1080,&art,640,600)!=count {return Err("Silueta original incompleta".into())}
            for y in 0..480 {for x in 0..640 {
                let a=art[y*640+x];let value=image[(600+y)*1920+640+x];
                if value!=if a==0 {77}else{a} {return Err("El remate no conserva la transparencia original".into())}
            }}
            if image[..600*1920].iter().any(|&p|p!=77) || image[1079*1920+639]!=77 || image[1079*1920+1280]!=77 {return Err("La silueta invade el terreno exterior".into())}
            let style=console_style(count);
            let caps=finish_console_sides(&mut image,1920,1080,&art,640,600,style);
            if caps==0 {return Err("Los cierres laterales no se dibujan".into())}
            let mut preview=Vec::new();
            for y in 880..1080 {preview.extend_from_slice(&image[y*1920+624..y*1920+1296]);}
            fs::write(qa.join(format!("{race}console-finished.raw")),preview).map_err(io)?;
            log(&format!("TEST arte {race}: {count} píxeles originales + {caps} de cierre lateral"));
            let (w,h)=(1920usize,1080usize);
            let mut source=vec![77u8;w*h];let mut mask=vec![0u8;w*h];
            overlay_console(&mut source,w,h,&art,0,600);
            finish_console_sides(&mut source,w,h,&art,0,600,style);
            for y in 0..h {for x in 0..w {if source[y*w+x]!=77 {mask[y*w+x]=1;}}}
            // A synthetic black control must stay black; index zero is opaque
            // inside controls and transparent only outside the explicit mask.
            source[1079*w+30]=0;mask[1079*w+30]=1;
            let mut enlarged=vec![77u8;w*h];
            scale_ui_region(&mut enlarged,&source,&mask,w,h,0,600,652,480,0,360,978,720);
            for y in 0..h {for x in 0..w {
                let expected=if x<978 && y>=360 {
                    let from=(600+(y-360)*480/720)*w+x*652/978;
                    if mask[from]!=0 {source[from]}else{77}
                }else{77};
                if enlarged[y*w+x]!=expected {return Err("La interfaz ampliada pierde su máscara o sus píxeles negros".into())}
            }}
            if enlarged[1079*w+45]!=0 {return Err("El negro de los controles se hizo transparente".into())}
            fs::write(qa.join(format!("{race}console-left-150.raw")),&enlarged).map_err(io)?;
            log(&format!("TEST {race}: barra izquierda al 150%, máscara y fondo negro verificados"));
        }
    }
    unsafe {
        let dll=bundle().join("sc_resolution.dll"); let path:Vec<u16>=dll.as_os_str().encode_wide().chain(Some(0)).collect();
        let m=LoadLibraryExW(path.as_ptr(),NULL,0);
        if m.is_null() {return Err(format!("No se puede cargar la DLL de prueba: {}",GetLastError()))}
        let f=GetProcAddress(m,b"ResolutionSelfTest\0".as_ptr());
        if f.is_null() {FreeLibrary(m);return Err("Falta el autotest de la DLL".into())}
        let test:unsafe extern "system" fn(Handle)->u32=mem::transmute(f);let result=test(NULL); FreeLibrary(m);
        if result!=1 {return Err("Falló la ejecución de los trampolines x86".into())}
    }
    log("TEST OK: SHA256, tablas, parser, carga DLL y trampolines x86"); Ok(())
}

unsafe fn text(hwnd:Handle,id:i32)->String {
    let mut s=[0u16;2048];let n=GetWindowTextW(GetDlgItem(hwnd,id),s.as_mut_ptr(),s.len() as i32);
    String::from_utf16_lossy(&s[..n as usize])
}
unsafe fn selected(hwnd:Handle,id:i32)->usize {SendMessageW(GetDlgItem(hwnd,id),0x147,0,0).max(0) as usize}
unsafe fn config(hwnd:Handle)->Config {Config{game:PathBuf::from(text(hwnd,100)),preset:selected(hwnd,101),mode:selected(hwnd,102),output:selected(hwnd,103),filter:selected(hwnd,104)}}
unsafe fn label(hwnd:Handle,s:&str,x:i32,y:i32,w:i32,h:i32,title:bool)->Handle {
    let c=control(hwnd,"STATIC",s,0,x,y,w,h,0);SendMessageW(c,0x30,if title {TITLE}else{FONT} as usize,1);c
}
unsafe fn control(hwnd:Handle,class:&str,s:&str,style:u32,x:i32,y:i32,w:i32,h:i32,id:i32)->Handle {
    let c=CreateWindowExW(0,wide(class).as_ptr(),wide(s).as_ptr(),0x50000000|style,x,y,w,h,hwnd,id as Handle,GetModuleHandleW(ptr::null()),NULL);
    SendMessageW(c,0x30,FONT as usize,1);c
}
unsafe fn combo(hwnd:Handle,id:i32,x:i32,y:i32,w:i32,items:&[String],sel:usize) {
    let c=control(hwnd,"COMBOBOX","",0x00210213,x,y,w,240,id);
    for s in items {SendMessageW(c,0x143,0,wide(s).as_ptr() as isize);}
    SendMessageW(c,0x14e,sel,0);
}
unsafe fn paint_box(dc:Handle,r:Rect,color:u32) {
    let brush=CreateSolidBrush(color);FillRect(dc,&r,brush);DeleteObject(brush);
}
unsafe fn panel(dc:Handle,l:i32,t:i32,r:i32,b:i32) {
    paint_box(dc,Rect{left:l,top:t,right:r,bottom:b},0x725137);
    paint_box(dc,Rect{left:l+1,top:t+1,right:r-1,bottom:b-1},0x21160e);
    paint_box(dc,Rect{left:l+5,top:t+5,right:r-5,bottom:b-5},BG);
    for x in [l+7,r-10] {for y in [t+7,b-10] {
        paint_box(dc,Rect{left:x,top:y,right:x+3,bottom:y+3},0x8a7c65);
    }}
}
unsafe fn paint_launcher(hwnd:Handle) {
    let mut paint:PaintInfo=mem::zeroed();let dc=BeginPaint(hwnd,&mut paint);
    paint_box(dc,Rect{left:0,top:0,right:660,bottom:590},BG);
    for i in 0..72 {
        let x=(i*97+19)%638;let y=(i*43+11)%112;
        paint_box(dc,Rect{left:x,top:y,right:x+1,bottom:y+1},if i%5==0 {0x9c8770}else{0x40301e});
    }
    SetBkMode(dc,1);SelectObject(dc,TITLE);SetTextColor(dc,0xc3c5b4);
    let mut r=Rect{left:28,top:23,right:616,bottom:76};
    DrawTextW(dc,wide("STARCRAFT HD").as_ptr(),-1,&mut r,0x21);
    SelectObject(dc,SMALL);SetTextColor(dc,0x9e9870);
    r=Rect{left:28,top:80,right:616,bottom:104};
    DrawTextW(dc,wide("B R O O D   W A R").as_ptr(),-1,&mut r,0x21);
    panel(dc,20,124,624,365);panel(dc,20,378,624,429);
    paint_box(dc,Rect{left:28,top:116,right:616,bottom:118},0x8b6b3f);
    EndPaint(hwnd,&paint);
}
unsafe fn draw_button(item:&DrawItem) {
    let dc=item.dc;let r=item.rect;
    paint_box(dc,r,0x9d7950);
    paint_box(dc,Rect{left:r.left+2,top:r.top+2,right:r.right-2,bottom:r.bottom-2},if item.state&1!=0 {0x24170a}else{0x342616});
    paint_box(dc,Rect{left:r.left+6,top:r.top+6,right:r.right-6,bottom:r.bottom-6},0x140e07);
    SetBkMode(dc,1);SetTextColor(dc,if item.state&4!=0 {0x5c6554}else{0x48dbe5});
    SelectObject(dc,TITLE);let mut text=r;
    DrawTextW(dc,wide("JUGAR").as_ptr(),-1,&mut text,0x25);
}
unsafe fn draw_combo(item:&DrawItem) {
    paint_box(item.dc,item.rect,if item.state&1!=0 {0x392c1b}else{0x21160e});
    let mut text=[0u16;512];
    let index=if item.item==u32::MAX {SendMessageW(item.window,0x147,0,0)}else{item.item as isize};
    if index>=0 {SendMessageW(item.window,0x148,index as usize,text.as_mut_ptr() as isize);}
    SelectObject(item.dc,FONT);SetBkMode(item.dc,1);SetTextColor(item.dc,if item.state&4!=0 {0x687366}else{0xcddac2});
    let mut r=item.rect;r.left+=9;
    DrawTextW(item.dc,text.as_ptr(),-1,&mut r,0x24);
}
unsafe fn ui(hwnd:Handle) {
    let c=load();
    FONT=CreateFontW(-17,0,0,0,500,0,0,0,1,0,0,5,0,wide("Bahnschrift").as_ptr());
    SMALL=CreateFontW(-14,0,0,0,400,0,0,0,1,0,0,5,0,wide("Bahnschrift").as_ptr());
    TITLE=CreateFontW(-38,0,0,0,600,0,0,0,1,0,0,5,0,wide("Bahnschrift SemiCondensed").as_ptr());
    FIELD_BRUSH=CreateSolidBrush(0x21160e);
    label(hwnd,"RESOLUCIÓN",40,146,265,24,false);
    label(hwnd,"PANTALLA",332,146,265,24,false);
    combo(hwnd,101,40,177,264,&PRESETS.iter().map(|&(w,h)|format!("{w} × {h}")).collect::<Vec<_>>(),c.preset);
    combo(hwnd,102,332,177,264,&["Sin bordes".into(),"Ventana".into()],c.mode);
    label(hwnd,"SALIDA",40,248,265,24,false);
    label(hwnd,"IMAGEN",332,248,265,24,false);
    combo(hwnd,103,40,279,264,&["Escritorio".into(),"1920 × 1080".into(),"2560 × 1440".into(),"3840 × 2160".into()],c.output);
    combo(hwnd,104,332,279,264,&["Píxel nítido".into(),"Suavizado".into(),"CRT · Televisor".into(),"PC · Tubo".into()],c.filter);
    control(hwnd,"EDIT",&c.game.to_string_lossy(),0x00000080,36,393,568,24,100);
    control(hwnd,"BUTTON","JUGAR",0x1000b,28,448,588,52,200);
    let state=control(hwnd,"STATIC","Listo",1,28,516,588,24,203);
    SendMessageW(state,0x30,SMALL as usize,1);
}
unsafe fn status(hwnd:Handle,s:&str) {
    SetWindowTextW(hwnd,wide(&format!("StarCraft HD · {s}")).as_ptr());
    SetWindowTextW(GetDlgItem(hwnd,203),wide(s).as_ptr());
}
unsafe extern "system" fn window(hwnd:Handle,msg:u32,w:usize,l:isize)->isize {
    match msg {
        1=>{ui(hwnd);0}
        0x0f=>{paint_launcher(hwnd);0}
        0x2b=>{let item=&*(l as *const DrawItem);if item.kind==3 {draw_combo(item);}else{draw_button(item);}1}
        0x2c=>{(*(l as *mut MeasureItem)).height=28;1}
        0x138|0x133|0x134=>{let field=l as Handle==GetDlgItem(hwnd,100) || msg==0x134;
            SetTextColor(w as Handle,if field {0xc7c6b4}else{0xa8b69b});
            SetBkColor(w as Handle,if field {0x21160e}else{BG});SetBkMode(w as Handle,1);
            if field {FIELD_BRUSH as isize}else{BRUSH as isize}}
        0x111=>{
            match w&0xffff {
                200 if !BUSY=>{
                    let c=config(hwnd);let (rw,rh)=PRESETS[c.preset];
                    if rw as i32>GetSystemMetrics(0) || rh as i32>GetSystemMetrics(1) {
                        // Oversized render resolutions can still be downscaled by the presenter.
                        log("La resolución interna es mayor que el escritorio; se ajustará a la salida.");
                    }
                    BUSY=true;for id in [100,101,102,103,104,200,201] {EnableWindow(GetDlgItem(hwnd,id),0);}
                    status(hwnd,"Preparando el juego…");let handle=hwnd as usize;
                    std::thread::spawn(move||{
                        let result=launch(c,Some(handle));
                        let boxed=Box::new(result);
                        unsafe {PostMessageW(handle as Handle,DONE,0,Box::into_raw(boxed) as isize);}
                    });
                }
                201=>{match save(&config(hwnd)) {Ok(())=>status(hwnd,"Configuración guardada"),Err(e)=>{MessageBoxW(hwnd,wide(&e).as_ptr(),wide("No se pudo guardar").as_ptr(),0x10);}}}
                202=>{ShellExecuteW(hwnd,wide("open").as_ptr(),wide(&bundle().to_string_lossy()).as_ptr(),ptr::null(),ptr::null(),1);}
                _=>()
            }0
        }
        0x8002=>{status(hwnd,"En partida");0}
        DONE=>{
            let result=*Box::from_raw(l as *mut Result<(),String>);BUSY=false;
            for id in [100,101,102,103,104,200,201] {EnableWindow(GetDlgItem(hwnd,id),1);}
            match result {Ok(())=>status(hwnd,"Listo"),Err(e)=>{log(&format!("ERROR: {e}"));status(hwnd,"No se pudo completar el arranque");MessageBoxW(hwnd,wide(&e).as_ptr(),wide("StarCraft HD").as_ptr(),0x10);}}0
        }
        0x10=>{if BUSY {MessageBoxW(hwnd,wide("Cierra StarCraft antes de cerrar el launcher para que pueda restaurar la configuración.").as_ptr(),wide("Partida en curso").as_ptr(),0x40);}else{DestroyWindow(hwnd);}0}
        2=>{PostQuitMessage(0);0}
        _=>DefWindowProcW(hwnd,msg,w,l)
    }
}
fn main() {
    let args:Vec<String>=std::env::args().collect();
    if args.iter().any(|a|a=="--self-test") {
        let r=self_test();if let Err(e)=&r {log(&format!("TEST ERROR: {e}"));}std::process::exit(if r.is_ok(){0}else{1});
    }
    let mut cli=None;
    if let Some(i)=args.iter().position(|a|a=="--launch"||a=="--smoke-test") {
        if let Some(s)=args.get(i+1) {if let Some(n)=PRESETS.iter().position(|&(w,h)|s==&format!("{w}x{h}")) {let mut c=load();c.preset=n;c.mode=usize::from(args.iter().any(|a|a=="--windowed"));if let Some(j)=args.iter().position(|a|a=="--filter") {
            c.filter=args.get(j+1).and_then(|v|v.parse::<usize>().ok()).filter(|&v|v<=3).unwrap_or(0);
        } cli=Some(c);}}
        if cli.is_none() {log("ERROR: --launch necesita una resolución admitida");std::process::exit(2)}
    }
    unsafe {
        SetProcessDPIAware();
        let mutex=CreateMutexW(NULL,0,wide("Local\\StarCraftRustResolutionLauncher").as_ptr());
        if mutex.is_null() || GetLastError()==183 {MessageBoxW(NULL,wide("El launcher ya está abierto.").as_ptr(),wide("StarCraft").as_ptr(),0x40);return}
        if let Err(e)=Session::recover() {MessageBoxW(NULL,wide(&e).as_ptr(),wide("Recuperación pendiente").as_ptr(),0x10);CloseHandle(mutex);return}
        if let Some(c)=cli {let r=launch(c,None);if let Err(e)=&r {log(e);}CloseHandle(mutex);std::process::exit(if r.is_ok(){0}else{1});}
        let instance=GetModuleHandleW(ptr::null());BRUSH=CreateSolidBrush(BG);
        let class=wide("StarCraftRustLauncher");
        let mut icon=LoadIconW(instance,1usize as *const u16);
        if icon.is_null() {icon=LoadIconW(NULL,32512usize as *const u16);}
        let wc=WndClass{style:3,proc:Some(window),cls_extra:0,wnd_extra:0,instance,icon,cursor:LoadCursorW(NULL,32512usize as *const u16),background:BRUSH,menu:ptr::null(),class:class.as_ptr()};
        RegisterClassW(&wc);
        let hwnd=CreateWindowExW(0,class.as_ptr(),wide("StarCraft HD").as_ptr(),0x00ca0000,(GetSystemMetrics(0)-660)/2,(GetSystemMetrics(1)-590)/2,660,590,NULL,NULL,instance,NULL);
        if hwnd.is_null() {CloseHandle(mutex);return}
        ShowWindow(hwnd,1);UpdateWindow(hwnd);let mut m:Msg=mem::zeroed();
        while GetMessageW(&mut m,NULL,0,0)>0 {TranslateMessage(&m);DispatchMessageW(&m);}
        CloseHandle(mutex);
    }
}
