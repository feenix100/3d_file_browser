use std::path::Path;

pub fn path_to_breadcrumbs(path: &Path) -> Vec<String> {
    path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .map(|part| format!("> {}", part))
        .collect()
}
