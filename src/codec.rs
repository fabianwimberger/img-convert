use crate::files::is_jpeg;
use crate::settings::OutputFormat;
use image::codecs::png::{CompressionType, FilterType as PngFilterType, PngEncoder};
use image::image_dimensions;
use image::imageops::FilterType;
use image::{DynamicImage, ExtendedColorType, ImageEncoder};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};

pub fn convert(
    src: &Path,
    out_path: &Path,
    short_side: Option<u32>,
    quality: f32,
    format: OutputFormat,
    keep_exif: bool,
) -> Result<(), String> {
    if format == OutputFormat::Jxl
        && is_jpeg(src)
        && command_available("cjxl")
        && let Ok((w, h)) = image_dimensions(src)
        && short_side.is_none_or(|target| w.min(h) <= target)
    {
        return transcode_jpeg_to_jxl(src, out_path, keep_exif);
    }

    let img = resize(open_image(src)?, short_side);
    match format {
        OutputFormat::Jpg => encode_jpeg(&img, out_path, quality)?,
        OutputFormat::Avif => encode_avif(&img, out_path, quality)?,
        OutputFormat::Jxl => encode_jxl(&img, out_path, quality)?,
        OutputFormat::Heic => encode_heic(&img, out_path, quality)?,
    }

    if keep_exif && command_available("exiftool") {
        copy_exif_with_exiftool(src, out_path)?;
    }

    Ok(())
}

pub fn resize(img: DynamicImage, short_side: Option<u32>) -> DynamicImage {
    let Some(short_side) = short_side else {
        return img;
    };

    let (w, h) = (img.width(), img.height());
    let short = w.min(h);
    if short <= short_side {
        return img;
    }

    let scale = short_side as f64 / short as f64;
    let new_w = (w as f64 * scale).round() as u32;
    let new_h = (h as f64 * scale).round() as u32;
    img.resize_exact(new_w, new_h, FilterType::Lanczos3)
}

pub fn command_available(cmd: &str) -> bool {
    static CACHE: OnceLock<Mutex<HashMap<String, bool>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    {
        let guard = cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(value) = guard.get(cmd).copied() {
            return value;
        }
    }
    let found = locate_command(cmd).is_some();
    let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
    guard.insert(cmd.to_owned(), found);
    found
}

