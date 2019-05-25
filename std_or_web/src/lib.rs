use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
pub mod fs {
    use log::info;
    use std::io::{Result, Read, Write};
    use std::path::Path;
    use std::convert::AsRef;
    use stdweb::web::window;
    use hex;

    pub struct File {
        path: String,
        offset: usize,
    }

    impl File {
        pub fn open<P: AsRef<Path>>(path: P) -> Result<File> {
            let path: &str = path.as_ref().to_str().unwrap();
            info!("fs open {:?}", path);

            if !window().local_storage().contains_key(path) {
                Err(std::io::Error::from_raw_os_error(1))
            } else {
                Ok(File { path: path.to_string(), offset: 0 })
            }
        }
        pub fn create<P: AsRef<Path>>(path: P) -> Result<File> {
            let path: &str = path.as_ref().to_str().unwrap();
            info!("fs create {:?}", path);

            match window().local_storage().insert(path, "") {
                Ok(_) => Ok(File { path: path.to_string(), offset: 0 }),
                Err(_) => Err(std::io::Error::from_raw_os_error(1)),
            }
        }
    }

    impl Read for File {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            if let Some(string) = window().local_storage().get(&self.path) {
                match hex::decode(&string) {
                    Ok(data) => {
                        info!("self.offset = {}", self.offset);
                        info!("buf.len() = {}", buf.len());

                        let mut end = self.offset + buf.len();
                        if end > data.len() {
                            end = data.len();
                        }
                        info!("data.len() = {}", data.len());
                        info!("end = {}", end);

                        info!("data = {:?}", data);

                        let bytes = &data[self.offset..end];

                        info!("bytes = {:?}", bytes);
                        buf[..bytes.len()].copy_from_slice(&bytes);
                        self.offset = end;
                        Ok(bytes.len())
                    },
                    Err(_) => {
                        Err(std::io::Error::from_raw_os_error(8))
                    }
                }
            } else {
                Err(std::io::Error::from_raw_os_error(7))
            }
        }
    }
    impl Read for &File {
        fn read(&mut self, _buf: &mut [u8]) -> Result<usize> {
            //Ok(0)
            unimplemented!()
        }
    }

    impl Write for File {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            let string = window().local_storage().get(&self.path).unwrap();
            let new_string = string + &hex::encode(buf);
            self.offset += buf.len();
            match window().local_storage().insert(&self.path, &new_string) {
                Ok(_) => Ok(buf.len()),
                Err(_) => Err(std::io::Error::from_raw_os_error(1))
            }
        }
        fn flush(&mut self) -> Result<()> {
            Ok(())
        }
    }
}
    } else {
        pub use std::fs;
    }
}
