use byteorder::{BigEndian, ReadBytesExt};
use serde::Deserialize;
use std::{
    collections::HashMap,
    io::{BufRead, Read},
};
use tracing::debug;

#[derive(Clone, Debug)]
pub struct SlimeWorld {
    pub version: u8,
    pub world_version: i32,
    pub world_flags: WorldFlags,
    pub chunks: Vec<Chunk>,
}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub x: i32,
    pub z: i32,
    pub sections: Vec<Section>,
    // FIXME: use strong type
    pub heightmaps: fastnbt::Value,
    pub poi_chunks: Option<fastnbt::Value>,
    pub block_ticks: Option<fastnbt::Value>,
    pub fluid_ticks: Option<fastnbt::Value>,
    pub tile_entities: Vec<fastnbt::Value>,
    pub entities: fastnbt::Value,
    // but leave this as Value
    pub extra: Option<fastnbt::Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TileEntities {
    tile_entities: Vec<fastnbt::Value>,
}

#[derive(Clone, Debug)]
pub struct Section {
    pub sky_light: Option<Vec<u8>>,
    pub block_light: Option<Vec<u8>>,
    pub block_states: PalettedContainer<4096, BlockState>,
    // FIXME: use strong type
    pub biomes: fastnbt::Value,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct WorldFlags(u8);

impl From<u8> for WorldFlags {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl WorldFlags {
    const fn poi_chunks(&self) -> bool {
        self.0 & 1 == 1
    }

    const fn fluid_ticks(&self) -> bool {
        self.0 & 2 == 2
    }

    const fn block_ticks(&self) -> bool {
        self.0 & 4 == 4
    }

