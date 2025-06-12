use alloc::string::{String, ToString as _};

use crate::process::StreamType;

pub mod root;

pub enum Directory {
    Regular {
        files: alloc::collections::BTreeMap<String, File>,
        directories: alloc::collections::BTreeMap<String, Directory>,
    },
    Special {
        files: alloc::collections::BTreeMap<String, File>,
        directories: alloc::collections::BTreeMap<String, Directory>,
        update: fn(
            files: &mut alloc::collections::BTreeMap<String, File>,
            directories: &mut alloc::collections::BTreeMap<String, Directory>,
        ),
    },
}

unsafe impl Send for Directory {}

pub enum File {
    Regular {
        contents: alloc::vec::Vec<u8>,
    },
    Special {
        read: alloc::boxed::Box<
            dyn Fn(
                &File,
                usize,
                &mut dyn crate::io::Write,
            ) -> Result<usize, crate::io::WriterError>,
        >,
        write: (),
    },
    Stream {
        stream_type: StreamType,
    },
}

impl File {
    pub fn regular(contents: impl Into<alloc::vec::Vec<u8>>) -> Self {
        File::Regular {
            contents: contents.into(),
        }
    }

    pub fn special(
        read: impl Fn(&File, usize, &mut dyn crate::io::Write) -> Result<usize, crate::io::WriterError>
            + 'static,
    ) -> Self {
        File::Special {
            read: alloc::boxed::Box::new(read),
            write: (),
        }
    }

    pub fn stream(stream_type: StreamType) -> Self {
        File::Stream { stream_type }
    }

    pub fn read(
        &self,
        offset: usize,
        mut writer: impl crate::io::Write,
    ) -> Result<usize, crate::io::WriterError> {
        match self {
            File::Regular { contents } => writer.write(&contents[offset..]),
            File::Special { read, .. } => {
                read(self, 0, &mut crate::io::Ignorer::ignoring(offset, writer))
            }
            File::Stream { .. } => panic!("cannot read from a stream file"),
        }
    }
}

impl Directory {
    pub fn new(directories: &[&str]) -> Self {
        let mut root = Directory::empty();
        for d in directories {
            root.create_directories(&[d]);
        }
        root
    }

    pub fn update(&mut self) {
        if let Directory::Special {
            update,
            files,
            directories,
        } = self
        {
            update(files, directories);
        }
    }

    pub fn files(&mut self) -> &mut alloc::collections::BTreeMap<String, File> {
        match self {
            Directory::Regular { files, .. } => files,
            Directory::Special {
                files,
                directories,
                update,
            } => {
                update(files, directories);
                files
            }
        }
    }

    pub fn directories(&mut self) -> &mut alloc::collections::BTreeMap<String, Directory> {
        self.update();

        match self {
            Directory::Regular { directories, .. } => directories,
            Directory::Special {
                files,
                directories,
                update,
            } => {
                update(files, directories);
                directories
            }
        }
    }

    pub fn directory(&mut self, path: &str) -> Option<&Directory> {
        let path = path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<alloc::vec::Vec<_>>();
        self.directory_impl(path.as_slice())
    }

    pub fn directory_mut(&mut self, path: &str) -> Option<&mut Directory> {
        let path = path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<alloc::vec::Vec<_>>();
        self.directory_mut_impl(path.as_slice())
    }

    fn directory_impl(&mut self, path: &[&str]) -> Option<&Directory> {
        match path {
            [] => Some(self),
            [name] => self.directories().get(*name),
            [name, rest @ ..] => self
                .directories()
                .get_mut(*name)
                .and_then(|dir| dir.directory_impl(rest)),
        }
    }

    fn directory_mut_impl(&mut self, path: &[&str]) -> Option<&mut Directory> {
        match path {
            [] => Some(self),
            [name] => self.directories().get_mut(*name),
            [name, rest @ ..] => self
                .directories()
                .get_mut(*name)
                .and_then(|dir| dir.directory_mut_impl(rest)),
        }
    }

    pub fn file(&mut self, path: &str) -> Option<&File> {
        let path = path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<alloc::vec::Vec<_>>();
        self.file_impl(path.as_slice())
    }

