#![feature(unique)]
#![allow(dead_code)]
extern crate libc;
use std::mem;
use std::ptr;

#[repr(C)]
#[allow(non_snake_case)]
struct SHA_CTX {
    h0: libc::c_uint,
    h1: libc::c_uint,
    h2: libc::c_uint,
    h3: libc::c_uint,
    h4: libc::c_uint,
    N1: libc::c_uint,
    NH: libc::c_uint,
    data: [libc::c_uint; 16],
    num: libc::c_uint
}

enum RSA{}

#[allow(non_camel_case_types)]
enum EVP_CIPHER_CTX{}

#[allow(non_camel_case_types)]
enum EVP_CIPHER{}

enum ENGINE{}

const RSA_PKCS1_PADDING : libc::c_int = 1;

extern {
    fn SHA1_Init(c: *mut SHA_CTX) -> libc::c_int;
    fn SHA1_Update(c: *mut SHA_CTX, data: *const u8, len: libc::size_t) -> libc::c_int;
    fn SHA1_Final(md: *const u8, c: *mut SHA_CTX) -> libc::c_int;

    fn d2i_RSA_PUBKEY(a: *mut *mut RSA, data: *const *const u8, len: libc::c_int) -> *mut RSA;
    fn RSA_free(rsa: *mut RSA);
    fn RSA_size(rsa: *mut RSA) -> libc::c_int;
    fn RSA_public_encrypt(flen: libc::c_int, from: *const u8, to: *mut u8, rsa: *mut RSA, padding: libc::c_int) -> libc::c_int;

    fn RAND_bytes(buf: *mut u8, len: libc::c_int) -> libc::c_int;

    fn EVP_CIPHER_CTX_init(a: *mut EVP_CIPHER_CTX);
    fn EVP_EncryptInit_ex(ctx: *mut EVP_CIPHER_CTX, cipher: *const EVP_CIPHER, engine: *mut ENGINE, key: *const u8, iv: *const u8) -> libc::c_int;
    fn EVP_DecryptInit_ex(ctx: *mut EVP_CIPHER_CTX, cipher: *const EVP_CIPHER, engine: *mut ENGINE, key: *const u8, iv: *const u8) -> libc::c_int;
    fn EVP_EncryptUpdate(ctx: *mut EVP_CIPHER_CTX, out: *mut u8, outl: *mut libc::c_int, input: *const u8, inl: libc::c_int) -> libc::c_int;
    fn EVP_DecryptUpdate(ctx: *mut EVP_CIPHER_CTX, out: *mut u8, outl: *mut libc::c_int, input: *const u8, inl: libc::c_int) -> libc::c_int;
    fn EVP_aes_128_cfb8() -> *const EVP_CIPHER;
    fn EVP_CIPHER_block_size(cipher: *const EVP_CIPHER) -> libc::c_int;
    fn EVP_CIPHER_CTX_free(ctx: *mut EVP_CIPHER_CTX);
    fn EVP_CIPHER_CTX_cleanup(ctx: *mut EVP_CIPHER_CTX) -> libc::c_int;
    fn EVP_CIPHER_CTX_new() -> *mut EVP_CIPHER_CTX;
}

pub struct EVPCipher {
    internal: ptr::Unique<EVP_CIPHER_CTX>,

    block_size: usize,
}

impl EVPCipher {
    pub fn new(key: &[u8], iv: &[u8], decrypt: bool) -> EVPCipher {
        unsafe {
            let mut e = EVPCipher {
                internal: ptr::Unique::new(EVP_CIPHER_CTX_new()),
                block_size: EVP_CIPHER_block_size(EVP_aes_128_cfb8()) as usize,
            };
            EVP_CIPHER_CTX_init(e.internal.get_mut());
            if decrypt {
                if EVP_DecryptInit_ex(e.internal.get_mut(), EVP_aes_128_cfb8(), ptr::null_mut(), key.as_ptr(), iv.as_ptr()) == 0 {
                    panic!("Init error");
                }
            } else {
                if EVP_EncryptInit_ex(e.internal.get_mut(), EVP_aes_128_cfb8(), ptr::null_mut(), key.as_ptr(), iv.as_ptr()) == 0 {
                    panic!("Init error");
                }
            }
            e
        }
    }

