use fastpt::Decoder;
use memmap2::Mmap;
use std::{env::args, fs::File, time::Instant};

fn main() {
    let path = args().skip(1).next().unwrap();
    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
        .unwrap();

    let data =
        unsafe { Mmap::map(&file) }.unwrap_or_else(|e| panic!("failed to map {path:?}: {e:?}"));

    let mut decoder = Decoder::new(&data);
    assert_eq!(decoder.sync_offset(), 0);

    let mut sum: u64 = 0;
    let start = Instant::now();

    while let Some(packet) = decoder.next() {
        sum = sum.wrapping_add(packet);
    }

    let end = Instant::now();

    println!(
        "{:.2} GB/s, {:#x} {sum}",
        (decoder.offset() as f64) / ((end - start).as_nanos() as f64),
        decoder.offset()
    );
}
