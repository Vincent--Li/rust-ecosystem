use anyhow::Result;
use bytes::{BufMut, BytesMut};

fn main() -> Result<()> {
    let mut buf = BytesMut::with_capacity(1024);
    // b开头的是字节流, 不是字符串
    buf.extend_from_slice(b"hello world");
    buf.put(&b"goodbye world"[..]);
    buf.put_i64(0xdeadbeef);

    println!("buf: {:?}", buf);
    let a = buf.split();
    println!("{:?}", a);
    println!("{:?}", buf);

    Ok(())
}
