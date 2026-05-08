use crate::codec::convert;
use crate::files::{collect_images, reserve_output};
use crate::settings::OutputFormat;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};

const MAX_WORKERS: usize = 4;

#[derive(Clone, Copy)]
struct ConvertPlan {
    short_side: Option<u32>,
    quality: f32,
    format: OutputFormat,
    keep_exif: bool,
}

pub enum Msg {
    Started(usize),
    FileOk(String, String),
    FileErr(String, String),
    Done {
        ok: usize,
        fail: usize,
        skipped: usize,
        cancelled: bool,
        out_dir: PathBuf,
    },
}

pub fn run_batch(
    folder: PathBuf,
    short_side: Option<u32>,
    quality: f32,
    format: OutputFormat,
    keep_exif: bool,
    cancel: Arc<AtomicBool>,
    tx: mpsc::Sender<Msg>,
) {
    let files = collect_images(&folder);
    let total = files.len();
    let _ = tx.send(Msg::Started(total));

    let out_dir = folder.join("converted");
    if total == 0 {
        let _ = tx.send(Msg::Done {
            ok: 0,
            fail: 0,
            skipped: 0,
            cancelled: false,
            out_dir,
        });
        return;
    }

    if let Err(e) = fs::create_dir_all(&out_dir) {
        let _ = tx.send(Msg::FileErr("output dir".into(), e.to_string()));
        let _ = tx.send(Msg::Done {
            ok: 0,
            fail: total,
            skipped: 0,
            cancelled: false,
            out_dir,
        });
        return;
    }

    let plan = ConvertPlan {
        short_side,
        quality,
        format,
        keep_exif,
    };
    let jobs = build_jobs(files, &out_dir, format.extension());
    let results = run_jobs(jobs, plan, &cancel, &tx);
    let ok = results
        .iter()
        .filter(|result| **result == Some(true))
        .count();
    let fail = results
        .iter()
        .filter(|result| **result == Some(false))
        .count();
    let skipped = results.iter().filter(|result| result.is_none()).count();

    let _ = tx.send(Msg::Done {
        ok,
        fail,
        skipped,
        cancelled: cancel.load(Ordering::Relaxed),
        out_dir,
    });
}

fn build_jobs(files: Vec<PathBuf>, out_dir: &Path, ext: &str) -> Vec<(PathBuf, PathBuf)> {
    let mut used = existing_output_names(out_dir);
    files
        .into_iter()
        .map(|src| {
            let stem = src
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let out = reserve_output(out_dir, &stem, ext, &mut used);
            (src, out)
        })
        .collect()
}

fn existing_output_names(out_dir: &Path) -> HashSet<String> {
    fs::read_dir(out_dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().to_str().map(str::to_lowercase))
        .collect()
}

fn run_jobs(
    jobs: Vec<(PathBuf, PathBuf)>,
    plan: ConvertPlan,
    cancel: &AtomicBool,
    tx: &mpsc::Sender<Msg>,
) -> Vec<Option<bool>> {
    match rayon::ThreadPoolBuilder::new()
        .num_threads(worker_count())
        .build()
    {
        Ok(pool) => pool.install(|| {
            jobs.par_iter()
                .map(|(src, out)| run_one(src, out, plan, cancel, tx))
                .collect()
        }),
        Err(_) => jobs
            .iter()
            .map(|(src, out)| run_one(src, out, plan, cancel, tx))
            .collect(),
    }
}

fn run_one(
    src: &Path,
    out: &Path,
    plan: ConvertPlan,
    cancel: &AtomicBool,
    tx: &mpsc::Sender<Msg>,
) -> Option<bool> {
    if cancel.load(Ordering::Relaxed) {
        return None;
    }

    let name = src
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    match convert(
        src,
        out,
        plan.short_side,
        plan.quality,
        plan.format,
        plan.keep_exif,
    ) {
        Ok(()) => {
            let out_name = out
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let _ = tx.send(Msg::FileOk(name, out_name));
            Some(true)
        }
        Err(e) => {
            let _ = tx.send(Msg::FileErr(name, e));
            Some(false)
        }
    }
}

fn worker_count() -> usize {
    std::thread::available_parallelism()
        .map(|workers| workers.get().clamp(1, MAX_WORKERS))
        .unwrap_or(2)
}
