use anyhow::Result;
use bytes::{BufMut, BytesMut};

fn main() -> Result<()> {
    let mut bytes = BytesMut::with_capacity(1024);
    bytes.extend_from_slice(b"hello world\n");
    println!("{:?}", bytes);
    let a = bytes.split();
    println!("{:?}", a);
    println!("{:?}", bytes);
    let mut b = a.freeze();
    println!("{:?}", b);

    // find '\n' from b by method binary_search
    let pos = b.iter().position(|&x| x == b'\n').unwrap();
    let c = b.split_to(pos);

    println!("{:?}", c);
    println!("{:?}", b);

    let mut bytes = BytesMut::with_capacity(1024);
    bytes.extend_from_slice(b"hello world\n");
    bytes.put_i64(1234567890);
    println!("{:?}", bytes);
    let pos = bytes.iter().position(|&x| x == b'\n').unwrap();
    println!("pos = {}", pos);
    let mut c = bytes.split_to(pos);
    println!("c = {:?}", c);
    println!("{:?}", bytes);

    c.extend_from_slice(b" world\n");
    println!("{:?}", c);

    bytes.extend_from_slice(b"hello world\n");
    println!("{:?}", bytes);

    // update the third value of bytes to 'S'
    bytes[2] = b'S';
    println!("{:?}", bytes);
    Ok(())
}
