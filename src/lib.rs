#![cfg_attr(not(test), no_std)]

/// A CPIO file (newc format) reader.
///
/// # Example
///
/// ```rust,should_panic
/// use cpio::CpioNewcReader;
///
/// let reader = CpioNewcReader::new(&[]);
/// for obj in reader {
///     println!("{}", obj.unwrap().name);    
/// }
/// ```
pub struct CpioNewcReader<'a> {
    buf: &'a [u8],
}

impl<'a> CpioNewcReader<'a> {
    /// Creates a new CPIO reader on the buffer.
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }
}

/// File system object in CPIO file.
pub struct Object<'a> {
    /// The file metadata.
    pub metadata: Metadata,
    /// The full pathname.
    pub name: &'a str,
    /// The file data.
    pub data: &'a [u8],
}

impl<'a> Iterator for CpioNewcReader<'a> {
    type Item = Result<Object<'a>, ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: To workaround lifetime
        let s: &'a mut Self = unsafe { core::mem::transmute(self) };
        match inner(&mut s.buf) {
            Ok(Object {
                name: "TRAILER!!!", ..
            }) => None,
            res => Some(res),
        }
    }
}

fn inner<'a>(buf: &'a mut &'a [u8]) -> Result<Object<'a>, ReadError> {
    const HEADER_LEN: usize = 110;
    const MAGIC_NUMBER: &[u8] = b"070701";

    if buf.len() < HEADER_LEN {
        return Err(ReadError::BufTooShort);
    }
    let magic = buf.read_bytes(6)?;
    if magic != MAGIC_NUMBER {
        return Err(ReadError::InvalidMagic);
    }
    let ino = buf.read_hex_u32()?;
    let mode = buf.read_hex_u32()?;
    let uid = buf.read_hex_u32()?;
    let gid = buf.read_hex_u32()?;
    let nlink = buf.read_hex_u32()?;
    let mtime = buf.read_hex_u32()?;
    let file_size = buf.read_hex_u32()?;
    let dev_major = buf.read_hex_u32()?;
    let dev_minor = buf.read_hex_u32()?;
    let rdev_major = buf.read_hex_u32()?;
    let rdev_minor = buf.read_hex_u32()?;
    let name_size = buf.read_hex_u32()? as usize;
    let _check = buf.read_hex_u32()?;
    let metadata = Metadata {
        ino,
        mode,
        uid,
        gid,
        nlink,
        mtime,
        file_size,
        dev_major,
        dev_minor,
        rdev_major,
        rdev_minor,
    };
    let name_with_nul = buf.read_bytes(name_size)?;
    if name_with_nul.last() != Some(&0) {
        return Err(ReadError::InvalidName);
    }
    let name = core::str::from_utf8(&name_with_nul[..name_size - 1])
        .map_err(|_| ReadError::InvalidName)?;
    buf.read_bytes(pad_to_4(HEADER_LEN + name_size))?;

    let data = buf.read_bytes(file_size as usize)?;
    buf.read_bytes(pad_to_4(file_size as usize))?;

    Ok(Object {
        metadata,
        name,
        data,
    })
}

trait BufExt<'a> {
    fn read_hex_u32(&mut self) -> Result<u32, ReadError>;
    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], ReadError>;
}

impl<'a> BufExt<'a> for &'a [u8] {
    fn read_hex_u32(&mut self) -> Result<u32, ReadError> {
        let (hex, rest) = self.split_at(8);
        *self = rest;
        let str = core::str::from_utf8(hex).map_err(|_| ReadError::InvalidASCII)?;
        let value = u32::from_str_radix(str, 16).map_err(|_| ReadError::InvalidASCII)?;
        Ok(value)
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], ReadError> {
        if self.len() < len {
            return Err(ReadError::BufTooShort);
        }
        let (bytes, rest) = self.split_at(len);
        *self = rest;
        Ok(bytes)
    }
}

/// pad out to a multiple of 4 bytes
fn pad_to_4(len: usize) -> usize {
    match len % 4 {
        0 => 0,
        x => 4 - x,
    }
}

/// The error type which is returned from CPIO reader.
#[derive(Debug, PartialEq, Eq)]
pub enum ReadError {
    InvalidASCII,
    InvalidMagic,
    InvalidName,
    BufTooShort,
}

/// The file metadata.
#[derive(Debug)]
pub struct Metadata {
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub nlink: u32,
    pub mtime: u32,
    pub file_size: u32,
    pub dev_major: u32,
    pub dev_minor: u32,
    pub rdev_major: u32,
    pub rdev_minor: u32,
}
