use std::{
    fmt::Debug,
    io::{self, Cursor},
};
use tokio::io::{AsyncWriteExt, BufWriter};

use bytes::Buf;

/// Clocking-Server で扱うフレームを表すトレイトです。
pub trait ClockingFrame: Sized + Send + Sync + Debug + PartialEq {
    type Context;
    const MINIMAL_FRAME_SIZE: usize;

    /// フレームの解析を行います。
    ///
    /// フレームが解析可能であれば、`Some(Self)`を返し、そうでない場合は`None`を返します。
    #[allow(async_fn_in_trait)]
    async fn parse_frame(cursor: &mut Cursor<&[u8]>, ctx: &Self::Context) -> Option<Self> {
        if cursor.remaining() < Self::MINIMAL_FRAME_SIZE {
            return None;
        }
        Self::parse_frame_unchecked(cursor, ctx).await
    }

    /// フレームの解析を行います。
    ///
    /// **このメソッドは直接呼びだすべきではありません。**  
    /// 代わりに、`parse_frame`を呼び出してください。
    ///
    /// 以下は、`parse_frame_unchecked`を実装する際のための注釈です。
    ///
    /// ## 期待される動作
    /// 1. フレームの大きさが足りない場合は、直ちに `None` を返します。
    /// 2. フレームの大きさが足りている場合は、解析を行います
    /// 3. 解析に成功した場合は、`Some(Self)` を返します。
    ///    このとき、`cursor` は、フレームの終端まで読み捨てている必要があります。
    /// 4. 解析に失敗した場合は、できるかぎり早期に`None` を返します。
    ///
    /// この関数が呼び出された時点で、`cursor`は少なくとも `Self::MINIMAL_FRAME_SIZE` バイトのデータを保持していることが保証されるので、  
    /// それに配慮する必要ありません。
    #[allow(async_fn_in_trait)]
    async fn parse_frame_unchecked(cursor: &mut Cursor<&[u8]>, ctx: &Self::Context)
        -> Option<Self>;
    /// フレームの書き込みを行います。
    #[allow(async_fn_in_trait)]
    async fn write_frame<W: AsyncWriteExt + Unpin>(
        &self,
        stream: &mut BufWriter<W>,
        ctx: &Self::Context,
    ) -> io::Result<()>;
}

#[cfg(test)]
pub mod test_util {
    use std::io::Cursor;

    use super::ClockingFrame;
    use bytes::Buf;
    use pretty_assertions::assert_eq;
    use tokio::io::{AsyncWriteExt, BufWriter};

    pub async fn encode<T: ClockingFrame>(payload: &T, ctx: &T::Context) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut writer = BufWriter::new(&mut buf);

        payload.write_frame(&mut writer, ctx).await.unwrap();
        writer.flush().await.unwrap();

        buf
    }

    pub async fn decode<'a, T: ClockingFrame>(
        buf: &'a [u8],
        ctx: &T::Context,
    ) -> (Option<T>, Cursor<&'a [u8]>) {
        let mut cursor = std::io::Cursor::new(buf);
        (T::parse_frame(&mut cursor, ctx).await, cursor)
    }

    /// あるペイロードを、フレームに変換して、そのフレームを解析した結果が、元のペイロードと一致することを確認します。
    pub async fn test_clockingframe_reflective<T: ClockingFrame>(payload: T, ctx: T::Context) {
        let binary = encode(&payload, &ctx).await;
        let (re_encoded, cursor) = decode::<T>(&binary, &ctx).await;

        assert_eq!(re_encoded.unwrap(), payload);
        assert_eq!(cursor.has_remaining(), false);
    }

    /// フレームの解析が失敗することを確認します。
    ///
    /// `fail_at`が指定された場合、その位置で解析が失敗することを確認します。
    pub async fn test_clockingframe_mustfail<T: ClockingFrame>(
        buf: &[u8],
        ctx: &T::Context,
        fail_at: Option<usize>,
    ) {
        let (decoded, cursor) = decode::<T>(buf, ctx).await;
        assert_eq!(decoded, None);
        if let Some(fail_at) = fail_at {
            assert_eq!(cursor.position() as usize, fail_at);
        }
    }
}
