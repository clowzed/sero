use axum::http::StatusCode;
use std::{fmt::Debug, path::PathBuf};

#[derive(Debug)]
pub struct ActionParameters {
    pub subdomain_id: i64,
}

#[derive(Debug)]
pub struct AssociateParameters<T>
where
    T: AsRef<str>,
{
    pub user_id: i64,
    pub subdomain_name: T,
}

#[derive(Debug)]
pub struct FileSearchParameters<T>
where
    T: AsRef<str>,
{
    pub path: T,
    pub subdomain_id: i64,
}

#[derive(Debug)]
pub enum SiteFile {
    //* File was successfully retrieved
    Found(PathBuf),
    //* Subdomain is disabled
    //* Some(path) means there is 503.html
    //* None means there is no 503.html
    Disabled(Option<PathBuf>),

    //* File was not found
    //* Some(path) means there is 404.html
    //* None means there is no 404.html
    NotFound(Option<PathBuf>),
}

impl SiteFile {
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            SiteFile::Found(path) => Some(path),
            SiteFile::NotFound(inner) => inner.as_ref(),
            SiteFile::Disabled(inner) => inner.as_ref(),
        }
    }
}

impl From<&SiteFile> for StatusCode {
    fn from(value: &SiteFile) -> Self {
        match value {
            SiteFile::Found(_) => StatusCode::OK,
            SiteFile::Disabled(_) => StatusCode::SERVICE_UNAVAILABLE,
            SiteFile::NotFound(_) => StatusCode::NOT_FOUND,
        }
    }
}
