# miUPV 🎓🦀

> **Herramienta de línea de comandos y librería en Rust para los servicios universitarios de la Universitat Politècnica de València (UPV).**

[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Vibecoded with Antigravity](https://img.shields.io/badge/Vibecoded%20with-Antigravity%20(Google%20DeepMind)-7928CA.svg)](https://deepmind.google/)

---

### ⚡ Sobre el desarrollo del proyecto

Este proyecto fue **100% vibecodeado utilizando [Antigravity](https://deepmind.google/) (el asistente agente de Google DeepMind)**, **sin tener experiencia previa en Rust**.

Todo el proceso de ingeniería inversa de la app oficial de Android de la UPV, extracción de secretos OAuth 2.0 PKCE, ingeniería de protocolos de Oracle PL/SQL y PoliformaT (Sakai), mitigación de cuellos de botella de red, arquitectura *Stale-While-Revalidate* y diseño de la interfaz interactiva de terminal (TUI) con Ratatui fue guiado y desarrollado iterativamente en pair-programming con el agente.

---

## 🚀 Características

- **Autenticación Oficial & Autorrenovación 100% Autónoma:**
  - Réplica en software de la máquina de estados de 2 clientes OAuth 2.0 PKCE de la app oficial:
    - **Cliente 1 (`CR8FB51...`)**: Password Grant con DNI para emitir el **Refresh Token de 30 días**.
    - **Cliente 2 (`JI995FA...`)**: Token Exchange para emitir el **Access Token Bearer (24h)**.
  - **Autorrenovación permanente y transparente**: Renueva automáticamente la sesión mensual sin intervención manual ni necesidad de volver a loguearse o capturar tokens.
  - Soporte alternativo de login web mediante **CAS SSO** (Intranet UPV).
  - Almacenamiento seguro mediante llavero del sistema o permisos restringidos de usuario `0600` (`~/.config/miupv/credentials.json`).
- **Calendario Académico Interactivo (TUI) con Caché Instantánea:**
  - Interfaz de terminal con navegación por teclado (`ratatui`).
  - Arquitectura *Stale-While-Revalidate*: Abre en **< 12 ms** mostrando el horario en caché y sincroniza en segundo plano con el servidor de la UPV sin bloquear.
  - Conversión automática de timestamps UTC a hora peninsular española.
  - Exportación estándar a formato RFC 5545 (`.ics`) para Google Calendar / Apple Calendar.
- **Calificaciones y Asignaturas:** Consulta tus notas, convocatorias, actas y créditos de cada curso.
- **PoliformaT (Sakai LMS):** Consulta de tareas activas y entregadas, asignaturas de clase y avisos docentes.
- **Servei d'Esports:** Consulta de catálogo deportivo y disponibilidad de pistas (pádel, tenis, baloncesto, fútbol) en los campus de Vera, Gandia y Alcoy.
- **Cabinas de Estudio:** Consulta y reserva de puestos y cabinas en Biblioteca Central y resto de bibliotecas UPV.
- **Carnet Virtual (TUI) y Código QR:** Renderizado en terminal del código QR oficial para acceso por tornos y préstamos.
- **Tutorías Docentes:** Listado de asignaturas, profesorado asignado y tutorías solicitadas/programadas.
- **Wake-on-LAN (WOL):** Encendido remoto de ordenadores de laboratorios asignados.
- **Autocompletado Contextual:** Soporte para autocompletado inteligente con `Tab` en Zsh, Bash y Fish.

---

## 📋 Requisitos

### Sistema Operativo
- **Linux** (probado en Arch, Ubuntu, Debian, Fedora) o **macOS**.

### Dependencias de compilación
1. **Rust y Cargo** (versión 1.75 o superior recomendada):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **Librerías del sistema:**
   - En distribuciones basadas en Debian/Ubuntu:
     ```bash
     sudo apt update && sudo apt install -y pkg-config libssl-dev libdbus-1-dev
     ```
   - En Arch Linux:
     ```bash
     sudo pacman -S --needed pkgconf openssl dbus
     ```
   - En Fedora:
     ```bash
     sudo dnf install -y pkgconf openssl-devel dbus-devel
     ```

---

## 📦 Instalación

### Opción A: Binarios precompilados (GitHub Releases)
Puedes descargar directamente los ejecutables listos para usar desde la sección de **[Releases](https://github.com/Manel-Romero/miUPV/releases)**:
- **Linux x86_64** (`x86_64-unknown-linux-gnu`)
- **macOS Apple Silicon** (M1/M2/M3/M4, `aarch64-apple-darwin`)
- **macOS Intel** (`x86_64-apple-darwin`)
- **Windows x64** (`x86_64-pc-windows-msvc`)

Descomprime el archivo y añade el ejecutable `miupv` a tu `PATH`.

### Opción B: Compilación desde código fuente

1. Clonar el repositorio:
```bash
git clone https://github.com/Manel-Romero/miUPV.git
cd miUPV
```

2. Compilar e instalar el binario CLI:
```bash
cargo install --path miupv-cli
```
*(Esto compilará el ejecutable optimizado en release y lo colocará en `~/.cargo/bin/miupv`)*.

Asegúrate de tener `~/.cargo/bin` en tu variable de entorno `PATH`:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### 3. Instalar autocompletado para la terminal (Tab)
Ejecuta el asistente interactivo de autocompletado:
```bash
miupv completions --install
```
Recarga tu shell (`source ~/.zshrc` o `source ~/.bashrc`) y prueba a pulsar `miupv <Tab>`.

---

## 🔐 Primer Inicio de Sesión

Inicia sesión con tu DNI y tu contraseña institucional de la UPV:
```bash
miupv login
```
El sistema ejecutará automáticamente el handshake OAuth 2.0 oficial, emitirá los tokens de sesión y vinculará tus credenciales con seguridad (permisos `0600`) para que la sesión se autorrenueve cada 30 días automáticamente.

Para comprobar el estado de tu sesión:
```bash
miupv auth status
```

Para forzar una renovación inmediata del Refresh Token:
```bash
miupv auth renew
```

---

## 🛠️ Ejemplos de Uso

### Calendario y Horario
```bash
# Abre la interfaz visual interactiva (TUI) con arranque instantáneo
miupv calendario

# O exporta todo tu semestre a un archivo .ics estándar
miupv calendario export --out mi_horario.ics
```

### Calificaciones y Notas
```bash
# Consulta las notas del curso actual
miupv notas

# Formato JSON para integraciones o scripts
miupv notas --json
```

### PoliformaT (Tareas y Entregas)
```bash
# Consulta tus tareas pendientes
miupv tareas

# Ver todas las tareas incluyendo entregadas o calificadas
miupv tareas --all
```

### Pistas Deportivas
```bash
# Consultar pistas libres de baloncesto para hoy
miupv deportes disponibles --deporte basketball

# Consultar pistas de tenis o pádel
miupv deportes disponibles --deporte padel
miupv deportes disponibles --deporte tenis
```

### Cabinas de Biblioteca
```bash
# Ver reservas activas de cabinas
miupv cabinas mis-reservas

# Consultar disponibilidad para hoy
miupv cabinas
```

### Carnet Universitario y Tornos
```bash
# Muestra tus datos de la tarjeta virtual TUI
miupv carnet info

# Dibuja el código QR en la terminal para pasar tornos
miupv carnet qr
```

### Tutorías
```bash
# Lista de asignaturas y profesorado asignado
miupv tutorias asignaturas

# Estado de tus tutorías solicitadas
miupv tutorias
```

---

## 🏗️ Arquitectura del Repositorio

El proyecto está organizado como un Workspace modular de Cargo:

```text
miUPV/
├── miupv-core/        # Librería núcleo en Rust (deserializadores, OAuth PKCE, llamadas API)
│   ├── src/
│   │   ├── client.rs       # Cliente HTTP con soporte UTF-8 / Latin1 y autenticación
│   │   ├── pkce.rs         # Generación segura de pares RFC 7636 (S256)
│   │   ├── services/       # Implementación modular de cada servicio UPV
│   │   └── models/         # Tipos y modelos fuertemente tipados
│   └── tests/              # Suite de pruebas unitarias y de integración
├── miupv-cli/         # Aplicación de terminal
│   ├── src/
│   │   ├── commands/       # Handlers de cada comando CLI y TUI del calendario
│   │   ├── ui.rs           # Spinners, tablas comfy-table y estilos
│   │   └── main.rs         # CLI entrypoint con clap
├── Cargo.toml         # Configuración del workspace de Cargo
└── README.md
```

---

## ⚖️ Licencia y Descargo de Responsabilidad

Este proyecto se distribuye bajo la licencia **MIT**. Consulta el archivo [LICENSE](LICENSE) para más detalles.

*Descargo de responsabilidad: Este es un proyecto independiente desarrollado por estudiantes/alumnos para mejorar la experiencia de usuario y accesibilidad en entornos de escritorio y terminal. No es una herramienta oficial de la Universitat Politècnica de València.*
