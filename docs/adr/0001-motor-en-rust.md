# ADR 0001: el motor es Rust dentro del proceso Tauri

Estado: aceptada (17 de noviembre de 2026).

## Contexto

El usuario pidió evaluar Svelte con Python. Las sesiones de Steam, GOG y Epic viven en WebView2, y
algunas cookies HttpOnly solo se leen con las APIs nativas del webview.

## Decisión

Toda la lógica de dominio, red, sesión y persistencia va en Rust, dentro del proceso de Tauri.
Svelte solo pinta. No hay sidecar Python ni Node.

## Consecuencias

- Los secretos no cruzan ninguna frontera entre procesos.
- Un solo binario que firmar, sin PyInstaller ni falsos positivos de antivirus.
- Legendary (GPLv3) no se empaqueta. El protocolo de Epic se reimplementa leyendo su código sin
  copiarlo. Si la cobertura es insuficiente, el plan B (sidecar solo para la biblioteca Epic) lo
  decide el usuario con la evidencia de la fase 1.
- Alternativas descartadas: TypeScript como motor (secretos en la UI, CORS) y sidecar Node.
