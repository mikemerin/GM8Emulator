use crate::asset::{assert_ver, Asset, AssetDataError};
use crate::byteio::{ReadBytes, ReadString, WriteBytes, WriteString};
use std::io::{self, Seek, SeekFrom};

pub const VERSION: u32 = 800;

pub struct Sound {
    /// The asset name present in GML and the editor.
    pub name: String,

    /// The source file name, including the extension.
    pub source: String,

    /// The file type (extension).
    pub extension: String,

    /// The raw filedata.
    /// This is optional because the associated data can be blank
    /// since in the IDE when you create a new sound it has no associated data.
    pub data: Option<Box<[u8]>>,

    /// Stupid legacy garbage indicating what kind of sound it is.
    /// If it's SoundKind::Multimedia, it should be opened with your system's media player.
    pub kind: SoundKind,

    /// The output volume.
    /// Value is between 0.0 and 1.0, although the editor only allows as low as 0.3.
    /// This is meant to be applied at first load time and cannot be changed.
    pub volume: f64,

    /// Stereo Panning.
    /// Value is between -1.0 and +1.0 (-1 Left <- 0 -> Right +1)
    /// This is meant to be applied at first load time and cannot be changed.
    pub pan: f64,

    /// Indicates whether the "preload" option was checked for this sound.
    /// This would mean the samples would be decoded at load time,
    /// and you wouldn't be able to manipulate volume, pan, etc from GML.
    pub preload: bool,

    /// Various "effects" which can be checked in the sound editor.
    /// These are meant to be computed at load time and aren't dynamic.
    pub fx: SoundFX,
}

/// Various filters which can be set on any sound.
pub struct SoundFX {
    pub chorus: bool,
    pub echo: bool,
    pub flanger: bool,
    pub gargle: bool,
    pub reverb: bool,
}

#[derive(Copy, Clone, PartialEq)]
pub enum SoundKind {
    /// Normal Sound
    Normal = 0,
    /// Background music
    BackgroundMusic = 1,
    /// 3D Sound
    ThreeDimensional = 2,
    /// Use multimedia player
    Multimedia = 3,
}

impl From<u32> for SoundKind {
    fn from(n: u32) -> SoundKind {
        match n {
            0 => SoundKind::Normal,
            1 => SoundKind::BackgroundMusic,
            2 => SoundKind::ThreeDimensional,
            3 => SoundKind::Multimedia,

            _ => SoundKind::Normal,
        }
    }
}

impl Asset for Sound {
    fn deserialize<B>(bytes: B, strict: bool, _version: u32) -> Result<Self, AssetDataError>
    where
        B: AsRef<[u8]>,
        Self: Sized
    {
        let mut reader = io::Cursor::new(bytes.as_ref());
        let name = reader.read_pas_string()?;

        if strict {
            let version = reader.read_u32_le()?;
            assert_ver(version, VERSION)?;
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let kind = SoundKind::from(reader.read_u32_le()?);
        let extension = reader.read_pas_string()?;
        let source = reader.read_pas_string()?;

        let data = if reader.read_u32_le()? != 0 {
            let len = reader.read_u32_le()? as usize;
            let pos = reader.position() as usize;
            reader.seek(SeekFrom::Current(len as i64))?;
            Some(reader.get_ref()[pos..pos + len].to_vec().into_boxed_slice())
        } else {
            None
        };

        let effects = reader.read_u32_le()?;
        let chorus: bool = (effects & 1) != 0;
        let echo: bool = (effects & 2) != 0;
        let flanger: bool = (effects & 4) != 0;
        let gargle: bool = (effects & 8) != 0;
        let reverb: bool = (effects & 16) != 0;

        let volume = reader.read_f64_le()?;
        let pan = reader.read_f64_le()?;
        let preload = reader.read_u32_le()? != 0;

        Ok(Sound {
            name,
            source,
            extension,
            kind,
            data,
            volume,
            pan,
            preload,
            fx: SoundFX {
                chorus,
                echo,
                flanger,
                gargle,
                reverb
            },
        })
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write
    {
        let mut result = writer.write_pas_string(&self.name)?;
        result += writer.write_u32_le(VERSION)?;
        result += writer.write_u32_le(self.kind as u32)?;
        result += writer.write_pas_string(&self.extension)?;
        result += writer.write_pas_string(&self.source)?;

        if let Some(data) = &self.data {
            result += writer.write_u32_le(1)?;
            result += writer.write_u32_le(data.len() as u32)?;
            result += writer.write(&data)?;
        } else {
            result += writer.write_u32_le(0)?;
        }

        let effects = (if self.fx.chorus { 1 } else { 0 })
            + (if self.fx.echo { 2 } else { 0 })
            + (if self.fx.flanger { 4 } else { 0 })
            + (if self.fx.gargle { 8 } else { 0 })
            + (if self.fx.reverb { 16 } else { 0 });
        result += writer.write_u32_le(effects)?;
        result += writer.write_f64_le(self.volume)?;
        result += writer.write_f64_le(self.pan)?;
        result += writer.write_u32_le(if self.preload { 1 } else { 0 })?;

        Ok(result)
    }
}
