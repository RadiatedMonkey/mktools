use std::io::Cursor;

use crate::error::EncodingResult;

macro_rules! impl_byteorder_arrays {
    ($($ty: ty),*) => {
        paste::paste! {
            pub trait WriteArrayExt {
                fn write_u8_array<const N: usize>(&mut self, values: [u8; N]) -> std::io::Result<()>;
                fn write_i8_array<const N: usize>(&mut self, values: [i8; N]) -> std::io::Result<()>;

                $(
                    fn [<write_ $ty _array>]<const N: usize, B: byteorder::ByteOrder>(&mut self, values: [$ty; N])
                        -> std::io::Result<()>;
                )*
            }

            impl<W: byteorder::WriteBytesExt> WriteArrayExt for W {
                fn write_u8_array<const N: usize>(&mut self, values: [u8; N]) -> std::io::Result<()> {
                    for b in values {
                        self.write_u8(b)?;
                    }

                    Ok(())
                }

                fn write_i8_array<const N: usize>(&mut self, values: [i8; N]) -> std::io::Result<()> {
                    for b in values {
                        self.write_i8(b)?;
                    }

                    Ok(())
                }

                $(
                    fn [< write_ $ty _array >]<const N: usize, B: byteorder::ByteOrder>(&mut self, values: [$ty; N])
                        -> std::io::Result<()>
                    {
                        for b in values {
                            self.[<write_ $ty>]::<B>(b)?;
                        }

                        Ok(())
                    }
                )*
            }

            pub trait ReadArrayExt {
                fn read_u8_array<const N: usize>(&mut self) -> std::io::Result<[u8; N]>;
                fn read_i8_array<const N: usize>(&mut self) -> std::io::Result<[i8; N]>;

                $(
                    fn [< read_ $ty _array >]<const N: usize, B: byteorder::ByteOrder>(&mut self)
                        -> std::io::Result<[$ty; N]>;
                )*
            }

            impl<R: byteorder::ReadBytesExt> ReadArrayExt for R {
                fn read_u8_array<const N: usize>(&mut self) -> std::io::Result<[u8; N]> {
                    let mut array = [0; N];
                    for i in 0..N {
                        array[i] = self.read_u8()?;
                    }

                    Ok(array)
                }

                fn read_i8_array<const N: usize>(&mut self) -> std::io::Result<[i8; N]> {
                    let mut array = [0; N];
                    for i in 0..N {
                        array[i] = self.read_i8()?;
                    }

                    Ok(array)
                }

                $(
                    fn [< read_ $ty _array >]<const N: usize, B: byteorder::ByteOrder>(&mut self)
                        -> std::io::Result<[$ty; N]>
                    {
                        let mut array = [0; N];
                        for i in 0..N {
                            array[i] = self.[< read_ $ty >]::<B>()?;
                        }

                        Ok(array)
                    }
                )*
            }
        }
    }
}

impl_byteorder_arrays!(u16, i16, u32, i32, u64, i64, u128, i128);

pub trait Decode: Sized {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self>;
}

pub trait Encode {
    fn encode(&self) -> EncodingResult<Vec<u8>> {
        let mut out = Vec::new();
        self.encode_into(&mut out)?;

        Ok(out)
    }

    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()>;
}
