use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FavoriteLocation {
    pub label: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct NavigationState {
    pub current_path: PathBuf,
    pub back_stack: Vec<PathBuf>,
    pub forward_stack: Vec<PathBuf>,
    pub favorites: Vec<FavoriteLocation>,
}

impl NavigationState {
    pub fn new(start: PathBuf) -> Self {
        Self {
            current_path: start,
            back_stack: Vec::new(),
            forward_stack: Vec::new(),
            favorites: default_favorites(),
        }
    }

    pub fn visit(&mut self, next: PathBuf, current: PathBuf) {
        self.back_stack.push(current);
        self.current_path = next;
        self.forward_stack.clear();
    }

    pub fn back(&mut self) -> Option<PathBuf> {
        let prev = self.back_stack.pop()?;
        self.forward_stack.push(self.current_path.clone());
        self.current_path = prev.clone();
        Some(prev)
    }

    pub fn forward(&mut self) -> Option<PathBuf> {
        let next = self.forward_stack.pop()?;
        self.back_stack.push(self.current_path.clone());
        self.current_path = next.clone();
        Some(next)
    }
}

pub fn go_up_path(path: &Path) -> Option<PathBuf> {
    path.parent().map(|p| p.to_path_buf())
}

pub fn default_favorites() -> Vec<FavoriteLocation> {
    let mut out = Vec::new();
    if let Ok(home) = std::env::var("USERPROFILE") {
        let home = PathBuf::from(home);
        out.push(FavoriteLocation {
            label: "Desktop".to_string(),
            path: home.join("Desktop"),
        });
        out.push(FavoriteLocation {
            label: "Documents".to_string(),
            path: home.join("Documents"),
        });
        out.push(FavoriteLocation {
            label: "Downloads".to_string(),
            path: home.join("Downloads"),
        });
        out.push(FavoriteLocation {
            label: "Pictures".to_string(),
            path: home.join("Pictures"),
        });
    }
    out.push(FavoriteLocation {
        label: "This PC".to_string(),
        path: PathBuf::from("C:\\"),
    });
    out
}
