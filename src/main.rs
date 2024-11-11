use libipt::{
    packet::{Packet, PacketDecoder},
    ConfigBuilder, PtErrorCode,
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

    let mut sum: u64 = 0;
    let mut simulated_ringbuf_a = vec![];
    let mut simulated_ringbuf_b = vec![];

    let start = Instant::now();

    data.chunks_mut(1024 * 1024 * 512).for_each(|c| {
        simulated_ringbuf_a.extend_from_slice(c);

        let read = decode(&mut sum, &mut simulated_ringbuf_a);

        simulated_ringbuf_b.clear();
        simulated_ringbuf_b.extend_from_slice(&simulated_ringbuf_a[read..]);
        std::mem::swap(&mut simulated_ringbuf_a, &mut simulated_ringbuf_b);
    });

    let end = Instant::now();

    println!(
        "{:.2} GB/s, {sum}",
        (data.len() as f64) / ((end - start).as_nanos() as f64)
    );
}

fn decode(sum: &mut u64, buf: &mut [u8]) -> usize {
    let mut decoder = PacketDecoder::new(&ConfigBuilder::new(buf).unwrap().finish()).unwrap();
    decoder.sync_forward().unwrap();
    assert_eq!(decoder.sync_offset().unwrap(), 0);

    loop {
        match decoder.next() {
            Ok(Packet::Ptw(inner)) => *sum = sum.wrapping_add(inner.payload()),
            Ok(_) => (),
            Err(e) => match e.code() {
                PtErrorCode::Eos => return dbg!(decoder.sync_offset().unwrap() as usize),
                _ => panic!("{e:?}"),
            },
        }
    }
}
