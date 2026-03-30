pub mod entries;
pub mod metadata;
pub mod navigation;
pub mod ops;
pub mod search;
pub mod watch;

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use entries::DirectorySnapshot;
use ops::{create_folder, delete_entry, rename_entry};

#[derive(Debug, Clone)]
pub enum FsRequest {
    LoadDirectory(std::path::PathBuf),
    CreateFolder {
        parent: std::path::PathBuf,
        name: String,
    },
    Rename {
        from: std::path::PathBuf,
        to_name: String,
    },
    Delete {
        path: std::path::PathBuf,
    },
}

#[derive(Debug)]
pub enum FsResponse {
    DirectoryLoaded(anyhow::Result<DirectorySnapshot>),
    OperationFinished(anyhow::Result<String>),
}

pub struct FsService {
    tx: Sender<FsRequest>,
    rx: Receiver<FsResponse>,
}

impl FsService {
    pub fn start() -> Self {
        let (req_tx, req_rx) = mpsc::channel::<FsRequest>();
        let (resp_tx, resp_rx) = mpsc::channel::<FsResponse>();

        thread::spawn(move || {
            while let Ok(request) = req_rx.recv() {
                let response = match request {
                    FsRequest::LoadDirectory(path) => {
                        FsResponse::DirectoryLoaded(entries::read_directory_snapshot(&path))
                    }
                    FsRequest::CreateFolder { parent, name } => {
                        FsResponse::OperationFinished(create_folder(&parent, &name))
                    }
                    FsRequest::Rename { from, to_name } => {
                        FsResponse::OperationFinished(rename_entry(&from, &to_name))
                    }
                    FsRequest::Delete { path } => {
                        FsResponse::OperationFinished(delete_entry(&path))
                    }
                };
                let _ = resp_tx.send(response);
            }
        });

        Self {
            tx: req_tx,
            rx: resp_rx,
        }
    }

    pub fn send(&self, req: FsRequest) {
        let _ = self.tx.send(req);
    }

    pub fn try_recv(&self) -> Option<FsResponse> {
        self.rx.try_recv().ok()
    }
}
