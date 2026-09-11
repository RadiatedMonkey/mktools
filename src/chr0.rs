use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    brres::{IndexGroup, IndexGroupEntry, Subfile, SubfileHeader, SubfileType},
    encoding::{Decode, ReadStringExt},
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
    None = 0b000,
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

    /// Applies the given bitmask to the flags, shifts it to the given position and converts it into
    /// an [`AnimationFormat`].
    pub fn from_flag(flag: u32, mask: u32, shift_by: u32) -> EncodingResult<Self> {
        let b = ((flag & mask) >> shift_by) as u8;
        AnimationFormat::try_from(b)
    }
}

impl TryFrom<u8> for AnimationFormat {
    type Error = EncodingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0b000 => Self::None,
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
                pub translation_format: AnimationFormat,
                pub rotation_format: AnimationFormat,
                pub scale_format: AnimationFormat,
                $(pub $x: bool),*
            }

            impl AnimationTypeCode {
                fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
                    let flags = reader.read_u32::<BigEndian>()?;
                    tracing::trace!("Animation type code is {flags:#X?}");

                    Self::try_from(flags)
                }
            }

            impl TryFrom<u32> for AnimationTypeCode {
                type Error = EncodingError;

                fn try_from(v: u32) -> EncodingResult<Self> {
                    Ok(Self {
                        // translation_format: (((v & TRANSLATION_FORMAT_MASK) >> 30) as u8),
                        translation_format: AnimationFormat::from_flag(v, TRANSLATION_FORMAT_MASK, 30)?,
                        rotation_format: AnimationFormat::from_flag(v, ROTATION_FORMAT_MASK, 27)?,
                        scale_format: AnimationFormat::from_flag(v, SCALE_FORMAT_MASK, 25)?,
                        $(
                            $x: (v & [<$x:upper _MASK>]) == [<$x:upper _MASK>]
                        ),*
                    })
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

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    Fixed(f32),
    Animated(AnimationFrames),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame4Info {
    pub index: u8,
    pub step: f32,
    pub tangent: f32,
}

impl Frame4Info {
    pub const INDEX_MASK: u32 = 0xff000000;
    pub const STEP_MASK: u32 = 0x00fff000;
    pub const TANGENT_MASK: u32 = 0x00000fff;
}

impl Decode for Frame4Info {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let frame_info = reader.read_u32::<BigEndian>()?;

        let index = ((frame_info & Self::INDEX_MASK) >> 24) as u8;
        let step = ((frame_info & Self::STEP_MASK) >> 12) as f32;
        let tangent = (frame_info & Self::TANGENT_MASK) as f32;

        Ok(Self {
            index,
            step,
            tangent,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame6Info {
    pub index: f32,
    pub step: f32,
    pub tangent: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame12Info {
    pub index: f32,
    pub value: f32,
    pub tangent: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interpolated4Frame {
    pub frame_scale: f32,
    pub step: f32,
    pub base: f32,
    pub frame_info: Vec<Frame4Info>,
}

impl Decode for Interpolated4Frame {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let frame_count = reader.read_u16::<BigEndian>()?;
        tracing::trace!("Reading {frame_count} I4 frames");

        let _unknown1 = reader.read_u16::<BigEndian>()?;
        let frame_scale = reader.read_f32::<BigEndian>()?;
        let step = reader.read_f32::<BigEndian>()?;
        let base = reader.read_f32::<BigEndian>()?;

        let mut frame_info = Vec::with_capacity(frame_count as usize);
        for _ in 0..frame_count {
            frame_info.push(Frame4Info::decode(reader)?);
        }

        Ok(Self {
            frame_scale,
            step,
            base,
            frame_info,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interpolated6Frame {
    pub frame_scale: f32,
    pub step: f32,
    pub base: f32,
    pub frame_info: Vec<Frame6Info>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interpolated12Frame {
    pub frame_scale: f32,
    pub frame_info: Vec<Frame12Info>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Linear1Frame {
    pub step: f32,
    pub base: f32,
    pub frame_info: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Linear4Frame {}

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationFrames {
    Interpolated4(Interpolated4Frame),
    Interpolated6(Interpolated6Frame),
    Interpolated12(Interpolated12Frame),
    Linear1(Linear1Frame),
    Linear4(Linear4Frame),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentData {
    pub x: ComponentType,
    pub y: ComponentType,
    pub z: ComponentType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimationData {
    pub scale: Option<ComponentData>,
    pub rotation: Option<ComponentData>,
    pub translation: Option<ComponentData>,
}

impl AnimationData {
    fn decode_anim_frame(
        reader: &mut Cursor<&[u8]>,
        subfile_header: &SubfileHeader,
        format: AnimationFormat,
    ) -> EncodingResult<AnimationFrames> {
        tracing::trace!("Decoding animation frames of format {format:?}");

        let frame_offset = reader.read_u32::<BigEndian>()?;
        let frame_start = subfile_header.header_start + frame_offset;

        reader.set_position(frame_start as u64);

        Ok(match format {
            AnimationFormat::Interpolated4 => {
                AnimationFrames::Interpolated4(Interpolated4Frame::decode(reader)?)
            }
            _ => todo!(),
        })
    }

    fn decode_scale(
        reader: &mut Cursor<&[u8]>,
        subfile_header: &SubfileHeader,
        anim_ty_code: &AnimationTypeCode,
    ) -> EncodingResult<ComponentData> {
        tracing::trace!(
            "Decoding scale animations (iso: {}, x fixed: {}, y fixed: {}, z fixed: {})",
            anim_ty_code.scale_isotropic,
            anim_ty_code.scale_x_fixed,
            anim_ty_code.scale_y_fixed,
            anim_ty_code.scale_z_fixed
        );

        if anim_ty_code.scale_isotropic {
            let iso_scale;

            // Only one piece of data is stored, rather than for each component
            if anim_ty_code.scale_x_fixed {
                iso_scale = ComponentType::Fixed(reader.read_f32::<BigEndian>()?);
            } else {
                let frame =
                    Self::decode_anim_frame(reader, subfile_header, anim_ty_code.scale_format)?;
                iso_scale = ComponentType::Animated(frame);
            }

            Ok(ComponentData {
                x: iso_scale.clone(),
                y: iso_scale.clone(),
                z: iso_scale,
            })
        } else {
            let z_scale;
            if anim_ty_code.scale_z_fixed {
                z_scale = ComponentType::Fixed(reader.read_f32::<BigEndian>()?);
            } else {
                let frame =
                    Self::decode_anim_frame(reader, subfile_header, anim_ty_code.scale_format)?;
                z_scale = ComponentType::Animated(frame);
            }

            dbg!(&z_scale);

            let y_scale;
            if anim_ty_code.scale_y_fixed {
                y_scale = ComponentType::Fixed(reader.read_f32::<BigEndian>()?);
            } else {
                let frame =
                    Self::decode_anim_frame(reader, subfile_header, anim_ty_code.scale_format)?;
                y_scale = ComponentType::Animated(frame);
            }

            dbg!(&y_scale);

            let x_scale;
            if anim_ty_code.scale_x_fixed {
                x_scale = ComponentType::Fixed(reader.read_f32::<BigEndian>()?);
            } else {
                let frame =
                    Self::decode_anim_frame(reader, subfile_header, anim_ty_code.scale_format)?;
                x_scale = ComponentType::Animated(frame);
            }

            dbg!(&z_scale);

            Ok(ComponentData {
                x: x_scale,
                y: y_scale,
                z: z_scale,
            })
        }
    }

    fn decode_rotation(
        reader: &mut Cursor<&[u8]>,
        subfile_header: &SubfileHeader,
        anim_ty_code: &AnimationTypeCode,
    ) -> EncodingResult<ComponentData> {
        tracing::trace!(
            "Decoding rotation animations (iso: {}, x fixed: {}, y fixed: {}, z fixed: {})",
            anim_ty_code.rotation_isotropic,
            anim_ty_code.rotation_x_fixed,
            anim_ty_code.rotation_y_fixed,
            anim_ty_code.rotation_z_fixed
        );

        if anim_ty_code.rotation_isotropic {
            let iso_rot;
            if anim_ty_code.rotation_x_fixed {
                iso_rot = ComponentType::Fixed(reader.read_f32::<BigEndian>()?);
            } else {
                let frame =
                    Self::decode_anim_frame(reader, subfile_header, anim_ty_code.rotation_format)?;
                iso_rot = ComponentType::Animated(frame);
            }

            Ok(ComponentData {
                x: iso_rot.clone(),
                y: iso_rot.clone(),
                z: iso_rot,
            })
        } else {
            let z_rot;
            if anim_ty_code.rotation_z_fixed {
                z_rot = ComponentType::Fixed(reader.read_f32::<BigEndian>()?);
            } else {
                let frame =
                    Self::decode_anim_frame(reader, subfile_header, anim_ty_code.rotation_format)?;
                z_rot = ComponentType::Animated(frame);
            }

            let y_rot;
            if anim_ty_code.rotation_y_fixed {
                y_rot = ComponentType::Fixed(reader.read_f32::<BigEndian>()?);
            } else {
                todo!("Seems like the frame info offset read in decode_anim_frame is incorrect");

                let frame =
                    Self::decode_anim_frame(reader, subfile_header, anim_ty_code.rotation_format)?;
                y_rot = ComponentType::Animated(frame);
            }

            let x_rot;
            if anim_ty_code.rotation_x_fixed {
                x_rot = ComponentType::Fixed(reader.read_f32::<BigEndian>()?);
            } else {
                let frame =
                    Self::decode_anim_frame(reader, subfile_header, anim_ty_code.rotation_format)?;
                x_rot = ComponentType::Animated(frame);
            }

            Ok(ComponentData {
                x: x_rot,
                y: y_rot,
                z: z_rot,
            })
        }
    }

    fn decode_translation(
        reader: &mut Cursor<&[u8]>,
        subfile_header: &SubfileHeader,
        anim_ty_code: &AnimationTypeCode,
    ) -> EncodingResult<ComponentData> {
        todo!()
    }

    pub fn decode(
        reader: &mut Cursor<&[u8]>,
        subfile_header: &SubfileHeader,
        anim_ty_code: &AnimationTypeCode,
    ) -> EncodingResult<Self> {
        let scale = if anim_ty_code.has_scale {
            Some(Self::decode_scale(reader, subfile_header, anim_ty_code)?)
        } else {
            None
        };

        let rotation = if anim_ty_code.has_rotation {
            Some(Self::decode_rotation(reader, subfile_header, anim_ty_code)?)
        } else {
            None
        };

        let translation = if anim_ty_code.has_translation {
            Some(Self::decode_translation(
                reader,
                subfile_header,
                anim_ty_code,
            )?)
        } else {
            None
        };

        // If a component is fixed, the current float is simply the value for the bone.
        Ok(AnimationData {
            scale,
            rotation,
            translation,
        })
    }
}

const IDENTITY_SCALE_MASK: u32 = 0x00000001;
const IDENTITY_ROTATION_MASK: u32 = 0x00000002;
const IDENTITY_TRANSLATION_MASK: u32 = 0x00000004;
const UNIFORM_SCALE_MASK: u32 = 0x00000008;
const CONSTANT_SCALE_MASK: u32 = 0x00000010;
const CONSTANT_ROTATION_MASK: u32 = 0x00000020;
const CONSTANT_TRANSLATION_MASK: u32 = 0x00000040;
const CURVE_INTERPOLATION_MASK: u32 = 0x00000080;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationFlags {
    pub identity_scale: bool,
    pub identity_rotation: bool,
    pub identity_translation: bool,
    pub uniform_scale: bool,
    pub constant_scale: bool,
    pub constant_rotation: bool,
    pub constant_translation: bool,
    /// Set when Hermes/Bezier tangents need step-evaluation instead of linear blending.
    pub curve_interpolation: bool,
}

impl Decode for AnimationFlags {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let flags = reader.read_u32::<BigEndian>()?;

        let identity_scale = (flags & IDENTITY_SCALE_MASK) == IDENTITY_SCALE_MASK;
        let identity_rotation = (flags & IDENTITY_ROTATION_MASK) == IDENTITY_ROTATION_MASK;
        let identity_translation = (flags & IDENTITY_TRANSLATION_MASK) == IDENTITY_TRANSLATION_MASK;
        let uniform_scale = (flags & UNIFORM_SCALE_MASK) == UNIFORM_SCALE_MASK;
        let constant_scale = (flags & CONSTANT_SCALE_MASK) == CONSTANT_SCALE_MASK;
        let constant_rotation = (flags & CONSTANT_ROTATION_MASK) == CONSTANT_ROTATION_MASK;
        let constant_translation = (flags & CONSTANT_TRANSLATION_MASK) == CONSTANT_TRANSLATION_MASK;
        let curve_interpolation = (flags & CURVE_INTERPOLATION_MASK) == CURVE_INTERPOLATION_MASK;

        Ok(Self {
            identity_scale,
            identity_rotation,
            identity_translation,
            uniform_scale,
            constant_scale,
            constant_rotation,
            constant_translation,
            curve_interpolation,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chr0Bone {
    /// Name of the bone that this animates.
    pub name: String,
    pub anim_ty_code: AnimationTypeCode,
    pub anim_flags: AnimationFlags,
    pub anim_data: AnimationData,
}

impl Chr0Bone {
    pub fn decode(
        reader: &mut Cursor<&[u8]>,
        subfile_header: &SubfileHeader,
        name: &str,
    ) -> EncodingResult<Self> {
        // Points to the same string as the file name in the index group entry,
        // so we don't need it.
        let _bone_name_offset = reader.read_u32::<BigEndian>()?;
        let anim_ty_code = AnimationTypeCode::decode(reader)?;
        let anim_flags = AnimationFlags::decode(reader)?;
        let anim_data = AnimationData::decode(reader, subfile_header, &anim_ty_code)?;

        Ok(Self {
            name: name.to_owned(),
            anim_ty_code,
            anim_flags,
            anim_data,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chr0Subfile {
    pub subfile_header: SubfileHeader,
    pub chr0_header: Chr0Header,
    /// Per-bone animation data.
    pub bones: Vec<Chr0Bone>,
    /// This index group lists all the individual bones in the CHR0 file.
    pub bones_group: IndexGroup,
}

impl Chr0Subfile {
    pub fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let subfile_header = SubfileHeader::decode(reader, SubfileType::Chr0)?;

        reader.set_position(reader.position() + 4); // there are 4 bytes of padding between the headers
        let chr0_header = Chr0Header::decode(reader)?;

        // This subgroup references all the bones in the animation file.
        let bones_group = IndexGroup::decode(reader)?;
        let mut bones = Vec::with_capacity(bones_group.entries.len());

        for entry in &bones_group.entries[1..] {
            let name = bones_group.get_entry_name(reader.get_ref(), entry)?;
            dbg!(name);

            let data = bones_group.get_entry_data_start(entry);
            reader.set_position(data as u64);

            tracing::trace!("Reading CHR0 animations for bone `{name}` at location {data}");
            bones.push(Chr0Bone::decode(reader, &subfile_header, name)?);
        }

        Ok(Self {
            subfile_header,
            chr0_header,
            bones_group,
            bones,
        })
    }
}

impl Subfile for Chr0Subfile {
    const MAGIC: [u8; 4] = [0x43, 0x48, 0x52, 0x30]; // "CHR0"
}
