use std::{
    collections::HashMap,
    io::{Cursor, Read},
};

use anyhow::Result;

// Kept struct in source code to show how the .PAK file is structured
// The order of the .PAK file has the same order as the struct below (hence the #[repr(C)] attribute)

#[allow(unused)]
#[repr(C)]
struct RawPack {
    /// The amount of files this PAK file contains
    count: u32,

    /// Size of name-mapping header
    cb_nhdr: u32,

    /// Size of codepoint header
    cb_cphdr: u32,

    /// Name mapping header raw data
    nhdr: Vec<u8>,

    /// Codepoint header raw data
    cphdr: Vec<u8>,

    /// File contents
    ///
    /// Every entry starts with it's size encoded in unsigned 32-bit big endian
    data: Vec<u8>,
}

pub struct EmojiPack {
    names: HashMap<String, String>,
    codepoints: HashMap<String, u64>,
    data: Vec<u8>,
}

impl EmojiPack {
    pub fn new(data: &[u8]) -> Result<Self> {
        let cb_nhdr = u32::from_be_bytes(data[4..8].try_into()?);
        let cb_cphdr = u32::from_be_bytes(data[8..12].try_into()?);

        let nhdr = &data[12..12 + cb_nhdr as usize];
        let cphdr = &data[12 + cb_nhdr as usize..12 + cb_nhdr as usize + cb_cphdr as usize];
        let data = &data[12 + cb_nhdr as usize + cb_cphdr as usize..];

        // Read name header
        let length = nhdr.len() as u64;

        let mut cursor = Cursor::new(nhdr);
        let mut names = HashMap::new();

        while cursor.position() < length {
            // Read name
            let mut length_bytes = [0; 2];
            cursor.read_exact(&mut length_bytes)?;

            let mut str_buf = vec![0; u16::from_be_bytes(length_bytes) as usize];
            cursor.read_exact(&mut str_buf)?;

            let name = String::from_utf8(str_buf)?;

            // Read codepoint
            cursor.read_exact(&mut length_bytes)?;

            let mut str_buf = vec![0; u16::from_be_bytes(length_bytes) as usize];
            cursor.read_exact(&mut str_buf)?;

            let codepoint = String::from_utf8(str_buf)?;

            names.insert(name, codepoint);
        }

        // Read codepoint header
        let length = cphdr.len() as u64;
        let mut cursor = Cursor::new(cphdr);

        let mut codepoints = HashMap::new();

        while cursor.position() < length {
            // Read name
            let mut length_bytes = [0; 2];
            cursor.read_exact(&mut length_bytes)?;

            let mut str_buf = vec![0; u16::from_be_bytes(length_bytes) as usize];
            cursor.read_exact(&mut str_buf)?;

            let codepoint = String::from_utf8(str_buf)?;

            // Read codepoint
            let mut len_buf = [0; 8];
            cursor.read_exact(&mut len_buf)?;

            let offset = u64::from_be_bytes(len_buf);

            codepoints.insert(codepoint, offset);
        }

        Ok(Self {
            names,
            codepoints,
            data: data.to_vec(),
        })
    }

    pub fn get_emoji(&self, name: &str) -> Option<&[u8]> {
        let codepoint = self.names.get(name)?;
        let offset = *self.codepoints.get(codepoint)?;

        let mut file_len_buf = [0; 4];
        file_len_buf.copy_from_slice(&self.data[offset as usize..offset as usize + 4]);

        let file_len = u32::from_be_bytes(file_len_buf);

        Some(&self.data[offset as usize + 4..offset as usize + 4 + file_len as usize])
    }
}

pub fn load_emojis() -> Result<EmojiPack> {
    EmojiPack::new(include_bytes!("../assets/twemoji.pak"))
}
