# StarCraft HD: guía para agentes

## Alcance y estructura

Este directorio es un workspace Rust de Windows x86. `launcher/src/main.rs`
contiene el launcher Win32, configuración, restauración y arranque normal;
`resolution/src/lib.rs`, el componente de resolución; `common.rs`, las llamadas
Win32 y operaciones de composición compartidas. `resolution/data/*.bin` son las
siete tablas verificadas. No modifiques lógica de juego, temporizadores ni red.

El destino es `i686-pc-windows-gnu`: las convenciones stdcall/fastcall y los
registros EAX/ECX/EDX son específicos de x86. Las firmas de cada instrucción
se comprueban antes de escribir en memoria. Nunca omitas estas comprobaciones.

## Compilar

- Instala Rust con el toolchain `stable-i686-pc-windows-gnu` y Python 3.
- Dependencias de desarrollo: `python -m pip install -r requirements-dev.txt`.
- Ejecuta `./build.ps1 -GameExe 'C:/ruta/StarCraft.exe'` con una copia propia
  de StarCraft 1.16.1. El script también admite `STARCRAFT_EXE`.
- Si existe `toolchain/` local se usa el entorno portátil; no se distribuye.
- Sin EXE se compilan las tablas incluidas. Con EXE se regeneran usando el mapa
  MIT de `tools/vendor/renderer_patch_sites.py` y se comprueban sus firmas.
- `tools/package.ps1` prepara el paquete. El artefacto es `StarCraft-HD.exe`.
  No recuperes el nombre anterior ni dependas de una ruta absoluta del autor.

Hashes admitidos: StarCraft.exe
`ad6b58b27b8948845ccfa69bcfcc1b10d6aa7a27a371ee3e61453925288c6a46`;
storm.dll `706ff2164ca472f27c44235ed55586644e5c86e68cd69b62d76f5a78778bff25`.

## Contratos que hay que conservar

- El launcher no debe inyectar código en otros procesos: no usar WriteProcessMemory,
  VirtualAllocEx ni CreateRemoteThread. Abre el juego normalmente; si Windows exige
  elevación, delega en ShellExecuteEx y respeta el aviso del sistema.
- `bootstrap/` genera version.dll, que reenvía la ABI al Version del directorio del
  sistema. Solo prepara un salto en la entrada del EXE de StarCraft. La carga de
  sc_resolution.dll ocurre en esa entrada, fuera de DllMain y del bloqueo del loader.
  Restaurar los siete bytes originales antes de inicializar. Alinear pila y conservar
  todos los registros/banderas. La base GNU predeterminada 0x10000000 colisiona con
  Smackw32.dll, que no admite reubicación; usar 0x61000000 para este componente.
- Conservar EXE en disco. Comprobar las firmas antes de escribir en memoria propia.
- En vídeos, el dispatcher 0x4d4470 recibe el ID en EAX. No truncarlo con una firma
  stdcall. Mientras reproduce, conservar la presentación nativa de Smacker, sin
  redirigir sus locks ni componer los menús sobre ella. No borrar el framebuffer de briefing cada frame;
  solo limpiar al cargar un nuevo backgnd.pcx/title.pcx.
- Defender detectó el launcher antiguo como Behavior:Win32/DefenseEvasion.A!ml.
  No restaurar cuarentena, añadir exclusiones, desactivar protección ni ocultar
  código para evitar la detección. Verificar el nuevo paquete y no garantizar que
  nunca será detectado. Mantener la implementación auditable y sin empaquetadores.
- Alinear la pila a 16 bytes antes de callbacks Rust. Storm ordinal 325 recibe
  OCHO parámetros y termina con `ret 0x20`; truncar esa firma causó 0xc0000005.
- Los diálogos trabajan en coordenadas lógicas; la presentación y su transformación
  inversa del ratón deben ser idénticas. La barra va a la izquierda al 150% en
  1080p; recursos a la derecha, ventanas al 150%, ayudas al 200%.
- Conservar el PCX de consola al cargarlo. El juego libera el descriptor original
  después de crear las superficies: no leerlo como si siguiera vivo.
- Componer la UI en un lienzo separado y publicar un único fotograma terminado.
  No capturar las copias intermedias mientras `UI_BUILDING` está activo.
  Mantener el orden de ventanas, listas desplegables, ayudas y cursor.
- El fondo de recursos usa una máscara basada en RGB de la paleta y conexión
  con el borde; no basta con tratar el índice 0 como transparente.
- El callback restore-under no se ejecuta cuando el juego oculta el cursor.
  Renovar la capa de primer plano al terminar compose en ese caso, incluidos los
  ceros, sin borrar los textos directos del briefing. No reutilizar el cursor del
  frame anterior: al cambiar la paleta aparece como una flecha naranja.
- Tras publicar una imagen, redirigir también las primarias del intervalo sin
  raíces de menú y retener el último frame hasta disponer de contenido. Una
  superficie vacía o solo con cursor no es una pantalla de carga válida.
- Preparar la imagen inicial ampliada UNA vez por carga. Después solo copiar
  el fondo y componer las diferencias del texto nativo (Loading/Cargando).
  No volver a escalar millones de píxeles ni invalidar todos los controles de
  esta pantalla cada fotograma: retrasa la carga del juego.
- Las imágenes IA se incluyen como índices de la paleta nativa. El PNG de
  `assets/title-ai.png` es la fuente; el `.idx` se usa en ejecución.
- cnc-ddraw 7.1 presenta la imagen con sincronización de superficies. Los filtros
  CRT y PC tubo son GLSL originales en `assets/filters`; requieren OpenGL.
- Resolver rutas desde la ubicación del ejecutable. Si el paquete está junto al
  juego, esa carpeta tiene prioridad sobre una configuración antigua. No copiar
  `ddraw.dll` sobre sí misma: comparar rutas canónicas antes de copiar.
- `Session` conserva un diario en `session-backup`; restaurarlo al salir. No borrar
  un diario con `ready` ni sustituir archivos mientras haya una partida abierta.
  Nunca finalizar una partida del usuario para poder compilar o publicar.

## Verificación y entrega

Ejecuta el autotest de la build. Comprueba visualmente los cambios de dibujo en
una partida de prueba: barra, minimapa, recursos, ventanas superpuestas,
desplegables, ayudas, cursor y clics. `tools/check_shaders.py` compila ambos filtros
con el controlador local. Un arranque de 12 segundos no demuestra que una partida
completa o una campaña sea estable: no presentar esas afirmaciones como verificadas.

El README público solo contiene instalación, mejoras, imágenes y requisitos;
no añadas relatos de implementación ni resultados de pruebas. Conserva las
atribuciones MIT de los componentes externos. Publica únicamente el código,
activos del parche, imágenes elegidas y ZIP limpio: nunca EXE/DLL originales de
Blizzard, MPQ, partidas, volcados de memoria, registros, credenciales ni toolchains.
