use std::{
    slice,
    io::Result
};
use byteorder::ByteOrder;
use futures::io::{
    AsyncRead,
    AsyncWrite,
    AsyncReadExt,
    AsyncWriteExt
};

//byteorder_async crate seems to be dead and I don't want to maintain a fork, pulled functions here

pub(crate) trait AsyncReadByteOrder: AsyncRead+Unpin {
    #[inline]
    async fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf).await?;
        Ok(buf[0])
    }
    #[inline]
    async fn read_i8(&mut self) -> Result<i8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf).await?;
        Ok(buf[0] as i8)
    }

    #[inline]
    async fn read_u16<T: ByteOrder>(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf).await?;
        Ok(T::read_u16(&buf))
    }

    #[inline]
    async fn read_i16<T: ByteOrder>(&mut self) -> Result<i16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf).await?;
        Ok(T::read_i16(&buf))
    }

    #[inline]
    async fn read_u24<T: ByteOrder>(&mut self) -> Result<u32> {
        let mut buf = [0; 3];
        self.read_exact(&mut buf).await?;
        Ok(T::read_u24(&buf))
    }

    #[inline]
    async fn read_i24<T: ByteOrder>(&mut self) -> Result<i32> {
        let mut buf = [0; 3];
        self.read_exact(&mut buf).await?;
        Ok(T::read_i24(&buf))
    }

    #[inline]
    async fn read_u32<T: ByteOrder>(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf).await?;
        Ok(T::read_u32(&buf))
    }

    #[inline]
    async fn read_i32<T: ByteOrder>(&mut self) -> Result<i32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf).await?;
        Ok(T::read_i32(&buf))
    }

    #[inline]
    async fn read_u48<T: ByteOrder>(&mut self) -> Result<u64> {
        let mut buf = [0; 6];
        self.read_exact(&mut buf).await?;
        Ok(T::read_u48(&buf))
    }

    #[inline]
    async fn read_i48<T: ByteOrder>(&mut self) -> Result<i64> {
        let mut buf = [0; 6];
        self.read_exact(&mut buf).await?;
        Ok(T::read_i48(&buf))
    }

    #[inline]
    async fn read_u64<T: ByteOrder>(&mut self) -> Result<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf).await?;
        Ok(T::read_u64(&buf))
    }

    #[inline]
    async fn read_i64<T: ByteOrder>(&mut self) -> Result<i64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf).await?;
        Ok(T::read_i64(&buf))
    }

    #[inline]
    async fn read_u128<T: ByteOrder>(&mut self) -> Result<u128> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf).await?;
        Ok(T::read_u128(&buf))
    }

    #[inline]
    async fn read_i128<T: ByteOrder>(&mut self) -> Result<i128> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf).await?;
        Ok(T::read_i128(&buf))
    }

    #[inline]
    async fn read_uint<T: ByteOrder>(&mut self, nbytes: usize) -> Result<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf[..nbytes]).await?;
        Ok(T::read_uint(&buf[..nbytes], nbytes))
    }

    #[inline]
    async fn read_int<T: ByteOrder>(&mut self, nbytes: usize) -> Result<i64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf[..nbytes]).await?;
        Ok(T::read_int(&buf[..nbytes], nbytes))
    }

    #[inline]
    async fn read_uint128<T: ByteOrder>(&mut self, nbytes: usize) -> Result<u128> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf[..nbytes]).await?;
        Ok(T::read_uint128(&buf[..nbytes], nbytes))
    }

    #[inline]
    async fn read_int128<T: ByteOrder>(&mut self, nbytes: usize) -> Result<i128> {
        let mut buf = [0; 16];
        self.read_exact(&mut buf[..nbytes]).await?;
        Ok(T::read_int128(&buf[..nbytes], nbytes))
    }

    #[inline]
    async fn read_f32<T: ByteOrder>(&mut self) -> Result<f32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf).await?;
        Ok(T::read_f32(&buf))
    }

    #[inline]
    async fn read_f64<T: ByteOrder>(&mut self) -> Result<f64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf).await?;
        Ok(T::read_f64(&buf))
    }

    #[inline]
    async fn read_u16_into<T: ByteOrder>(&mut self, dst: &mut [u16]) -> Result<()> {
        {
            let buf = unsafe { slice_to_u8_mut(dst) };
            self.read_exact(buf).await?;
        }
        T::from_slice_u16(dst);
        Ok(())
    }

    #[inline]
    async fn read_u32_into<T: ByteOrder>(&mut self, dst: &mut [u32]) -> Result<()> {
        {
            let buf = unsafe { slice_to_u8_mut(dst) };
            self.read_exact(buf).await?;
        }
        T::from_slice_u32(dst);
        Ok(())
    }

    #[inline]
    async fn read_u64_into<T: ByteOrder>(&mut self, dst: &mut [u64]) -> Result<()> {
        {
            let buf = unsafe { slice_to_u8_mut(dst) };
            self.read_exact(buf).await?;
        }
        T::from_slice_u64(dst);
        Ok(())
    }

    #[inline]
    async fn read_u128_into<T: ByteOrder>(
        &mut self,
        dst: &mut [u128],
    ) -> Result<()> {
        {
            let buf = unsafe { slice_to_u8_mut(dst) };
            self.read_exact(buf).await?;
        }
        T::from_slice_u128(dst);
        Ok(())
    }

    #[inline]
    async fn read_i8_into(&mut self, dst: &mut [i8]) -> Result<()> {
        let buf = unsafe { slice_to_u8_mut(dst) };
        self.read_exact(buf).await
    }

    #[inline]
    async fn read_i16_into<T: ByteOrder>(&mut self, dst: &mut [i16]) -> Result<()> {
        {
            let buf = unsafe { slice_to_u8_mut(dst) };
            self.read_exact(buf).await?;
        }
        T::from_slice_i16(dst);
        Ok(())
    }

    #[inline]
    async fn read_i32_into<T: ByteOrder>(&mut self, dst: &mut [i32]) -> Result<()> {
        {
            let buf = unsafe { slice_to_u8_mut(dst) };
            self.read_exact(buf).await?;
        }
        T::from_slice_i32(dst);
        Ok(())
    }

    #[inline]
    async fn read_i64_into<T: ByteOrder>(&mut self, dst: &mut [i64]) -> Result<()> {
        {
            let buf = unsafe { slice_to_u8_mut(dst) };
            self.read_exact(buf).await?;
        }
        T::from_slice_i64(dst);
        Ok(())
    }

    #[inline]
   async fn read_i128_into<T: ByteOrder>(
        &mut self,
        dst: &mut [i128],
    ) -> Result<()> {
        {
            let buf = unsafe { slice_to_u8_mut(dst) };
            self.read_exact(buf).await?;
        }
        T::from_slice_i128(dst);
        Ok(())
    }

    #[inline]
   async fn read_f32_into<T: ByteOrder>(&mut self, dst: &mut [f32]) -> Result<()> {
        {
            let buf = unsafe { slice_to_u8_mut(dst) };
            self.read_exact(buf).await?;
        }
        T::from_slice_f32(dst);
        Ok(())
    }

    #[inline]
   async fn read_f64_into<T: ByteOrder>(&mut self, dst: &mut [f64]) -> Result<()> {
        {
            let buf = unsafe { slice_to_u8_mut(dst) };
            self.read_exact(buf).await?;
        }
        T::from_slice_f64(dst);
        Ok(())
    }
}

