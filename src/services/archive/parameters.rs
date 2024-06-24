use bytes::Bytes;
use std::path::Path;

pub struct UploadParameters<T>
where
    T: AsRef<Path>,
{
    pub subdomain_id: i64,
    pub contents: Bytes,
    pub upload_folder: T,
}
