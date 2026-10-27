# La Guerra del Anillo HD

Parche de resoluciones para el ejecutable español **1.00.0010** de La Guerra del Anillo (2003).

[Descargar La Guerra del Anillo HD](https://github.com/jcarlosab/parches/raw/refs/heads/main/La-Guerra-del-Anillo/downloads/La-Guerra-del-Anillo-HD.zip)

## Instalación

Con el juego cerrado, guarda una copia de `Rings.ini` y copia `d3d9.dll` y `WotR-HD.ini` junto a `Rings.exe`. Abre el juego y selecciona la resolución en **Opciones → Audio / Vídeo → Resolución**; pulsa **Aplicar**.

El parche añade 1280×720, 1920×1080 y 2560×1440 a los modos reconocidos. El selector muestra las resoluciones que anuncia la tarjeta gráfica, conservando todos los modos originales. No cambia `Rings.exe` en disco. No necesita un launcher.

También corrige el cierre del selector de campaña causado por el contador de una animación cuando Windows lleva más de diez días encendido.

Esta DLL sustituye al parche de 1080p externo; no se deben combinar ambas DLL. Su `d3d9.ini` no se utiliza.

## Cámara

En panorámico, el campo de visión pasa de 45° a 60° para mostrar más terreno, conservando el tamaño de la interfaz. Los menús y las escenas con bandas mantienen el campo original. Puedes cambiar `CameraDegrees` en `WotR-HD.ini` entre 45 y 75, o usar `EnableCamera=0` para conservar la cámara original; reinicia el juego para aplicar el cambio.

## Desinstalación

Selecciona primero una resolución original, cierra el juego y retira `d3d9.dll`. Si el juego no arranca, recupera `Rings.ini` de la copia de seguridad. `WotR-HD.log` registra si el ejecutable es compatible y si se aplicó el parche.

Este proyecto no incluye el juego. Los derechos del juego y sus recursos pertenecen a sus respectivos titulares.
