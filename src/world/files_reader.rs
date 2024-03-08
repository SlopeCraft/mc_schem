use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;

use sevenz_rust::SevenZReader;

use crate::error::Error;
use crate::world::{FileInfo, FilesInMemory, FilesRead, FolderOnDisk, SubDirectory};

impl FolderOnDisk {
    pub fn new(path: &str) -> Self {
        let mut ret = FolderOnDisk { path: path.replace('\\', "/") };
        if ret.path.ends_with('/') {
            ret.path.pop();
        }
        return ret;
    }
}

impl FilesRead for FolderOnDisk {
    fn files(&self) -> Vec<FileInfo> {
        let mut result = Vec::new();
        for entry in walkdir::WalkDir::new(&self.path) {
            if let Ok(entry) = entry {
                let filename: &str;
                if let Some(f) = entry.file_name().to_str() {
                    filename = f;
                } else {
                    continue;
                }

                if let Ok(metadata) = entry.metadata() {
                    if !metadata.is_file() {
                        continue;
                    }
                    let tmp = FileInfo {
                        name: filename.to_string(),
                        full_name: filename.to_string(),
                        size: metadata.len(),
                    };
                    result.push(tmp);
                }
            }
        }

        return result;
    }

    fn open_file(&self, filename: &str) -> Result<Box<dyn Read>, Error> {
        let filename = format!("{}/{filename}", self.path);
        return match File::open(filename) {
            Ok(file) => Ok(Box::new(file)),
            Err(e) => Err(Error::FileOpenError(e))
        };
    }

    fn read_file(&self, filename: &str, dest: &mut Vec<u8>) -> Result<(), Error> {
        let filename = format!("{}/{filename}", self.path);
        let mut src = match File::open(&filename) {
            Ok(file) => file,
            Err(e) => return Err(Error::FileOpenError(e))
        };

        let metadata = match std::fs::metadata(filename) {
            Ok(md) => md,
            Err(e) => return Err(Error::FileOpenError(e))
        };
        dest.reserve(metadata.len() as usize);
        if let Err(e) = src.read_to_end(dest) {
            return Err(Error::IOReadError(e));
        }
        return Ok(());
    }
}

impl FilesInMemory {
    pub fn from_7z_reader<T: Read + Seek>(mut src: SevenZReader<T>, source: Option<String>) -> Result<FilesInMemory, Error> {
        let mut result = FilesInMemory {
            files: HashMap::new(),
            source: source.unwrap_or("7z file loaded from SevenZReader, filename unknown".to_string()),
        };
        let for_each_res = src.for_each_entries(|entry, reader| {
            let mut vec = Vec::with_capacity(entry.size as usize);
            match reader.read_to_end(&mut vec) {
                Ok(_) => {}
                Err(e) => return Err(sevenz_rust::Error::Io(e, Cow::from("")))
            }

            result.files.insert(entry.name.clone(), vec);
            return Ok(true);
        });
        if let Err(e7z) = for_each_res {
            return Err(Error::SevenZipDecompressError(e7z));
        }
        return Ok(result);
    }

    pub fn from_7z_file(path: impl AsRef<Path> + std::fmt::Display, password: &str) -> Result<FilesInMemory, Error> {
        let filename = path.to_string();
        let szr = match SevenZReader::open(path, sevenz_rust::Password::from(password)) {
            Ok(r) => r,
            Err(e) => return Err(Error::SevenZipDecompressError(e)),
        };
        return Self::from_7z_reader(szr, Some(filename));
    }
}

impl FilesRead for FilesInMemory {
    fn files(&self) -> Vec<FileInfo> {
        let mut vec = Vec::with_capacity(self.files.len());
        for (name, bytes) in &self.files {
            vec.push(FileInfo {
                name: name.clone(),
                full_name: name.clone(),
                size: bytes.len() as u64,
            });
        }
        return vec;
    }

    fn open_file(&self, filename: &str) -> Result<Box<dyn Read + '_>, Error> {
        return match self.files.get(filename) {
            Some(bytes) => {
                Ok(Box::new(bytes.as_slice()))
            }
            None => {
                Err(Error::NoSuchFile {
                    filename: filename.to_string(),
                    expected_to_exist_in: self.source.clone(),
                })
            }
        };
    }

    fn read_file(&self, filename: &str, dest: &mut Vec<u8>) -> Result<(), Error> {
        return match self.files.get(filename) {
            Some(bytes) => {
                dest.resize(bytes.len(), 0);
                dest.clone_from_slice(&bytes);
                Ok(())
            }
            None => {
                Err(Error::NoSuchFile {
                    filename: filename.to_string(),
                    expected_to_exist_in: self.source.clone(),
                })
            }
        };
    }

    fn read_file_nocopy(&self, filename: &str) -> Result<Option<&[u8]>, Error> {
        return match self.files.get(filename) {
            Some(bytes) => {
                Ok(Some(&bytes))
            }
            None => {
                Err(Error::NoSuchFile {
                    filename: filename.to_string(),
                    expected_to_exist_in: self.source.clone(),
                })
            }
        };
    }
}

impl FilesRead for SubDirectory<'_> {
    fn files(&self) -> Vec<FileInfo> {
        let src = self.root.files();
        let mut result = Vec::with_capacity(src.len());

        for mut info in src {
            if info.name.starts_with(&self.dirname_with_slash) {
                info.name = info.name[self.dirname_with_slash.len()..info.name.len()].to_string();
                result.push(info);
            }
        }

        return result;
    }

    fn open_file(&self, filename: &str) -> Result<Box<dyn Read + '_>, Error> {
        let mut new_filename = self.dirname_with_slash.clone();
        new_filename.push_str(filename);
        return self.root.open_file(&new_filename);
    }

    fn read_file(&self, filename: &str, dest: &mut Vec<u8>) -> Result<(), Error> {
        let mut new_filename = self.dirname_with_slash.clone();
        new_filename.push_str(filename);
        return self.root.read_file(&new_filename, dest);
    }
}