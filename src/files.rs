use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "jpe", "jif", "jfif", "png", "apng", "tif", "tiff", "webp", "gif", "bmp", "dib",
    "ico", "tga", "icb", "vda", "vst", "ff", "farbfeld", "qoi", "pbm", "pgm", "ppm", "pnm", "pam",
    "exr", "hdr", "dds", "jxl",
];

pub fn collect_images(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && is_supported(path))
        .collect();
    files.sort();
    files
}

pub fn is_supported(path: &Path) -> bool {
    extension(path).is_some_and(|ext| SUPPORTED_EXTENSIONS.contains(&ext.as_str()))
}

pub fn is_jpeg(path: &Path) -> bool {
    extension(path)
        .is_some_and(|ext| matches!(ext.as_str(), "jpg" | "jpeg" | "jpe" | "jif" | "jfif"))
}

pub fn is_jxl(path: &Path) -> bool {
    extension(path).is_some_and(|ext| ext == "jxl")
}

pub fn reserve_output(
    out_dir: &Path,
    stem: &str,
    ext: &str,
    used: &mut HashSet<String>,
) -> PathBuf {
    for index in 0usize.. {
        let name = if index == 0 {
            format!("{stem}.{ext}")
        } else {
            format!("{stem}_{index}.{ext}")
        };

        if used.insert(name.to_lowercase()) {
            return out_dir.join(name);
        }
    }

    unreachable!("unbounded suffix search always returns")
}

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_supported_accepts_common_formats() {
        for name in [
            "a.jpg", "a.JPG", "a.jpeg", "a.png", "a.tif", "a.TIFF", "a.webp", "a.gif", "a.bmp",
            "a.ico", "a.tga", "a.exr", "a.hdr", "a.qoi", "a.jxl",
        ] {
            assert!(is_supported(Path::new(name)), "{name}");
        }
        assert!(!is_supported(Path::new("a.txt")));
        assert!(!is_supported(Path::new("a")));
    }

    #[test]
    fn reserve_output_deduplicates() {
        let dir = Path::new("/tmp/out");
        let mut used = HashSet::new();
        assert_eq!(
            reserve_output(dir, "photo", "jpg", &mut used)
                .file_name()
                .unwrap(),
            "photo.jpg"
        );
        assert_eq!(
            reserve_output(dir, "photo", "jpg", &mut used)
                .file_name()
                .unwrap(),
            "photo_1.jpg"
        );
        assert_eq!(
            reserve_output(dir, "photo", "jpg", &mut used)
                .file_name()
                .unwrap(),
            "photo_2.jpg"
        );
    }

    #[test]
    fn reserve_output_is_case_insensitive() {
        let dir = Path::new("/tmp/out");
        let mut used = HashSet::new();
        used.insert("photo.jpg".into());
        let path = reserve_output(dir, "Photo", "jpg", &mut used);
        assert_eq!(path.file_name().unwrap(), "Photo_1.jpg");
    }

    #[test]
    fn reserve_output_uses_format_extension() {
        let dir = Path::new("/tmp/out");
        let mut used = HashSet::new();
        assert_eq!(
            reserve_output(dir, "photo", "avif", &mut used)
                .file_name()
                .unwrap(),
            "photo.avif"
        );
        assert_eq!(
            reserve_output(dir, "photo", "jxl", &mut used)
                .file_name()
                .unwrap(),
            "photo.jxl"
        );
        assert_eq!(
            reserve_output(dir, "photo", "heic", &mut used)
                .file_name()
                .unwrap(),
            "photo.heic"
        );
    }
}
