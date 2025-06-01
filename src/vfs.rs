use alloc::string::{String, ToString as _};

pub struct Directory {
    pub files: alloc::collections::BTreeMap<String, File>,
    pub directories: alloc::collections::BTreeMap<String, Directory>,
}

pub struct File {
    contents: alloc::vec::Vec<u8>,
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
        self._directory(path.as_slice())
    }

    pub fn directory_mut(&mut self, path: &str) -> Option<&mut Directory> {
        let path = path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<alloc::vec::Vec<_>>();
        self._directory_mut(path.as_slice())
    }

    fn _directory(&self, path: &[&str]) -> Option<&Directory> {
        match path {
            [] => Some(self),
            [name] => self.directories.get(*name),
            [name, rest @ ..] => self
                .directories
                .get(*name)
                .and_then(|dir| dir._directory(rest)),
        }
    }

    fn _directory_mut(&mut self, path: &[&str]) -> Option<&mut Directory> {
        match path {
            [] => Some(self),
            [name] => self.directories.get_mut(*name),
            [name, rest @ ..] => self
                .directories
                .get_mut(*name)
                .and_then(|dir| dir._directory_mut(rest)),
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
            [name] => self
                .directories
                .entry(name.to_string())
                .or_insert_with(Directory::empty),
            [name, rest @ ..] => {
                let dir = self
                    .directories
                    .entry(name.to_string())
                    .or_insert_with(Directory::empty);
                dir.create_directories(rest)
            }
        }
    }

    fn _print(&self, indent: usize) {
        for (name, file) in &self.files {
            log::debug!("{:indent$}File '{name}'", "");
            log::debug!(
                "{:indent$}\\Content Length: {}",
                "",
                file.contents.len(),
                indent = indent + 2
            );
        }
        for (name, dir) in &self.directories {
            log::debug!("{:indent$}|-Directory: {}", "", name, indent = indent);
            dir._print(indent + 2);
        }
    }
    pub fn print(&self) {
        self._print(0);
    }
}