    const fn other_flag_count(&self) -> u32 {
        self.0.count_ones()
            - (self.poi_chunks() as u32)
            - (self.fluid_ticks() as u32)
            - (self.block_ticks() as u32)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("Magic bytes are not present (expected 0xB10B, got {0:#X})")]
    MagicBytes(u16),
    #[error("World uses slime version {0}, which is unsupported; only version 13 currently works.")]
    UnsupportedVersion(u8),
    #[error("Got invalid decompression result: expected size {0}, got {1}")]
    DecompressSize(usize, usize),
    // TODO: better error type for this case
    #[error("Failed to convert decompressed data")]
    DecompressConvert,
    #[error("FastNBT deserialization error")]
    Nbt(#[from] fastnbt::error::Error),
}

type Result<T> = std::result::Result<T, Error>;

impl SlimeWorld {
    pub fn deserialize(buf: &mut impl BufRead) -> Result<Self> {
        let magic = buf.read_u16::<BigEndian>()?;
        if magic != 0xB10B {
            return Err(Error::MagicBytes(magic));
        }

        let version = buf.read_u8()?;
        if version != 0x0D {
            return Err(Error::UnsupportedVersion(version));
        }

        let world_version = buf.read_i32::<BigEndian>()?;
        let world_flags = buf.read_u8()?.into();

        let chunk_buf = read_compressed(buf)?;
        let chunks = read_chunks(&mut &*chunk_buf, world_flags)?;

        Ok(Self {
            version,
            world_version,
            world_flags,
            chunks,
        })
    }
}

fn read_compressed(buf: &mut impl BufRead) -> Result<Vec<u8>> {
    let compressed_chunks_size: usize = buf.read_i32::<BigEndian>()?.try_into().unwrap();
    let uncompressed_chunks_size: usize = buf.read_i32::<BigEndian>()?.try_into().unwrap();

    let mut compressed = vec![0; compressed_chunks_size];
    buf.read_exact(&mut compressed)?;
    let decoded = zstd::decode_all(&*compressed)?;

    if decoded.len() != uncompressed_chunks_size {
        return Err(Error::DecompressSize(
            uncompressed_chunks_size,
            decoded.len(),
        ));
    }

    Ok(decoded)
}

fn read_chunks(buf: &mut impl Read, world_flags: WorldFlags) -> Result<Vec<Chunk>> {
    let chunks_to_read = buf.read_i32::<BigEndian>()?;
    let mut chunks = Vec::with_capacity(chunks_to_read.try_into().unwrap());

    for chunk_index in 0..chunks_to_read {
        let x = buf.read_i32::<BigEndian>()?;
        let z = buf.read_i32::<BigEndian>()?;

        let section_count = buf.read_i32::<BigEndian>()?;
        debug!("chunk {x}, {z}: {section_count} sections");

        let mut sections = Vec::with_capacity(section_count.try_into().unwrap());
        for section in 0..section_count {
            let flags = buf.read_u8()?;
            debug!("flags for section {section}: {flags}");

            let sky_light = match flags & 1 {
                1 => {
                    debug!("reading skylight");
                    let mut sky_light = vec![0; 2048];
                    buf.read_exact(&mut sky_light)?;
                    Some(sky_light)
                }
                _ => None,
            };

            let block_light = match flags & 2 {
                2 => {
                    debug!("reading blocklight");
                    let mut block_light = vec![0; 2048];
                    buf.read_exact(&mut block_light)?;
                    Some(block_light)
                }
                _ => None,
            };

            debug!("reading block states");
            let block_states: PalettedContainer<4096, BlockState> = read_sized(buf)?;
            debug!("{:?}", block_states);
            debug!("reading biomes");
            let biomes: fastnbt::Value = read_sized(buf)?;
            debug!("{:?}", biomes);

            sections.push(Section {
                sky_light,
                block_light,
                block_states,
                biomes,
            })
        }

        debug!("reading heightmaps");
        let heightmaps: fastnbt::Value = read_sized(buf)?;

        let poi_chunks = match world_flags.poi_chunks() {
            true => {
                debug!("Loading poi chunks");
                Some(read_sized(buf)?)
            }
            false => None,
        };

        let block_ticks = match world_flags.block_ticks() {
            true => {
                debug!("Loading block ticks");
                Some(read_sized(buf)?)
            }
            false => None,
        };

        let fluid_ticks = match world_flags.fluid_ticks() {
            true => {
                debug!("Loading fluid ticks");
                Some(read_sized(buf)?)
            }
            false => None,
        };

        debug!("Loading {} extra flags", world_flags.other_flag_count());
        for i in 0..world_flags.other_flag_count() {
            let extra_len = buf.read_i32::<BigEndian>()?;
            debug!("Discarding {extra_len} bytes for extra flag {i}");
            let mut extra_buf = vec![0; extra_len.try_into().unwrap()];
            buf.read_exact(&mut extra_buf)?;
        }

        let tile_entities = read_sized::<TileEntities>(buf)?.tile_entities;
        let entities: fastnbt::Value = read_sized(buf)?;

        let extra_size = buf.read_i32::<BigEndian>()?;
        let extra: Option<fastnbt::Value> = match extra_size {
            0 => None,
            extra_size => {
                debug!("Loading remaining compound tag size {extra_size} bytes");
                let mut bytebuf = vec![0u8; extra_size.try_into().unwrap()];
                buf.read_exact(&mut bytebuf)?;
                let extra = fastnbt::from_bytes(&*bytebuf)?;
                debug!("extra: {extra:?}");
                Some(extra)
            }
        };

        chunks.push(Chunk {
            x,
            z,
            sections,
            heightmaps,
            poi_chunks,
            block_ticks,
            fluid_ticks,
            tile_entities,
            entities,
            extra,
        })
    }

    Ok(chunks)
}

fn read_sized<T: serde::de::DeserializeOwned>(buf: &mut impl Read) -> Result<T> {
    let size = buf.read_i32::<BigEndian>()?;
    debug!("loading nbt, size {size} bytes");
    let mut bytebuf = vec![0u8; size.try_into().unwrap()];
    buf.read_exact(&mut bytebuf)?;
    Ok(fastnbt::from_bytes(&*bytebuf)?)
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct BlockState {
    pub name: String,
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PalettedContainer<const SIZE: usize, T> {
    pub palette: Vec<T>,
    pub data: Option<fastnbt::LongArray>,
}

// TODO: move this stuff to a different package as it's shared between Anvil and Slime
impl<const SIZE: usize, T> PalettedContainer<SIZE, T> {
    pub fn get(&self, index: usize) -> Option<&T> {
        if self.palette.len() == 1 {
            return self.palette.get(0);
        }

        if index >= SIZE {
            panic!("index {index} outside range for PalettedContainer of size {SIZE}");
        }

        // All indices are the same length. This length is set to the minimum amount
        // of bits required to represent the largest index in the palette, and then
        // set to a minimum size of 4 bits.
        let index_len: usize = self.palette.len().ilog2().try_into().unwrap();

        debug!(
            "Using index length {index_len} for palette with {} items",
            self.palette.len()
        );

        // this can almost certainly be optimized but i just want something working for now
        let indices_per_long = 64 / index_len;
        let index_long = index / indices_per_long;
        let index_bit_pos = 64 - (index % indices_per_long);
        let mask = ((1 << index_len) - 1) << (index_bit_pos - index_len);
        let true_index = ((self.data.as_ref().unwrap()[index_long] & mask)
            >> (index_bit_pos - index_len)) as usize;
        self.palette.get(true_index)
    }
}
