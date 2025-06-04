use alloc::string::{String, ToString as _};

pub struct Directory {
    pub files: alloc::collections::BTreeMap<String, File>,
    pub directories: alloc::collections::BTreeMap<String, Directory>,
}

type ReadFn = fn(
    file: &File,
    offset: usize,
    writer: &mut dyn crate::io::Write,
) -> Result<usize, crate::io::WriteError>;

pub enum File {
    Regular {
        contents: alloc::vec::Vec<u8>,
    },
    Special {
        read: ReadFn,
        write: fn(&File, usize, &[u8]) -> usize,
    },
}

impl File {
    pub fn regular(contents: impl Into<alloc::vec::Vec<u8>>) -> Self {
        File::Regular {
            contents: contents.into(),
        }
    }

    pub fn special(read: ReadFn, write: fn(&File, usize, &[u8]) -> usize) -> Self {
        File::Special { read, write }
    }

    pub fn read(
        &self,
        offset: usize,
        mut writer: impl crate::io::Write,
    ) -> Result<usize, crate::io::WriteError> {
        match self {
            File::Regular { contents } => writer.write(&contents[offset..]),
            File::Special { read, .. } => read(self, offset, &mut writer),
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

    pub fn directory(&self, path: &str) -> Option<&Directory> {
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

    fn directory_impl(&self, path: &[&str]) -> Option<&Directory> {
        match path {
            [] => Some(self),
            [name] => self.directories.get(*name),
            [name, rest @ ..] => self
                .directories
                .get(*name)
                .and_then(|dir| dir.directory_impl(rest)),
        }
    }

    fn directory_mut_impl(&mut self, path: &[&str]) -> Option<&mut Directory> {
        match path {
            [] => Some(self),
            [name] => self.directories.get_mut(*name),
            [name, rest @ ..] => self
                .directories
                .get_mut(*name)
                .and_then(|dir| dir.directory_mut_impl(rest)),
        }
    }

    pub fn file(&self, path: &str) -> Option<&File> {
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

    fn file_impl(&self, path: &[&str]) -> Option<&File> {
        match path {
            [] => None,
            [name] => self.files.get(*name),
            [name, rest @ ..] => self
                .directories
                .get(*name)
                .and_then(|dir| dir.file_impl(rest)),
        }
    }

    fn file_mut_impl(&mut self, path: &[&str]) -> Option<&mut File> {
        match path {
            [] => None,
            [name] => self.files.get_mut(*name),
            [name, rest @ ..] => self
                .directories
                .get_mut(*name)
                .and_then(|dir| dir.file_mut_impl(rest)),
        }
    }

    pub fn empty() -> Self {
        Directory {
            files: alloc::collections::BTreeMap::new(),
            directories: alloc::collections::BTreeMap::new(),
        }
    }

    pub fn create_directories(&mut self, path: &[&str]) -> &mut Directory {
        match path {
            [] => self,
            &[name] => self
                .directories
                .entry(name.to_string())
                .or_insert_with(Directory::empty),
            [name, rest @ ..] => {
                let dir = self
                    .directories
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
            &[name] => self.files.entry(name.to_string()).or_insert(file),
            [name, rest @ ..] => {
                let dir = self
                    .directories
                    .entry((*name).to_string())
                    .or_insert_with(Directory::empty);
                dir.create_file_(rest, file)
            }
        }
    }

    fn print_impl(&self, indent: usize) {
        for (name, file) in &self.files {
            match file {
                File::Regular { contents } => {
                    log::debug!("{:indent$}+ '{name}' ({} B)", "", contents.len());
                }
                File::Special { .. } => {
                    log::debug!("{:indent$}+ '{name}' (special)", "");
                }
            }
        }
        for (name, dir) in &self.directories {
            log::debug!("{:indent$}+ {name}/", "");
            dir.print_impl(indent + 2);
        }
    }
    pub fn print(&self) {
        self.print_impl(0);
    }
}
