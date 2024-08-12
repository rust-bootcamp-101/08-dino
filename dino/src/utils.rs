use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{self},
    path::{Path, PathBuf},
};

use anyhow::Result;
use bundler::run_bundle;
use glob::{glob, GlobError};

use crate::BUILD_DIR;

// get all files with certain extension in a directory
pub(crate) fn get_files_with_exts(dir: &str, exts: &[&str]) -> Result<BTreeSet<PathBuf>> {
    let mut files = BTreeSet::new();
    for ext in exts {
        let rule = format!("{}/**/*.{}", dir, ext);
        let paths = glob(&rule)?.collect::<Result<BTreeSet<PathBuf>, GlobError>>()?;
        files.extend(paths);
    }
    Ok(files)
}

pub(crate) fn calc_project_hash(dir: &str) -> Result<String> {
    let hash = calc_hash_for_files(dir, &["ts", "js", "json", "yml"], 16)?;
    Ok(hash)
}

pub(crate) fn calc_hash_for_files(dir: &str, exts: &[&str], expect_len: usize) -> Result<String> {
    let files = get_files_with_exts(dir, exts)?;
    let mut hasher = blake3::Hasher::new();
    for file in files {
        hasher.update_reader(File::open(file)?)?;
    }
    let mut ret = hasher.finalize().to_string();
    ret.truncate(expect_len);
    Ok(ret)
}

pub(crate) fn build_project(dir: &str) -> Result<String> {
    fs::create_dir_all(BUILD_DIR)?;
    let hash = calc_project_hash(dir)?;
    // 注意生成的文件使用.mjs 目的是为了避免与.js文件 会被拿去build，导致生成的文件也会被拿去build
    let filename = format!("{}/{}.mjs", BUILD_DIR, hash);
    let config = format!("{}/{}.yml", BUILD_DIR, hash);

    // if the file already exists, skip building
    let dst = Path::new(&filename);
    if dst.exists() {
        return Ok(filename);
    }

    remove_dir_contents(BUILD_DIR)?;

    // build the project
    let content = run_bundle("main.ts", &Default::default())?;
    fs::write(dst, content)?;
    let mut dst = File::create(&config)?;
    let mut src = File::open("config.yml")?;
    io::copy(&mut src, &mut dst)?;
    Ok(filename)
}

// https://stackoverflow.com/questions/65573245/
fn remove_dir_contents<P: AsRef<Path>>(path: P) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        fs::remove_file(entry?.path())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_files_with_exts_should_work() -> Result<()> {
        let files = get_files_with_exts("fixtures/prj", &["ts", "js", "json"])?;
        assert_eq!(files.len(), 4);
        assert_eq!(
            files.into_iter().collect::<Vec<_>>(),
            [
                PathBuf::from("fixtures/prj/a.ts"),
                PathBuf::from("fixtures/prj/test1/b.ts"),
                PathBuf::from("fixtures/prj/test1/c.js"),
                PathBuf::from("fixtures/prj/test2/test3/d.json"),
            ]
        );
        Ok(())
    }

    #[test]
    fn calc_hash_for_files_should_work() -> Result<()> {
        let hash = calc_hash_for_files("fixtures/prj", &["ts", "js", "json"], 8)?;
        assert_eq!(hash, "af1349b9");
        Ok(())
    }
}