    pub fn file_mut(&mut self, path: &str) -> Option<&mut File> {
        let path = path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<alloc::vec::Vec<_>>();
        self.file_mut_impl(path.as_slice())
    }

    fn file_impl(&mut self, path: &[&str]) -> Option<&File> {
        match path {
            [] => None,
            [name] => self.files().get(*name),
            [name, rest @ ..] => self
                .directories()
                .get_mut(*name)
                .and_then(|dir| dir.file_impl(rest)),
        }
    }

    fn file_mut_impl(&mut self, path: &[&str]) -> Option<&mut File> {
        match path {
            [] => None,
            [name] => self.files().get_mut(*name),
            [name, rest @ ..] => self
                .directories()
                .get_mut(*name)
                .and_then(|dir| dir.file_mut_impl(rest)),
        }
    }

    pub fn empty() -> Self {
        Directory::Regular {
            files: alloc::collections::BTreeMap::new(),
            directories: alloc::collections::BTreeMap::new(),
        }
    }

    pub fn create_directories(&mut self, path: &[&str]) -> &mut Directory {
        match path {
            [] => self,
            &[name] => self
                .directories()
                .entry(name.to_string())
                .or_insert_with(Directory::empty),
            [name, rest @ ..] => {
                let dir = self
                    .directories()
                    .entry((*name).to_string())
                    .or_insert_with(Directory::empty);
                dir.create_directories(rest)
            }
        }
    }

    pub fn create_file(&mut self, path: &str, file: File) -> &mut File {
        let path = path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<alloc::vec::Vec<_>>();
        self.create_file_(path.as_slice(), file)
    }

    fn create_file_(&mut self, path: &[&str], file: File) -> &mut File {
        match path {
            [] => panic!("Cannot create file at root"),
            &[name] => self.files().entry(name.to_string()).or_insert(file),
            [name, rest @ ..] => {
                let dir = self
                    .directories()
                    .entry((*name).to_string())
                    .or_insert_with(Directory::empty);
                dir.create_file_(rest, file)
            }
        }
    }

    fn print_impl(&mut self, indent: usize) {
        for (name, file) in self.files() {
            match file {
                File::Regular { contents } => {
                    log::debug!("{:indent$}+ '{name}' ({} B)", "", contents.len());
                }
                File::Special { .. } => {
                    log::debug!("{:indent$}+ '{name}' (special)", "");
                }
                File::Stream { stream_type } => {
                    log::debug!("{:indent$}+ '{name}' (stream {:?})", "", stream_type);
                }
            }
        }
        for (name, dir) in self.directories() {
            log::debug!("{:indent$}+ {name}/", "");
            dir.print_impl(indent + 2);
        }
    }

    pub fn print(&mut self) {
        self.print_impl(0);
    }

    pub fn create_special_directory(
        &mut self,
        name: &str,
        update: fn(
            files: &mut alloc::collections::BTreeMap<String, File>,
            directories: &mut alloc::collections::BTreeMap<String, Directory>,
        ),
    ) -> &mut Directory {
        let path = name
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<alloc::vec::Vec<_>>();
        self.create_special_directory_(path.as_slice(), update)
    }

    fn create_special_directory_(
        &mut self,
        path: &[&str],
        update: fn(
            files: &mut alloc::collections::BTreeMap<String, File>,
            directories: &mut alloc::collections::BTreeMap<String, Directory>,
        ),
    ) -> &mut Directory {
        match path {
            [] => panic!("Cannot create special directory at root"),
            &[name] => self
                .directories()
                .entry(name.to_string())
                .or_insert_with(|| Directory::Special {
                    files: alloc::collections::BTreeMap::new(),
                    directories: alloc::collections::BTreeMap::new(),
                    update,
                }),
            [name, rest @ ..] => {
                let dir = self
                    .directories()
                    .entry((*name).to_string())
                    .or_insert_with(|| Directory::Special {
                        files: alloc::collections::BTreeMap::new(),
                        directories: alloc::collections::BTreeMap::new(),
                        update,
                    });
                dir.create_special_directory_(rest, update)
            }
        }
    }
}
