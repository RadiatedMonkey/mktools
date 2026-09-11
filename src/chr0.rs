use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    brres::{IndexGroup, IndexGroupEntry, Subfile, SubfileHeader, SubfileType},
    encoding::{Decode, ReadStringExt},
    error::{CorruptionError, EncodingError, EncodingResult},
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
                return Err(CorruptionError {
                    reason: format!("invalid animation policy: {value} (expected 0 or 1)"),
                    ..Default::default()
                }
                .into());
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
                return Err(CorruptionError {
                    reason: format!("invalid scaling rule: {} (expected 0, 1 or 2)", value),
                    ..Default::default()
                }
                .into());
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
                return Err(CorruptionError {
                    reason: format!("invalid animation format: {value:#b}"),
                    ..Default::default()
                }
                .into());
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
            pub struct AnimationCode {
                pub translation_format: AnimationFormat,
                pub rotation_format: AnimationFormat,
                pub scale_format: AnimationFormat,
                $(pub $x: bool),*
            }

            impl AnimationCode {
                fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
                    let flags = reader.read_u32::<BigEndian>()?;
                    tracing::trace!("Animation type code is {flags:#X?}");

                    Self::try_from(flags)
                }
            }

            impl TryFrom<u32> for AnimationCode {
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

/// The type of animation that is applied to the bone.
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    /// The bone stays in place throughout the entire animation.
    Fixed(f32),
    /// The bone is animated.
    Animated(AnimationFrames),
}

/// A 4-byte animation frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame4Info {
    /// The index of this frame in the animation.
    pub index: u8,
    pub step: u8,
    /// The tangent line to the interpolation slope of the animation.
    pub tangent: f32,
}

impl Frame4Info {
    pub const INDEX_MASK: u32 = 0xff000000; // Top 12 bits
    pub const STEP_MASK: u32 = 0x00fff000; // Middle 12 bits
    pub const TANGENT_MASK: u32 = 0x00000fff; // Bottom 8 bits
}

impl Decode for Frame4Info {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;

        let index = ((word & Self::INDEX_MASK) >> 24) as u8;
        let step = ((word & Self::STEP_MASK) >> 12) as u8;
        let tangent = (word >> 20) as f32 / 32.0;

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

impl Decode for Frame6Info {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let index = (reader.read_u16::<BigEndian>()? as f32) / 32.0f32;
        let step = reader.read_u16::<BigEndian>()? as f32;
        let tangent = (reader.read_u16::<BigEndian>()? as f32) / 256.0f32;

