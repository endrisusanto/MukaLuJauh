# 🚀 MukaLuJauh

<p align="center">
  <b>Fast, Privacy-first Face Unlock for Linux (Ubuntu) & Windows</b><br>
  <i>Reverse-engineered & rewritten in Rust from macOS Glance</i>
</p>

---

## ✨ Features

- **Blazing Fast & Ultra Lightweight:** Ditulis menggunakan Rust murni dengan konsumsi RAM dan CPU sangat rendah.
- **Privacy First & Zero Cloud:** Pemrosesan biometrik 100% on-device. Gambar webcam tidak pernah disimpan ke disk.
- **AES-256-GCM Encrypted Storage:** Vektor embedding 512-dimensi disimpan dalam bentuk terenkripsi kuat.
- **Multi-Angle Head Enrollment:** Merekam 9 sudut pose kepala dan menghitung *centroid* vektor untuk akurasi tinggi.
- **Anti-Spoofing / Liveness Check:** Mendeteksi micro-motion 3D, kedipan mata (*Eye Aspect Ratio*), pantulan cahaya (*glare*), dan deteksi frame layar HP (*bezel cue*).
- **Multi-Platform Support:** Siap dijalankan di **Ubuntu (Linux via PAM / systemd-logind)** dan **Windows**.

---

## 📦 Project Structure

```
mukalujauh/
├── .github/
│   └── workflows/
│       └── release.yml          # CI/CD multi-OS automated binary release
├── scripts/
│   └── bump_and_release.sh      # Script auto version bump, commit, tag & push
├── src/
│   ├── main.rs                  # CLI entrypoint & commands
│   ├── model.rs                 # 512-d Face vectors & centroid math
│   ├── pipeline.rs              # Cosine similarity face matching
│   ├── liveness.rs              # Anti-spoofing engine (motion, glare, bezel)
│   ├── crypto.rs                # AES-256-GCM encryption & key derivation
│   ├── config.rs                # Settings & TOML configuration manager
│   └── auth.rs                  # OS lock screen & PAM hooks
├── Cargo.toml
└── README.md
```

---

## 🛠️ CLI Usage

### 1. Inisialisasi & Daftar Wajah (Enrollment)
```bash
# Daftarkan wajah Anda dengan 9 sudut pose
cargo run -- enroll --name "Endri"
```

### 2. Cek Verifikasi Wajah (Verification Test)
```bash
cargo run -- verify
```

### 3. List Profil Terdaftar
```bash
cargo run -- list
```

### 4. Background Daemon (Lock Screen Monitor)
```bash
cargo run -- daemon
```

---

## 🚀 Automated Version Bump & GitHub Release

Script `scripts/bump_and_release.sh` secara otomatis:
1. Menghitung kenaikan versi SemVer (`patch`, `minor`, `major`).
2. Mengupdate `Cargo.toml` & memvalidasi build `cargo check`.
3. Membuat auto conventional commit (contoh: `chore(release): bump version to v0.1.1`).
4. Membuat Git tag (`v0.1.1`).
5. Mem-push commit dan tag ke repo [endrisusanto/MukaLuJauh](https://github.com/endrisusanto/MukaLuJauh).
6. Memicu GitHub Actions untuk build binary `.tar.gz` (Linux/macOS) & `.zip` (Windows) beserta checksum SHA256.

```bash
# Auto bump patch (0.1.0 -> 0.1.1)
./scripts/bump_and_release.sh patch

# Auto bump minor (0.1.0 -> 0.2.0)
./scripts/bump_and_release.sh minor

# Custom version
./scripts/bump_and_release.sh v1.0.0 "feat(release): first production ready release"
```

---

## 📄 License
MIT License.
