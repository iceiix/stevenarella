pub mod fs {
    use log::info;
    use std::io::{Result, Read, Write};
    use std::path::Path;
    use std::convert::AsRef;

    pub struct File {}
    impl File {
        pub fn open<P: AsRef<Path>>(path: P) -> Result<File> {
            info!("fs open {:?}", path.as_ref().to_str());
            Ok(File{})
        }
        pub fn create<P: AsRef<Path>>(path: P) -> Result<File> {
            info!("fs create {:?}", path.as_ref().to_str());
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
