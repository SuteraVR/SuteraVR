use std::{
    fmt::Debug,
    io::{self, BufWriter, Cursor, Write},
};

use bytes::Buf;

/// Clocking-Server で扱うフレームを表すトレイトです。
pub trait ClockingFrame: Sized + Send + Sync + Debug {
    type Context;
    const MINIMAL_FRAME_SIZE: usize;

    /// フレームの解析を行います。
    ///
    /// フレームが解析可能であれば、`Some(Self)`を返し、そうでない場合は`None`を返します。
    #[allow(async_fn_in_trait)]
    async fn parse_frame(cursor: &mut Cursor<&[u8]>, ctx: Self::Context) -> Option<Self> {
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
    async fn parse_frame_unchecked(cursor: &mut Cursor<&[u8]>, ctx: Self::Context) -> Option<Self>;

    /// フレームの書き込みを行います。
    #[allow(async_fn_in_trait)]
    async fn write_frame<W: ?Sized + Write>(
        &self,
        cursor: &mut BufWriter<W>,
        ctx: Self::Context,
    ) -> io::Result<()>;
}

pub struct SuteraHeader {
    pub message_length: u64,
}