impl<R: AsyncRead+Unpin> AsyncReadByteOrder for R {}

pub(crate) trait AsyncWriteByteOrder: AsyncWrite+Unpin {
    #[inline]
    async fn write_u8(&mut self, n: u8) -> Result<()> {
        self.write_all(&[n]).await
    }

    #[inline]
    async fn write_i8(&mut self, n: i8) -> Result<()> {
        self.write_all(&[n as u8]).await
    }

    #[inline]
    async fn write_u16<T: ByteOrder>(&mut self, n: u16) -> Result<()> {
        let mut buf = [0; 2];
        T::write_u16(&mut buf, n);
        self.write_all(&buf).await
    }

    #[inline]
    async fn write_i16<T: ByteOrder>(&mut self, n: i16) -> Result<()> {
        let mut buf = [0; 2];
        T::write_i16(&mut buf, n);
        self.write_all(&buf).await
    }

    #[inline]
    async fn write_u24<T: ByteOrder>(&mut self, n: u32) -> Result<()> {
        let mut buf = [0; 3];
        T::write_u24(&mut buf, n);
        self.write_all(&buf).await
    }

    #[inline]
    async fn write_i24<T: ByteOrder>(&mut self, n: i32) -> Result<()> {
        let mut buf = [0; 3];
        T::write_i24(&mut buf, n);
        self.write_all(&buf).await
    }

