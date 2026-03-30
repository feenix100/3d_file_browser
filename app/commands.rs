use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum AppCommand {
    LoadDirectory(PathBuf),
    CreateFolder { parent: PathBuf, name: String },
    Rename { from: PathBuf, to_name: String },
    Delete { path: PathBuf },
}
