//! Functions to query filesystem for files and directories
use anyhow::Result;
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};
use tap::Pipe;

/// Get all files in SwiftFileList file.
pub fn get_files_list<T, P>(file_lists: P) -> Result<Vec<T>>
where
    T: From<String>,
    P: AsRef<Path>,
{
    read_to_string(file_lists)?
        .pipe(|s| shell_words::split(&s))?
        .into_iter()
        .map(T::from)
        .collect::<Vec<_>>()
        .pipe(Result::Ok)
}

/// Find All swift files in a directory.
#[tracing::instrument(skip_all)]
pub fn find_swift_files(project_root: &Path) -> Result<Vec<String>> {
    wax::walk("**/*.swift", project_root, usize::MAX)?
        .enumerate()
        .map(|(i, entry)| {
            entry.ok()?.path().to_str()?.to_string().pipe(|path| {
                tracing::trace!("{i}: {path}");
                Some(path)
            })
        })
        .flatten()
        .collect::<Vec<_>>()
        .pipe(Result::Ok)
}

/// Is the given directory is a directory and has .git?
#[tracing::instrument]
fn is_project_root(directory: &Path) -> bool {
    if directory.is_dir() {
        directory.join(".git").exists()
    } else {
        tracing::warn!("Not a directory");
        false
    }
}

/// Find Header directory and frameworks from path.
#[tracing::instrument(ret, skip_all)]
pub fn find_header_dirs(root: &Path) -> Result<(Vec<String>, Vec<String>)> {
    wax::walk("**/*.h", root, usize::MAX)?
        .flatten()
        .enumerate()
        .map(|(i, entry)| {
            entry
                .path()
                .ancestors()
                .find(|p| p.extension().eq(&Some("framework".as_ref())))
                .pipe(|p| {
                    if let Some(path) = p {
                        let framework = path.file_name()?.to_str()?.to_string();
                        tracing::trace!("Framework {i}: {framework}");
                        Some((framework.into(), None))
                    } else {
                        let dir = entry.path().parent()?.file_name()?.to_str()?.to_string();
                        tracing::trace!("Directory {i}: {dir}");
                        Some((None, dir.into()))
                    }
                })
        })
        .flatten()
        .unzip()
        .pipe(|(dirs, frameworks): (Vec<_>, Vec<_>)| {
            let dirs = dirs.into_iter().flatten().collect();
            let frameworks = frameworks.into_iter().flatten().collect();
            Ok((dirs, frameworks))
        })
}

/// Find directory, swiftflags and comple file from a path to file within a project.
#[tracing::instrument(ret)]
pub fn find_swift_module_root(
    file_path: &Path,
) -> (Option<PathBuf>, Option<PathBuf>, Option<PathBuf>) {
    let mut compile_file = None;
    let mut directory = match file_path.parent() {
        Some(directory) => directory,
        None => return (None, None, None),
    };

    while directory.components().count() > 1 {
        let path = match directory.parent() {
            Some(path) => path,
            None => break,
        };

        let flag_path = path.join(".swiftflags");
        if flag_path.is_file() {
            return (Some(directory.to_path_buf()), Some(flag_path), compile_file);
        };

        if compile_file.is_none() {
            path.join(".compile")
                .pipe(|p| p.is_file().then(|| compile_file = p.into()));
        };

        if is_project_root(directory) {
            return (Some(directory.to_path_buf()), None, compile_file);
        } else {
            directory = path;
        }
    }

    (Some(directory.to_path_buf()), None, compile_file)
}
