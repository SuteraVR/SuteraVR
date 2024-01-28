use std::{
    fmt::Debug,
    io::{self, BufWriter, Cursor, Write},
};

/// Clocking-Server で扱うフレームを表すトレイトです。
trait Framable: Sized + Send + Sync + Debug {
    /// フレームの解析を行います。
    ///
    /// ## 期待される動作
    /// 1. フレームの大きさが足りない場合は、直ちに `None` を返します。
    /// 2. フレームの大きさが足りている場合は、解析を行います
    /// 3. 解析に成功した場合は、`Some(Self)` を返します。
    ///    このとき、`cursor` は、フレームの終端まで読み捨てている必要があります。
    /// 4. 解析に失敗した場合は、できるかぎり早期に`None` を返します。
    ///
    async fn parse_frame(cursor: &mut Cursor<&[u8]>) -> Option<Self>;

    /// フレームの書き込みを行います。
    async fn write_frame<W: ?Sized + Write>(&self, cursor: &mut BufWriter<W>) -> io::Result<()>;
}
