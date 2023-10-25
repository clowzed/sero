use async_zip::base::read::seek::ZipFileReader;
use bytes::Bytes;

#[derive(Clone, Debug)]
pub struct ArchiveFile {
    pub real_path: std::path::PathBuf,
    pub user_path: std::path::PathBuf,
}

impl helpers::Deletable for ArchiveFile {
    #[tracing::instrument]
    fn delete(&self) -> Result<(), std::io::Error> {
        std::fs::remove_file(&self.real_path)
    }
}

impl ArchiveFile {
    pub fn new(real_path: std::path::PathBuf, user_path: std::path::PathBuf) -> Self {
        Self {
            real_path,
            user_path,
        }
    }
}

mod helpers {
    pub trait Deletable {
        fn delete(&self) -> Result<(), std::io::Error>;
    }

    pub struct FileDropper<T: Deletable + Clone> {
        files: Vec<T>,
    }

    impl<T: Deletable + Clone> Default for FileDropper<T> {
        fn default() -> Self {
            Self {
                files: Default::default(),
            }
        }
    }

    impl<T: Deletable + Clone> Drop for FileDropper<T> {
        fn drop(&mut self) {
            for file in &self.files {
                file.delete().ok();
            }
        }
    }

    impl<T: Deletable + Clone + std::fmt::Debug> FileDropper<T> {
        #[tracing::instrument(skip(self))]
        pub fn add(&mut self, inner: T) {
            self.files.push(inner);
        }

        #[tracing::instrument(skip(self))]
        pub fn cleanup(&mut self) {
            self.files.clear();
        }

        pub fn files(&self) -> Vec<T> {
            self.files.clone()
        }
    }
}

pub struct ArchiveService;

use thiserror::Error;
use tokio::io::AsyncWriteExt;

#[derive(Error, Debug)]
pub enum ArchiveServiceError {
    #[error("Archive contains no files!")]
    EmptyArchive,

    #[error(transparent)]
    ZipError(#[from] async_zip::error::ZipError),

    #[error(transparent)]
    FsError(#[from] std::io::Error),

    #[error(transparent)]
    DbError(#[from] sea_orm::DbErr),
}

impl ArchiveService {
    #[tracing::instrument(skip(archive_bytes))]
    pub async fn process(archive_bytes: Bytes) -> Result<Vec<ArchiveFile>, ArchiveServiceError> {
        let mut zip = ZipFileReader::with_tokio(std::io::Cursor::new(&archive_bytes)).await?;

        let entries = zip
            .file()
            .entries()
            .iter()
            .cloned()
            .enumerate()
            .collect::<Vec<_>>();

        if entries.is_empty() {
            return Err(ArchiveServiceError::EmptyArchive);
        }

        let mut fmanager = helpers::FileDropper::default();

        for (index, entry) in entries {
            let entry = entry.entry();

            if entry.dir()? {
                continue;
            } // skip directories

            let filename_of_entry = entry.filename().as_str().unwrap();

            let path = std::path::PathBuf::from(filename_of_entry)
                .components()
                .skip(1)
                .collect::<std::path::PathBuf>();

            if path.file_name().unwrap_or_default() == "sero.toml" {
                continue;
            }

            let extension = path.extension().unwrap_or_default().to_str().unwrap();

            let extension = match extension.is_empty() {
                false => format!(".{}", extension),
                true => extension.to_string(),
            };

            let unique_filename = format!("{}-{}", uuid::Uuid::new_v4(), uuid::Uuid::new_v4());

            let filename_to_save: String = format!("{unique_filename}{extension}");

            let mut out = tokio::fs::File::create(&filename_to_save).await?;

            let mut bytes = vec![];
            let mut reader = zip.reader_with_entry(index).await.unwrap();

            reader.read_to_end_checked(&mut bytes).await?;

            out.write_all(&bytes).await?;
            fmanager.add(ArchiveFile::new(
                filename_to_save.into(),
                filename_of_entry.into(),
            ));
        }

        let result = fmanager.files();

        fmanager.cleanup(); // prevent deleting

        Ok(result)
    }
}
