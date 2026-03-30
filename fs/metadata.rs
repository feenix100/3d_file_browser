use std::fs::Metadata;
use std::path::Path;

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

pub fn is_hidden(_path: &Path, metadata: &Metadata) -> bool {
    #[cfg(target_os = "windows")]
    {
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        return metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0;
    }

    #[cfg(not(target_os = "windows"))]
    {
        _path.file_name()
            .and_then(|f| f.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
    }
}
