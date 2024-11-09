use libipt::{
    packet::{Packet, PacketDecoder},
    ConfigBuilder,
};
use memmap2::MmapMut;
use std::{env::args, fs::File, time::Instant};

fn main() {
    let path = args().skip(1).next().unwrap();
    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
        .unwrap();

    let mut data = unsafe { MmapMut::map_mut(&file) }
        .unwrap_or_else(|e| panic!("failed to map {path:?}: {e:?}"));

    let mut decoder = PacketDecoder::new(&ConfigBuilder::new(&mut data).unwrap().finish()).unwrap();
    decoder.sync_forward().unwrap();
    assert_eq!(decoder.sync_offset().unwrap(), 0);

    let mut sum: u64 = 0;
    let start = Instant::now();

    while let Ok(packet) = decoder.next() {
        if let Packet::Ptw(inner) = packet {
            sum = sum.wrapping_add(inner.payload());
        }
    }

    let end = Instant::now();

    println!(
        "{:.2} GB/s, {sum}",
        (data.len() as f64) / ((end - start).as_nanos() as f64)
    );
}