    #[inline]
    async fn write_u32<T: ByteOrder>(&mut self, n: u32) -> Result<()> {
        let mut buf = [0; 4];
        T::write_u32(&mut buf, n);
        self.write_all(&buf).await
    }

    #[inline]
    async fn write_i32<T: ByteOrder>(&mut self, n: i32) -> Result<()> {
        let mut buf = [0; 4];
        T::write_i32(&mut buf, n);
        self.write_all(&buf).await
    }

    #[inline]
    async fn write_u48<T: ByteOrder>(&mut self, n: u64) -> Result<()> {
        let mut buf = [0; 6];
        T::write_u48(&mut buf, n);
        self.write_all(&buf).await
    }

    #[inline]
    async fn write_i48<T: ByteOrder>(&mut self, n: i64) -> Result<()> {
        let mut buf = [0; 6];
        T::write_i48(&mut buf, n);
        self.write_all(&buf).await
    }

    #[inline]
    async fn write_u64<T: ByteOrder>(&mut self, n: u64) -> Result<()> {
        let mut buf = [0; 8];
        T::write_u64(&mut buf, n);
        self.write_all(&buf).await
    }

    #[inline]
    async fn write_i64<T: ByteOrder>(&mut self, n: i64) -> Result<()> {
        let mut buf = [0; 8];
        T::write_i64(&mut buf, n);
        self.write_all(&buf).await
    }

    #[inline]
    async fn write_u128<T: ByteOrder>(&mut self, n: u128) -> Result<()> {
        let mut buf = [0; 16];
        T::write_u128(&mut buf, n);
        self.write_all(&buf).await
    }

    #[inline]
    async fn write_i128<T: ByteOrder>(&mut self, n: i128) -> Result<()> {
        let mut buf = [0; 16];
        T::write_i128(&mut buf, n);
        self.write_all(&buf).await
    }

    #[inline]
    async fn write_uint<T: ByteOrder>(
        &mut self,
        n: u64,
        nbytes: usize,
    ) -> Result<()> {
        let mut buf = [0; 8];
        T::write_uint(&mut buf, n, nbytes);
        self.write_all(&buf[0..nbytes]).await
    }

    #[inline]
    async fn write_int<T: ByteOrder>(
        &mut self,
        n: i64,
        nbytes: usize,
    ) -> Result<()> {
        let mut buf = [0; 8];
        T::write_int(&mut buf, n, nbytes);
        self.write_all(&buf[0..nbytes]).await
    }

    #[inline]
    async fn write_uint128<T: ByteOrder>(
        &mut self,
        n: u128,
        nbytes: usize,
    ) -> Result<()> {
        let mut buf = [0; 16];
        T::write_uint128(&mut buf, n, nbytes);
        self.write_all(&buf[0..nbytes]).await
    }

    #[inline]
    async fn write_int128<T: ByteOrder>(
        &mut self,
        n: i128,
        nbytes: usize,
    ) -> Result<()> {
        let mut buf = [0; 16];
        T::write_int128(&mut buf, n, nbytes);
        self.write_all(&buf[0..nbytes]).await
    }

    #[inline]
    async fn write_f32<T: ByteOrder>(&mut self, n: f32) -> Result<()> {
        let mut buf = [0; 4];
        T::write_f32(&mut buf, n);
        self.write_all(&buf).await
    }

    #[inline]
    async fn write_f64<T: ByteOrder>(&mut self, n: f64) -> Result<()> {
        let mut buf = [0; 8];
        T::write_f64(&mut buf, n);
        self.write_all(&buf).await
    }
}

impl<W: AsyncWrite+Unpin> AsyncWriteByteOrder for W {}

#[allow(dead_code)]
#[inline]
unsafe fn slice_to_u8_mut<T: Copy>(slice: &mut [T]) -> &mut [u8] {
    slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut u8, std::mem::size_of_val(slice))
}