fn locate_command(cmd: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let exts: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .ok()
            .map(|raw| raw.split(';').map(str::to_owned).collect())
            .unwrap_or_else(|| vec![".EXE".into(), ".CMD".into(), ".BAT".into()])
    } else {
        vec![String::new()]
    };

    for dir in std::env::split_paths(&path) {
        for ext in &exts {
            let candidate = dir.join(format!("{cmd}{ext}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn open_image(src: &Path) -> Result<DynamicImage, String> {
    image::open(src).map_err(|e| format!("decode: {e}"))
}

fn copy_exif_with_exiftool(src: &Path, dst: &Path) -> Result<(), String> {
    let status = Command::new("exiftool")
        .arg("-TagsFromFile")
        .arg(src)
        .arg("-all:all")
        .arg("-overwrite_original")
        .arg(dst)
        .status()
        .map_err(|e| format!("exiftool spawn: {e}"))?;
    if !status.success() {
        return Err(format!("exiftool exited with {status}"));
    }
    Ok(())
}

fn transcode_jpeg_to_jxl(src: &Path, dst: &Path, keep_exif: bool) -> Result<(), String> {
    let mut command = Command::new("cjxl");
    command.arg("--container=1");
    if !keep_exif {
        command.arg("--strip");
    }
    let status = command
        .arg(src)
        .arg(dst)
        .status()
        .map_err(|e| format!("cjxl spawn: {e}"))?;
    if !status.success() {
        return Err(format!("cjxl exited with {status}"));
    }
    Ok(())
}

fn ppm_bytes(img: &DynamicImage) -> Vec<u8> {
    let rgb = img.to_rgb8();
    let header = format!("P6\n{} {}\n255\n", rgb.width(), rgb.height());
    let mut ppm = Vec::with_capacity(header.len() + rgb.as_raw().len());
    ppm.extend_from_slice(header.as_bytes());
    ppm.extend_from_slice(rgb.as_raw());
    ppm
}

fn png_bytes(img: &DynamicImage) -> Result<Vec<u8>, String> {
    let mut png = Vec::new();
    let encoder = PngEncoder::new_with_quality(
        &mut png,
        CompressionType::Uncompressed,
        PngFilterType::NoFilter,
    );
    if img.color().has_alpha() {
        let rgba = img.to_rgba8();
        encoder
            .write_image(
                rgba.as_raw(),
                rgba.width(),
                rgba.height(),
                ExtendedColorType::Rgba8,
            )
            .map_err(|e| format!("png encode: {e}"))?;
    } else {
        let rgb = img.to_rgb8();
        encoder
            .write_image(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                ExtendedColorType::Rgb8,
            )
            .map_err(|e| format!("png encode: {e}"))?;
    }
    Ok(png)
}

fn run_with_stdin(mut command: Command, input: &[u8], name: &str) -> Result<(), String> {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("{name} spawn: {e}"))?;

    child
        .stdin
        .as_mut()
        .ok_or_else(|| format!("{name} stdin unavailable"))?
        .write_all(input)
        .map_err(|e| format!("{name} stdin write: {e}"))?;
    drop(child.stdin.take());

    let status = child.wait().map_err(|e| format!("{name} wait: {e}"))?;
    if !status.success() {
        return Err(format!("{name} exited with {status}"));
    }
    Ok(())
}

fn encode_jpeg(img: &DynamicImage, dst: &Path, quality: f32) -> Result<(), String> {
    let mut command = Command::new("cjpeg");
    command
        .arg("-quality")
        .arg(format!("{}", quality.round() as i32))
        .arg("-outfile")
        .arg(dst);
    run_with_stdin(command, &ppm_bytes(img), "cjpeg")
}

fn encode_avif(img: &DynamicImage, dst: &Path, quality: f32) -> Result<(), String> {
    let png = png_bytes(img)?;
    let mut command = Command::new("avifenc");
    command
        .arg("--qcolor")
        .arg(format!("{}", quality.round() as i32))
        .arg("--stdin")
        .arg("--input-format")
        .arg("png")
        .arg(dst);
    run_with_stdin(command, &png, "avifenc")
}

fn encode_jxl(img: &DynamicImage, dst: &Path, quality: f32) -> Result<(), String> {
    let mut command = Command::new("cjxl");
    command
        .arg("-q")
        .arg(format!("{}", quality.round() as i32))
        .arg("-e")
        .arg("7")
        .arg("--container=1")
        .arg("-")
        .arg(dst);
    run_with_stdin(command, &ppm_bytes(img), "cjxl")
}

fn encode_heic(img: &DynamicImage, dst: &Path, quality: f32) -> Result<(), String> {
    let tmp = TmpFile::new("tif");
    img.save(tmp.path())
        .map_err(|e| format!("tif write: {e}"))?;

    let status = Command::new("heif-enc")
        .arg("-q")
        .arg(format!("{}", quality.round() as i32))
        .arg(tmp.path())
        .arg("-o")
        .arg(dst)
        .status()
        .map_err(|e| format!("heif-enc spawn: {e}"))?;

    if !status.success() {
        return Err(format!("heif-enc exited with {status}"));
    }
    Ok(())
}

struct TmpFile {
    path: PathBuf,
}

impl TmpFile {
    fn new(ext: &str) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};

        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let name = format!(
            "img_convert_{}_{}.{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
            ext
        );
        Self {
            path: temp_root().join(name),
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TmpFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn temp_root() -> PathBuf {
    let shm = PathBuf::from("/dev/shm");
    if shm.is_dir() {
        shm
    } else {
        std::env::temp_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resize_targets_short_side() {
        let img = DynamicImage::new_rgb8(4000, 3000);
        let resized = resize(img, Some(1440));
        assert_eq!(resized.width().min(resized.height()), 1440);
        let ratio = resized.width() as f64 / resized.height() as f64;
        assert!((ratio - 4.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn resize_noop_when_already_small() {
        let img = DynamicImage::new_rgb8(800, 600);
        let resized = resize(img, Some(3000));
        assert_eq!(resized.width(), 800);
        assert_eq!(resized.height(), 600);
    }

    #[test]
    fn resize_handles_portrait() {
        let img = DynamicImage::new_rgb8(3000, 4000);
        let resized = resize(img, Some(1440));
        assert_eq!(resized.width(), 1440);
        assert_eq!(resized.height(), 1920);
    }

    #[test]
    fn resize_original_is_noop() {
        let img = DynamicImage::new_rgb8(3000, 4000);
        let resized = resize(img, None);
        assert_eq!(resized.width(), 3000);
        assert_eq!(resized.height(), 4000);
    }

    #[test]
    fn png_bytes_preserves_alpha() {
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            2,
            2,
            image::Rgba([255, 0, 0, 128]),
        ));
        let png = png_bytes(&img).unwrap();
        let decoded = image::load_from_memory(&png).unwrap();
        assert!(decoded.color().has_alpha());
    }

    #[test]
    fn png_bytes_rgb_has_no_alpha() {
        let img =
            DynamicImage::ImageRgb8(image::RgbImage::from_pixel(2, 2, image::Rgb([255, 0, 0])));
        let png = png_bytes(&img).unwrap();
        let decoded = image::load_from_memory(&png).unwrap();
        assert!(!decoded.color().has_alpha());
    }
}