        Ok(Self {
            index,
            step,
            tangent,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame12Info {
    pub index: f32,
    pub value: f32,
    pub tangent: f32,
}

impl Decode for Frame12Info {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let index = reader.read_f32::<BigEndian>()?;
        let value = reader.read_f32::<BigEndian>()?;
        let tangent = reader.read_f32::<BigEndian>()?;

        Ok(Self {
            index,
            value,
            tangent,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interpolated4Frame {
    pub frame_scale: f32,
    pub step: f32,
    pub base: f32,
    pub frames: Vec<Frame4Info>,
}

impl Decode for Interpolated4Frame {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let frame_count = reader.read_u16::<BigEndian>()?;
        tracing::trace!(
            "Reading {frame_count} I4 frames at location {}",
            reader.position()
        );

        let _unknown1 = reader.read_u16::<BigEndian>()?;
        let frame_scale = reader.read_f32::<BigEndian>()?;
        let step = reader.read_f32::<BigEndian>()?;
        let base = reader.read_f32::<BigEndian>()?;

        let mut frames = Vec::with_capacity(frame_count as usize);
        for _ in 0..frame_count {
            frames.push(Frame4Info::decode(reader)?);
        }

        Ok(Self {
            frame_scale,
            step,
            base,
            frames,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interpolated6Frame {
    pub frame_scale: f32,
    pub step: f32,
    pub base: f32,
    pub frames: Vec<Frame6Info>,
}

impl Decode for Interpolated6Frame {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let frame_count = reader.read_u16::<BigEndian>()?;
        tracing::trace!(
            "Reading {frame_count} I6 frames at location {}",
            reader.position()
        );

        let _unknown1 = reader.read_u16::<BigEndian>()?;
        let frame_scale = reader.read_f32::<BigEndian>()?;
        let step = reader.read_f32::<BigEndian>()?;
        let base = reader.read_f32::<BigEndian>()?;

        let mut frames = Vec::with_capacity(frame_count as usize);
        for _ in 0..frame_count {
            frames.push(Frame6Info::decode(reader)?);
        }

        Ok(Self {
            frame_scale,
            step,
            base,
            frames,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interpolated12Frame {
    pub frame_scale: f32,
    pub frames: Vec<Frame12Info>,
}

impl Decode for Interpolated12Frame {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let frame_count = reader.read_u16::<BigEndian>()?;
        tracing::trace!(
            "Reading {frame_count} I12 frames at location {}",
            reader.position()
        );

        let _unknown1 = reader.read_u16::<BigEndian>()?;
        let frame_scale = reader.read_f32::<BigEndian>()?;

        let mut frames = Vec::with_capacity(frame_count as usize);
        for _ in 0..frame_count {
            frames.push(Frame12Info::decode(reader)?);
        }

        Ok(Self {
            frame_scale,
            frames,
        })
    }
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
        bone_data_start: u32,
        format: AnimationFormat,
    ) -> EncodingResult<AnimationFrames> {
        let orig_position = reader.position();
        let frame_offset = reader.read_i32::<BigEndian>()? as i64;

        let frame_start = bone_data_start as i64 + frame_offset;
        reader.set_position(frame_start as u64);

        let frames = match format {
            AnimationFormat::Interpolated4 => {
                AnimationFrames::Interpolated4(Interpolated4Frame::decode(reader)?)
            }
            AnimationFormat::Interpolated6 => {
                AnimationFrames::Interpolated6(Interpolated6Frame::decode(reader)?)
            }
            AnimationFormat::Interpolated12 => {
                AnimationFrames::Interpolated12(Interpolated12Frame::decode(reader)?)
            }
            _ => todo!("animation frame format {format:?}"),
        };

        reader.set_position(orig_position + 4);
        Ok(frames)
    }

    fn decode_scale(
        reader: &mut Cursor<&[u8]>,
        bone_data_start: u32,
        anim_ty_code: &AnimationCode,
    ) -> EncodingResult<ComponentData> {
        tracing::trace!(
            "Reading scale animations (iso: {}, x fixed: {}, y fixed: {}, z fixed: {})",
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
                    Self::decode_anim_frame(reader, bone_data_start, anim_ty_code.scale_format)?;
                iso_scale = ComponentType::Animated(frame);
            }

            Ok(ComponentData {
                x: iso_scale.clone(),
                y: iso_scale.clone(),
                z: iso_scale,
            })
        } else {
            let x_scale;
            if anim_ty_code.scale_x_fixed {
                x_scale = ComponentType::Fixed(reader.read_f32::<BigEndian>()?);
            } else {
                let frame =
                    Self::decode_anim_frame(reader, bone_data_start, anim_ty_code.scale_format)?;
                x_scale = ComponentType::Animated(frame);
            }

            let y_scale;
            if anim_ty_code.scale_y_fixed {
                y_scale = ComponentType::Fixed(reader.read_f32::<BigEndian>()?);
            } else {
                let frame =
                    Self::decode_anim_frame(reader, bone_data_start, anim_ty_code.scale_format)?;
                y_scale = ComponentType::Animated(frame);
            }

            let z_scale;
            if anim_ty_code.scale_z_fixed {
                z_scale = ComponentType::Fixed(reader.read_f32::<BigEndian>()?);
            } else {
                let frame =
                    Self::decode_anim_frame(reader, bone_data_start, anim_ty_code.scale_format)?;
                z_scale = ComponentType::Animated(frame);
            }

            Ok(ComponentData {
                x: x_scale,
                y: y_scale,
                z: z_scale,
            })
        }
    }

    fn decode_rotation(
        reader: &mut Cursor<&[u8]>,
        bone_data_start: u32,
        anim_code: &AnimationCode,
    ) -> EncodingResult<ComponentData> {
        tracing::trace!(
            "Reading rotation animations (iso: {}, x fixed: {}, y fixed: {}, z fixed: {})",
            anim_code.rotation_isotropic,
            anim_code.rotation_x_fixed,
            anim_code.rotation_y_fixed,
            anim_code.rotation_z_fixed
        );

        if anim_code.rotation_isotropic {
            let iso_rot = if anim_code.rotation_x_fixed {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                let frame =
                    Self::decode_anim_frame(reader, bone_data_start, anim_code.rotation_format)?;
                ComponentType::Animated(frame)
            };

            Ok(ComponentData {
                x: iso_rot.clone(),
                y: iso_rot.clone(),
                z: iso_rot,
            })
        } else {
            let x_rot = if anim_code.rotation_x_fixed {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                let frame =
                    Self::decode_anim_frame(reader, bone_data_start, anim_code.rotation_format)?;
                ComponentType::Animated(frame)
            };

            let y_rot = if anim_code.rotation_y_fixed {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                let frame =
                    Self::decode_anim_frame(reader, bone_data_start, anim_code.rotation_format)?;
                ComponentType::Animated(frame)
            };

            let z_rot = if anim_code.rotation_z_fixed {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                let frame =
                    Self::decode_anim_frame(reader, bone_data_start, anim_code.rotation_format)?;
                ComponentType::Animated(frame)
            };

            Ok(ComponentData {
                x: x_rot,
                y: y_rot,
                z: z_rot,
            })
        }
    }

    fn decode_translation(
        reader: &mut Cursor<&[u8]>,
        bone_data_start: u32,
        anim_code: &AnimationCode,
    ) -> EncodingResult<ComponentData> {
        tracing::trace!(
            "Reading translation animations (iso: {}, x fixed: {}, y fixed: {}, z fixed: {})",
            anim_code.translation_isotropic,
            anim_code.x_fixed,
            anim_code.y_fixed,
            anim_code.z_fixed
        );

        if anim_code.translation_isotropic {
            let iso_trans = if anim_code.x_fixed {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                ComponentType::Animated(Self::decode_anim_frame(
                    reader,
                    bone_data_start,
                    anim_code.translation_format,
                )?)
            };

            Ok(ComponentData {
                x: iso_trans.clone(),
                y: iso_trans.clone(),
                z: iso_trans,
            })
        } else {
            let trans_x = if anim_code.x_fixed {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                ComponentType::Animated(Self::decode_anim_frame(
                    reader,
                    bone_data_start,
                    anim_code.translation_format,
                )?)
            };

            let trans_y = if anim_code.y_fixed {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                ComponentType::Animated(Self::decode_anim_frame(
                    reader,
                    bone_data_start,
                    anim_code.translation_format,
                )?)
            };

            let trans_z = if anim_code.z_fixed {
                ComponentType::Fixed(reader.read_f32::<BigEndian>()?)
            } else {
                ComponentType::Animated(Self::decode_anim_frame(
                    reader,
                    bone_data_start,
                    anim_code.translation_format,
                )?)
            };

            Ok(ComponentData {
                x: trans_x,
                y: trans_y,
                z: trans_z,
            })
        }
    }

    pub fn decode(
        reader: &mut Cursor<&[u8]>,
        bone_data_start: u32,
        anim_code: &AnimationCode,
    ) -> EncodingResult<Self> {
        let scale = if anim_code.has_scale {
            Some(Self::decode_scale(reader, bone_data_start, anim_code)?)
        } else {
            None
        };

        let rotation = if anim_code.has_rotation {
            Some(Self::decode_rotation(reader, bone_data_start, anim_code)?)
        } else {
            None
        };

        let translation = if anim_code.has_translation {
            Some(Self::decode_translation(
                reader,
                bone_data_start,
                anim_code,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Chr0Bone {
    /// Name of the bone that this animates.
    pub name: String,
    pub anim_ty_code: AnimationCode,
    pub anim_data: AnimationData,
}

impl Chr0Bone {
    #[tracing::instrument(skip(reader, bone_data_start))]
    pub fn decode(
        reader: &mut Cursor<&[u8]>,
        bone_data_start: u32,
        name: &str,
    ) -> EncodingResult<Self> {
        // Points to the same string as the file name in the index group entry,
        // so we don't need it.
        let _bone_name_offset = reader.read_u32::<BigEndian>()?;
        let anim_ty_code = AnimationCode::decode(reader)?;
        let anim_data = AnimationData::decode(reader, bone_data_start, &anim_ty_code)?;

        Ok(Self {
            name: name.to_owned(),
            anim_ty_code,
            // anim_flags,
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

            let data_start = bones_group.get_entry_data_start(entry);
            reader.set_position(data_start as u64);

            tracing::trace!("Reading CHR0 animations for bone `{name}` at location {data_start}");
            bones.push(Chr0Bone::decode(reader, data_start, name)?);
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
