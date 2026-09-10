# 🚀 MukaLuJauh

<p align="center">
  <b>Fast, Privacy-first Face Unlock for Linux (Ubuntu) & Windows</b><br>
  <i>Reverse-engineered & rewritten in Rust from macOS Glance</i>
</p>

---

## 📥 Download & GUI Installers

Tersedia installer siap pakai untuk setiap sistem operasi di halaman [**Releases**](https://github.com/endrisusanto/MukaLuJauh/releases):

| OS | Format Installer | Cara Install |
|---|---|---|
| 🐧 **Ubuntu / Debian** | **`mukalujauh-amd64.deb`** | `sudo dpkg -i mukalujauh-amd64.deb` (atau double click file `.deb`) |
| 🪟 **Windows (10/11)** | **`mukalujauh-windows-x86_64-installer.exe`** | Double-click Setup Wizard Installer (GUI NSIS) |
| 🍎 **macOS (Apple Silicon)** | **`mukalujauh-macos-arm64.dmg`** | Double-click `.dmg` dan drag ke Applications |
| 📦 **Portable Binaries** | `.tar.gz` / `.zip` | Ekstrak dan jalankan langsung tanpa instalasi |

---

## ✨ Features

- **Blazing Fast & Ultra Lightweight:** Ditulis menggunakan Rust murni dengan konsumsi RAM dan CPU sangat rendah.
- **Privacy First & Zero Cloud:** Pemrosesan biometrik 100% on-device. Gambar webcam tidak pernah disimpan ke disk.
- **AES-256-GCM Encrypted Storage:** Vektor embedding 512-dimensi disimpan dalam bentuk terenkripsi kuat.
- **Multi-Angle Head Enrollment:** Merekam 9 sudut pose kepala dan menghitung *centroid* vektor untuk akurasi tinggi.
- **Anti-Spoofing / Liveness Check:** Mendeteksi micro-motion 3D, kedipan mata (*Eye Aspect Ratio*), pantulan cahaya (*glare*), dan deteksi frame layar HP (*bezel cue*).

---

## 🛠️ CLI Commands

Setelah diinstal, perintah `mukalujauh` dapat dipanggil dari terminal / PowerShell:

```bash
# 1. Daftarkan wajah baru (9 sudut pose)
mukalujauh enroll --name "Endri"

# 2. Tes verifikasi wajah & liveness check
mukalujauh verify

# 3. List profil wajah terdaftar
mukalujauh list

# 4. Jalankan daemon pemantau lock screen
mukalujauh daemon
```

---

## 🚀 Rilis Versi Baru (Auto Bump & Release)

```bash
# Otomatis naikkan patch (misal: v0.1.2) & trigger build installer multi-OS
./scripts/bump_and_release.sh patch
```
