use super::{error::ServiceError, models::*, parameters::UploadParameters};
use async_zip::base::read::seek::ZipFileReader;
use entity::prelude::*;
use sea_orm::{prelude::*, Set, TransactionTrait};
use std::{
    io::Cursor,
    path::{Path, PathBuf},
};
use tokio::{fs, fs::File, io::AsyncWriteExt as _};

pub struct Service;

impl Service {
    async fn process<U, B>(contents: B, upload_folder: U) -> Result<Vec<ArchiveFile>, ServiceError>
    where
        U: AsRef<Path>,
        B: AsRef<[u8]>,
    {
        let mut zip = ZipFileReader::with_tokio(Cursor::new(&contents)).await?;

        //? Iterating over the entries in a zip file
        //? and filtering out only the files (not directories).
        let entries = zip
            .file()
            .entries()
            .iter()
            .cloned()
            .enumerate()
            .filter_map(|(index, entry)| match entry.dir() {
                Ok(false) => Some((index, entry)),
                _ => None,
            })
            .collect::<Vec<_>>();
        tracing::trace!(amount = entries.len(), "Found files in zip!");

        let mut paths = vec![];

        for (index, entry) in entries {
            tracing::trace!(%index, "Processing entry...");

            //? Get entry path
            let path = {
                let entry_filename = entry
                    .filename()
                    .as_str()
                    .inspect_err(|cause| tracing::warn!(%cause, "Failed to convert entry filepath to str"))?;
                PathBuf::from(entry_filename)
            };
            tracing::trace!(?path, "Entry filepath was successfully retrieved");

            //? Generating filename for enty
            // Just random to prevent collisions
            let u1 = Uuid::new_v4();
            let u2 = Uuid::new_v4();

            let upload_folder_display = upload_folder.as_ref().display();

            let filename_to_save = PathBuf::from(format!("{upload_folder_display}/{u1}{u2}"));
            tracing::trace!(?filename_to_save, "Filename for saving entry was generated!");

            //? Creating file
            tracing::trace!(?filename_to_save, "Creating file...");
            let mut out = File::create(&filename_to_save)
                .await
                .inspect_err(|cause| tracing::error!(%cause, ?filename_to_save,"Failed to create file for entry"))?;

            //? Writing entry contents to file
            let mut bytes = vec![];
            let mut reader = zip
                .reader_with_entry(index)
                .await
                .inspect_err(|cause| tracing::error!(%cause, %index, "Failed to read entry by index"))?;

            tracing::trace!(%index, "Reading entry to the end...");
            reader
                .read_to_end_checked(&mut bytes)
                .await
                .inspect_err(|cause| tracing::error!(%cause, %index, "Failed to read entry to the end"))?;

            tracing::trace!(%index, ?filename_to_save, "Writing entry to new file...");
            out.write_all(&bytes).await.inspect_err(
                |cause| tracing::error!(%index, ?filename_to_save, %cause, "Failed to write entry contents!"),
            )?;

            tracing::trace!(%index, ?filename_to_save, "Entry was successfully written!");

            paths.push(ArchiveFile {
                real_path: filename_to_save,
                user_path: path,
            });
        }

        Ok(paths)
    }

    #[tracing::instrument(skip(connection, parameters))]
    pub async fn upload<C, P, T>(parameters: P, connection: &C) -> Result<(), ServiceError>
    where
        C: ConnectionTrait + TransactionTrait,
        P: Into<UploadParameters<T>>,
        T: AsRef<Path>,
    {
        let provided_parameters = parameters.into();

        //? Get subdomain model
        let subdomain = match SubdomainEntity::find_by_id(provided_parameters.subdomain_id)
            .one(connection)
            .await?
        {
            Some(subdomain) => Ok(subdomain),
            None => Err(ServiceError::SubdomainWasNotFound(provided_parameters.subdomain_id)),
        }?;

        //? Mark files for removal
        //? The actual deletion will also be handled
        tracing::trace!("Marking old files as obsolete...");
        let rows_affected = FileEntity::update_many()
            .filter(FileColumn::SubdomainId.eq(subdomain.id))
            .col_expr(FileColumn::Obsolete, Expr::value(true))
            .exec(connection)
            .await?
            .rows_affected;
        tracing::trace!(%rows_affected, "Files were successfully marked as obsolete!");

        //? Removing all cors rows related to subdomain
        tracing::trace!("Removing all related origins...");
        let rows_affected = OriginEntity::delete_many()
            .filter(OriginColumn::SubdomainId.eq(Some(subdomain.id)))
            .exec(connection)
            .await?
            .rows_affected;
        tracing::trace!(%rows_affected, "Origins were successfully removed!");

        //? Writing new archive
        let new_archive_path = {
            let upload_folder_display = provided_parameters.upload_folder.as_ref().display();
            let subdomain_id = subdomain.id;
            format!("{upload_folder_display}/{subdomain_id}.zip")
        };
        tracing::trace!("Archive will be written to {new_archive_path}");

        let mut new_archive_file = File::create(&new_archive_path).await.inspect_err(
            |cause| tracing::error!(%cause, "Failed to create archive file with path {new_archive_path}"),
        )?;

        new_archive_file
            .write_all(&provided_parameters.contents)
            .await
            .inspect_err(
                |cause| tracing::error!(%cause, "Failed to write archive contents in file with path {new_archive_path}"),
            )?;

        let files_upload_folder = provided_parameters
            .upload_folder
            .as_ref()
            .to_path_buf()
            .join(subdomain.id.to_string());

        tracing::trace!("Files will be uploaded to {:?}", files_upload_folder);

        tracing::trace!("Checking if exists: {:?}", files_upload_folder);

        if !files_upload_folder.exists() {
            tracing::trace!("Does not exist: {:?}", files_upload_folder);
            fs::create_dir_all(&files_upload_folder).await.inspect_err(
                |cause| tracing::error!(%cause, "Failed to create directory : {:?}", files_upload_folder),
            )?;
        }

        //TODO This is very bad if there are lots of files
        //TODO Possible solutions: stream or upload file one by one (this will be braking change)
        //? Processing all files
        tracing::trace!("Processing files from archive...");
        let files_to_be_inserted = Self::process(&provided_parameters.contents, files_upload_folder).await?;

        tracing::trace!(
            amount = files_to_be_inserted.len(),
            "Files were successfully processed!"
        );
        //? Saving files
        let models = files_to_be_inserted.iter().map(|file| FileActiveModel {
            subdomain_id: Set(Some(subdomain.id)),
            user_path: Set(file.user_path.display().to_string()),
            real_path: Set(file.real_path.display().to_string()),
            ..Default::default()
        });

        tracing::trace!("Saving paths to database...");
        FileEntity::insert_many(models).exec(connection).await?;

        tracing::trace!("Updating archive path in database...");
        //? Updating subdomain with new archive
        let mut active: SubdomainActiveModel = subdomain.clone().into();
        active.archive_path = Set(Some(new_archive_path));

        active.update(connection).await?;

        Ok(())
    }
}
