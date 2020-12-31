use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        // TODO: a real filesystem like https://emscripten.org/docs/api_reference/Filesystem-API.html
        pub mod fs {
            use std::convert::AsRef;
            use std::io::Result;
            use std::path::Path;
            use std::io::{Read, Write};

            pub struct File { }

            impl File {
                pub fn create<P: AsRef<Path>>(path: P) -> Result<File> {
                    println!("File create {}", path.as_ref().to_str().unwrap());
                    Ok(File{})
                }

                pub fn open<P: AsRef<Path>>(path: P) -> Result<File> {
                    println!("File open {}", path.as_ref().to_str().unwrap());
                    Err(std::io::Error::new(std::io::ErrorKind::Other, "No files exist on web"))
                }
            }

            impl Read for File {
                fn read(&mut self, _buf: &mut [u8]) -> Result<usize> {
                    println!("File read");
                    Ok(0)
                }
            }

            impl Read for &File {
                 fn read(&mut self, _buf: &mut [u8]) -> Result<usize> {
                     println!("&File read");
                     Ok(0)
                 }
            }

            impl Write for File {
                fn write(&mut self, buf: &[u8]) -> Result<usize> {
                    println!("File write {:?}", buf);
                    Ok(buf.len())
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
