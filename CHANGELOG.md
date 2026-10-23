# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.4.4] - 2026-10-23

img-convert now runs on eframe 0.37.0.

### Dependencies

- Bump `eframe` from 0.36.2 to 0.37.0

### Documentation & Links

- [README](https://github.com/fabianwimberger/img-convert#readme)

## [v0.4.3] - 2026-10-01

img-convert now runs on eframe 0.36.2 and no longer pulls in vulnerable quick-xml versions.

### Fixes

- Adapt dropped-folder handling to the eframe 0.36 file API

### Dependencies

- Bump `eframe` from 0.35.0 to 0.36.2
- Update `quick-xml` to 0.41.0 via `zbus_xml` and `wayland-scanner`, fixing RUSTSEC-2026-0194 and RUSTSEC-2026-0195
- Refresh the remaining locked dependencies

### Documentation & Links

- [README](https://github.com/fabianwimberger/img-convert#readme)

## [v0.4.2] - 2026-07-17

img-convert now runs on eframe 0.35.0 and fixes a real crossbeam-epoch security advisory found by cargo-audit.

### Fixes

- Bump `eframe` from 0.34.3 to 0.35.0
- Correct declared MSRV from 1.85 to 1.92 (the actual floor eframe 0.35.0 requires)
- Fix deprecated `Frame::show_inside` call, replaced with `Frame::show`

### Dependencies

- Bump `crossbeam-epoch` to 0.9.20, fixing RUSTSEC-2026-0204 (invalid pointer dereference)
- Align crate version metadata with the release version

### Documentation & Links

- [README](https://github.com/fabianwimberger/img-convert#readme)

## [v0.4.1] - 2026-05-09

img-convert now shows encoder stderr when conversion commands fail.

### Fixes

- Surface stderr from failed `cjpeg`, `avifenc`, `cjxl`, `heif-enc`, and `exiftool` runs in conversion errors
- Align crate version metadata with the release version

### Documentation & Links

- [README](https://github.com/fabianwimberger/img-convert#readme)

## [v0.4.0] - 2026-05-09

Alpha channel preservation, EXIF stripping fast path, stdin portability, and error propagation fixes.

### Fixes
- Alpha preservation fixed — output no longer flattens transparent pixels to a matte
- EXIF strip uses fast path when source lacks embedded thumbnail
- stdin reading works across macOS, Linux, and inside Docker
- Error propagation no longer panics on poisoned mutex state
- Dropped x86_64-apple-darwin from release matrix

### Documentation & Links
- https://github.com/fabianwimberger/img-convert

## [v0.3.0] - 2026-05-08

**The first release of a small cross-platform desktop GUI for batch resizing and compressing folders of images to JPG, AVIF, JXL, or HEIC.**

### Features

- **Batch processing** — convert an entire folder in parallel
- **Format choice** — JPG (`cjpeg`), AVIF (`avifenc`), JXL (`cjxl`), HEIC (`heif-enc`)
- **Wide input support** — JPEG, PNG, TIFF, WebP, GIF, BMP, ICO, TGA, QOI, PNM, EXR, HDR, DDS, Farbfeld
- **Lanczos3 resampling** — high-quality resize at the short side
- **JPEG → JXL fast path** — lossless transcode when no resize is needed
- **EXIF preservation** — copied via `exiftool` when available
- **Drag-and-drop** folder selection
- **Light / dark theme**
- **Cross-platform** — Linux, macOS (Apple Silicon), Windows

### Configuration

| Setting    | Options                                              |
|------------|------------------------------------------------------|
| Format     | JPG / AVIF / JXL / HEIC                              |
| Resolution | Original / High (3000 px) / Low (1440 px) short side |
| Quality    | High / Medium / Low (per-format quality mapping)     |
| EXIF       | Preserve / strip                                     |

### External Encoders

Only the encoders for formats you intend to use need to be on `$PATH`. The GUI shows availability and disables formats whose encoder is missing.

| Format | Command    | Typical package                    |
|--------|------------|------------------------------------|
| JPG    | `cjpeg`    | `mozjpeg` / `libjpeg-turbo`        |
| AVIF   | `avifenc`  | `libavif` / `libavif-utils`        |
| JXL    | `cjxl`     | `libjxl` / `jpeg-xl`               |
| HEIC   | `heif-enc` | `libheif` / `libheif-tools`        |
| EXIF   | `exiftool` | `perl-image-exiftool` / `exiftool` |

The Debian package below already lists these as `Recommends`.

### Downloads

| Platform                | Asset                                                  |
|-------------------------|--------------------------------------------------------|
| Linux x86_64            | `img-convert-v0.3.0-x86_64-unknown-linux-gnu.tar.gz`   |
| Debian / Ubuntu x86_64  | `img-convert_0.3.0-1_amd64.deb`                        |
| Windows x86_64          | `img-convert-v0.3.0-x86_64-pc-windows-msvc.zip`        |
| macOS (Apple Silicon)   | `img-convert-v0.3.0-aarch64-apple-darwin.tar.gz`       |

Checksums are in `SHA256SUMS`. Verify with `sha256sum -c SHA256SUMS`.

> macOS binaries are unsigned — on first launch run `xattr -dr com.apple.quarantine /path/to/img-convert` or right-click the binary and choose **Open**. Intel Macs are not covered by this release; install from source via `cargo install --git https://github.com/fabianwimberger/img-convert`.

### Documentation & Links

- README: https://github.com/fabianwimberger/img-convert#readme
- Pipeline overview: https://github.com/fabianwimberger/img-convert#pipeline
- Issues: https://github.com/fabianwimberger/img-convert/issues
