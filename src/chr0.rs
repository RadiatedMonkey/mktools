use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    brres::{IndexGroupEntry, Subfile, SubfileHeader, SubfileType},
    encoding::Decode,
    error::{EncodingError, EncodingResult},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnimationPolicy {
    OneTime,
    Loop,
}

impl TryFrom<u32> for AnimationPolicy {
    type Error = EncodingError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::OneTime,
            0x01 => Self::Loop,
            _ => {
                return Err(EncodingError::InvalidFile(format!(
                    "invalid animation policy: {value} (expected 0 or 1)"
                )));
            }
        })
    }
}

impl Decode for AnimationPolicy {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let policy = reader.read_u32::<BigEndian>()?;
        Self::try_from(policy)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ScalingRule {
    Standard,
    Softimage,
    Maya,
}

impl TryFrom<u32> for ScalingRule {
    type Error = EncodingError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::Standard,
            0x01 => Self::Softimage,
            0x02 => Self::Maya,
            _ => {
                return Err(EncodingError::InvalidFile(format!(
                    "invalid scaling rule: {} (expected 0, 1 or 2)",
                    value
                )));
            }
        })
    }
}

impl Decode for ScalingRule {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let rule = reader.read_u32::<BigEndian>()?;
        Self::try_from(rule)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chr0Header {
    pub frame_count: u16,
    pub anim_data_count: u16,
    pub anim_policy: AnimationPolicy,
    pub scaling_rule: ScalingRule,
}

impl Decode for Chr0Header {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let frame_count = reader.read_u16::<BigEndian>()?;
        let anim_data_count = reader.read_u16::<BigEndian>()?;
        let anim_policy = AnimationPolicy::decode(reader)?;
        let scaling_rule = ScalingRule::decode(reader)?;

