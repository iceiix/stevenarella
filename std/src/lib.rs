pub mod fs {
    use log::info;
    pub struct File {}
    impl File {
        pub fn open(path: &str) -> std::io::Result<File> {
            info!("fs open {}", path);
            Ok(File{})
        }
        pub fn create(path: &str) -> std::io::Result<File> {
            info!("fs create {}", path);
            Ok(File{})
        }
    }
    impl std::io::Read for File {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Ok(0)
        }
    }
    impl std::io::Write for File {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Ok(0)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}
