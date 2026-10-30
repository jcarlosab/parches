# Praetorians — Opciones extendidas

Utilidad independiente en Rust para Praetorians Mods Complex en Windows. Conserva el menú del juego y permite ajustar reglas del ejército con una interfaz propia.

[Descargar Opciones extendidas](https://github.com/jcarlosab/parches/raw/refs/heads/main/Praetorians/downloads/Praetorians-Opciones-extendidas.zip)

## Instalación y uso

1. Descarga y extrae el ZIP.
2. Copia `Opciones extendidas.exe` a la carpeta donde está `Praetorians.exe`.
3. Con el juego cerrado, abre la herramienta, selecciona las opciones y pulsa **Aplicar cambios**.
4. Inicia una partida nueva. El botón **Abrir juego** usa los ajustes ya aplicados; no aplica selecciones pendientes.

No requiere instalar Rust para utilizarla. Necesitas tu propia instalación de Mods Complex; el paquete no contiene el juego ni sus archivos de datos.

## Opciones

- **Clásico:** 500 tropas / 50 grupos por jugador.
- **Batalla grande:** 750 tropas / 75 grupos.
- **Batalla masiva:** 1000 tropas / 100 grupos.
- **Población inagotable en las aldeas:** reclutar no descuenta habitantes y cancelar no añade población artificialmente. Afecta a jugadores e IA.
- Muestra los límites actuales, detecta cambios del ejecutable y permite restaurar el límite original.
- Icono propio y copias de recuperación automáticas.

El límite de tropas es total por jugador: no cambia cuántos soldados forman cada grupo. La opción de aldeas conserva las comprobaciones para aceptar pedidos: una aldea vacía o con población insuficiente sigue sin poder aceptar ese pedido. No elimina el coste de honor ni el tiempo de entrenamiento.

## Compatibilidad y recuperación

La herramienta comprueba la versión exacta del ejecutable y solo modifica instrucciones conocidas. Admite las cuatro variantes originales de pantalla de Mods Complex (4:3, 16:9, 16:10 y XxX), de 3.551.232 bytes. Rechaza ejecutables desconocidos o con cambios ajenos.

Antes de cada modificación guarda el ejecutable anterior en `OpcionesUnidades.Backups`. Para deshacer las opciones, cierra el juego, selecciona 500/50, desmarca población inagotable y aplica. Para recuperar una copia manualmente, conserva primero el ejecutable actual y copia el `Praetorians.exe` del respaldo deseado a la carpeta del juego.

Mod Complex puede sustituir el ejecutable al cambiar resolución u otros ajustes. Si ocurre, vuelve a abrir esta herramienta y reaplica las opciones. No modifica la resolución, el registro de Windows, los menús ni los paquetes de mods. No incluye la corrección de la barra de carga.

## Estado de validación

Versión publicada: 30 de octubre de 2026. Cinco pruebas automatizadas verifican límites, restauración, rechazo de archivos incompatibles y combinaciones con las cuatro variantes de pantalla. También se han comprobado 64 casos de instrucciones nativas de consumo y cancelación. La opción de población inagotable todavía no está validada en una partida completa.

## Código fuente

Disponible en [source](source). Para compilar se necesita Rust para Windows, el destino GNU x86-64 y `windres` en PATH:

```powershell
cd source
cargo build --release --locked
```

Las pruebas de compatibilidad requieren una instalación propia en `C:/Games/Praetorians` con las cuatro copias originales `Praetorians_*_exe.bak`. No se distribuyen esas copias.

El emblema incluido se ha generado para esta herramienta. Praetorians y Mods Complex pertenecen a sus respectivos autores; esta utilidad es un proyecto independiente.
