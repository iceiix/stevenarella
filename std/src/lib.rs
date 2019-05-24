use log::info;
use std::io::{Result, Read, Write};

pub mod fs {
    pub struct File {}
    impl File {
        pub fn open(path: &str) -> Result<File> {
            info!("fs open {}", path);
            Ok(File{})
        }
        pub fn create(path: &str) -> Result<File> {
            info!("fs create {}", path);
            Ok(File{})
        }
    }
    impl Read for File {
        fn read(&mut self, _buf: &mut [u8]) -> Result<usize> {
            Ok(0)
        }
    }
    impl Write for File {
        fn write(&mut self, _buf: &[u8]) -> Result<usize> {
            Ok(0)
        }
        fn flush(&mut self) -> Result<()> {
            Ok(())
        }
    }
}