    pub fn decrypt(&mut self, data: &[u8]) -> Vec<u8> {
        unsafe {
            let mut out = Vec::with_capacity(data.len());
            let mut out_len: libc::c_int = 0;
            if EVP_DecryptUpdate(self.internal.get_mut(), out.as_mut_ptr(), &mut out_len, data.as_ptr(), data.len() as libc::c_int) == 0 {
                panic!("Decrypt error")
            }
            out.set_len(out_len as usize);
            out
        }
    }

    pub fn encrypt(&mut self, data: &[u8]) -> Vec<u8> {
        unsafe {
            let mut out = Vec::with_capacity(data.len());
            let mut out_len: libc::c_int = 0;
            if EVP_EncryptUpdate(self.internal.get_mut(), out.as_mut_ptr(), &mut out_len, data.as_ptr(), data.len() as libc::c_int) == 0 {
                panic!("Encrypt error")
            }
            out.set_len(out_len as usize);
            out
        }
    }
}


impl Drop for EVPCipher {
    fn drop(&mut self) {
        unsafe {
            EVP_CIPHER_CTX_cleanup(self.internal.get_mut());
            EVP_CIPHER_CTX_free(self.internal.get_mut());
        }
    }
}

pub fn gen_random(len: usize) -> Vec<u8> {
    unsafe {
        let mut data = Vec::with_capacity(len);
        data.set_len(len);
        RAND_bytes(data.as_mut_ptr(), len as libc::c_int);
        data
    }
}


pub struct PublicKey {
    rsa: *mut RSA
}

impl PublicKey {
    pub fn new(data: &[u8]) -> PublicKey {
        unsafe {
            let cert = d2i_RSA_PUBKEY(ptr::null_mut(), &data.as_ptr(), data.len() as libc::c_int);
            if cert.is_null() {
                panic!("Error");
            }
            PublicKey { rsa: cert }
        }
    }

    pub fn encrypt(&mut self, data: &Vec<u8>) -> Vec<u8> {
        unsafe {
            let size = RSA_size(self.rsa) as usize;
            let mut out = Vec::with_capacity(size);
            out.set_len(size);
            RSA_public_encrypt(data.len() as libc::c_int, data.as_ptr(), out.as_mut_ptr(), self.rsa, RSA_PKCS1_PADDING);
            out
        }
    }
}

impl Drop for PublicKey {
    fn drop(&mut self) {
        unsafe {
            RSA_free(self.rsa);
        }
    }
}

pub struct SHA1 {
    internal: SHA_CTX,
}

impl SHA1 {
    pub fn new() -> SHA1 {
        unsafe {
            let mut s : SHA1 = mem::uninitialized();
            if SHA1_Init(&mut s.internal) == 0 {
                panic!("error init sha1");
            }
            s
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        unsafe {
            if SHA1_Update(&mut self.internal, data.as_ptr(), data.len() as libc::size_t) == 0 {
                panic!("sha1 error");
            }
        }
    }

    pub fn bytes(mut self) -> Vec<u8> {
        unsafe {
            let mut dst = Vec::with_capacity(20);
            dst.set_len(20);
            if SHA1_Final(dst.as_mut_ptr(), &mut self.internal) == 0 {
                panic!("sha1 error");
            }
            dst
        }
    }
}

#[cfg(test)]
mod test {
    use super::{SHA1};
    #[test]
    fn test_sha1() {
        let mut sha1 = SHA1::new();
        sha1.update(b"Hello world");
        assert!(sha1.bytes().iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().connect("") == "7B502C3A1F48C8609AE212CDFB639DEE39673F5E");
    }
}