        Ok(Self {
            frame_count,
            anim_data_count,
            anim_policy,
            scaling_rule,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum AnimationFormat {
    Interpolated4 = 0b001,
    Interpolated6 = 0b010,
    Interpolated12 = 0b011,
    Linear1 = 0b100,
    Linear4 = 0b110,
}

impl AnimationFormat {
    pub fn is_linear(&self) -> bool {
        const LINEAR_MASK: u8 = 0b100;
        (*self as u8) & LINEAR_MASK != 0
    }
}

impl TryFrom<u8> for AnimationFormat {
    type Error = EncodingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0b001 => Self::Interpolated4,
            0b010 => Self::Interpolated6,
            0b011 => Self::Interpolated12,
            0b100 => Self::Linear1,
            0b110 => Self::Linear4,
            _ => {
                return Err(EncodingError::InvalidFile(format!(
                    "invalid animation format: {value:#b}"
                )));
            }
        })
    }
}

const TRANSLATION_FORMAT_MASK: u32 = 0xc0000000; // Bits 32-31
const ROTATION_FORMAT_MASK: u32 = 0x38000000; // Bits 30-28
const SCALE_FORMAT_MASK: u32 = 0x06000000; // Bits 27-26
const HAS_TRANSLATION_MASK: u32 = 0x01000000; // Bit 25
const HAS_ROTATION_MASK: u32 = 0x00800000; // Bit 24
const HAS_SCALE_MASK: u32 = 0x00400000; // Bit 23
const Z_FIXED_MASK: u32 = 0x00200000; // Bit 22
const Y_FIXED_MASK: u32 = 0x00100000; // Bit 21
const X_FIXED_MASK: u32 = 0x00080000; // Bit 20
const ROTATION_Z_FIXED_MASK: u32 = 0x00040000; // Bit 19
const ROTATION_Y_FIXED_MASK: u32 = 0x00020000; // Bit 18
const ROTATION_X_FIXED_MASK: u32 = 0x00010000; // Bit 17
const SCALE_Z_FIXED_MASK: u32 = 0x00008000; // Bit 16
const SCALE_Y_FIXED_MASK: u32 = 0x00004000; // Bit 15
const SCALE_X_FIXED_MASK: u32 = 0x00002000; // Bit 14
const DISABLE_CLASSIC_SCALE_MASK: u32 = 0x00001000; // Bit 13
const APPLY_CHILD_SCALE_COMPENSATE_MASK: u32 = 0x00000800; // Bit 12
const APPLY_SCALE_COMPENSATE_MASK: u32 = 0x00000400; // Bit 11
const USE_MODEL_TRANSLATION_MASK: u32 = 0x00000200; // Bit 10
const USE_MODEL_ROTATION_MASK: u32 = 0x00000100; // Bit 9
const USE_MODEL_SCALE_MASK: u32 = 0x00000080; // Bit 8
const TRANSLATION_ISOTROPIC_MASK: u32 = 0x00000040; // Bit 7
const ROTATION_ISOTROPIC_MASK: u32 = 0x0000020; // Bit 6
const SCALE_UNIFORM_MASK: u32 = 0x00000010; // Bit 5
const SCALE_ISOTROPIC_MASK: u32 = 0x00000008; // Bit 4
const ROTATION_TRANSLATION_ISOTROPIC_MASK: u32 = 0x00000004; // Bit 3
const USE_IDENTITY_MASK: u32 = 0x00000002; // Bit 2

macro_rules! apply_masks {
    ($($x:ident),*) => {
        paste::paste! {
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct AnimationTypeCode {
                pub translation_format: u8,
                pub rotation_format: u8,
                pub scale_format: u8,
                $(pub $x: bool),*
            }

            impl AnimationTypeCode {
                fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
                    let flags = reader.read_u32::<BigEndian>()?;
                    Ok(Self::from(flags))
                }
            }

            impl From<u32> for AnimationTypeCode {
                fn from(v: u32) -> Self {
                    Self {
                        translation_format: ((v & TRANSLATION_FORMAT_MASK) >> 30) as u8,
                        rotation_format: ((v & ROTATION_FORMAT_MASK) >> 27) as u8,
                        scale_format: ((v & SCALE_FORMAT_MASK) >> 25) as u8,
                        $(
                            $x: (v & [<$x:upper _MASK>]) == [<$x:upper _MASK>]
                        ),*
                    }
                }
            }
        }
    }
}

apply_masks! {
    has_translation,
    has_rotation,
    has_scale,
    z_fixed,
    y_fixed,
    x_fixed,
    rotation_z_fixed,
    rotation_y_fixed,
    rotation_x_fixed,
    scale_z_fixed,
    scale_y_fixed,
    scale_x_fixed,
    disable_classic_scale,
    apply_child_scale_compensate,
    apply_scale_compensate,
    use_model_translation,
    use_model_rotation,
    use_model_scale,
    translation_isotropic,
    rotation_isotropic,
    scale_uniform,
    scale_isotropic,
    rotation_translation_isotropic,
    use_identity
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationData {}

impl Decode for AnimationData {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let bone_name_offset = reader.read_u32::<BigEndian>()?;
        let anim_ty_code = AnimationTypeCode::decode(reader)?;

        dbg!(anim_ty_code);

        todo!("animation data after animation type code");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chr0Subfile {
    pub subfile_header: SubfileHeader,
    pub chr0_header: Chr0Header,
    pub animation_data: AnimationData,
}

impl Chr0Subfile {
    pub fn decode(index: &IndexGroupEntry, reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        tracing::trace!("Reading CHR0 file");

        let subfile_header = SubfileHeader::decode(reader, SubfileType::Chr0)?;

        reader.set_position(reader.position() + 4); // there are 4 bytes of padding between the headers
        let chr0_header = Chr0Header::decode(reader)?;

        reader.set_position(index.data_pointer as u64);
        let anim_data = AnimationData::decode(reader)?;

        todo!();
    }
}

impl Subfile for Chr0Subfile {
    const MAGIC: [u8; 4] = [0x43, 0x48, 0x52, 0x30]; // "CHR0"
}
