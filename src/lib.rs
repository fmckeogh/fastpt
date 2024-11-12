use crate::sync::{find_next_sync, PT_PSB_HI, PT_PSB_LO, PT_PSB_REPEAT_COUNT};

pub mod sync;

pub struct Decoder<'data> {
    data: &'data [u8],
    current_pos: usize,
    last_sync_offset: usize,
}

impl<'data> Decoder<'data> {
    pub fn new(data: &'data [u8]) -> Self {
        Self {
            data,
            current_pos: 0,
            last_sync_offset: 0,
        }
    }

    pub fn sync_forward(&mut self) {
        match find_next_sync(&self.data[self.last_sync_offset..]) {
            Some(n) => self.current_pos = n,
            None => panic!("Failed to find sync point"),
        }
    }

    pub fn sync_offset(&self) -> usize {
        self.last_sync_offset
    }
    pub fn offset(&self) -> usize {
        self.current_pos
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.current_pos = offset;
    }

    fn pos_inc(&mut self) -> u8 {
        let byte = unsafe { *self.data.get_unchecked(self.current_pos) };
        self.current_pos += 1;
        byte
    }

    fn psb(&mut self) {
        for _ in 0..PT_PSB_REPEAT_COUNT {
            if self.pos_inc() != PT_PSB_HI {
                panic!("bad packet")
            }

            if self.pos_inc() != PT_PSB_LO {
                panic!("bad packet")
            }
        }
    }

    fn process_next_packet(&mut self) -> Result<Option<u64>, Error> {
        let opcode = self.pos_inc();

        match opcode {
            OPCODE_PAD => {
                //println!("pad");
                // self.current_pos += 1;
            }
            OPCODE_EXT => {
                let ext = self.pos_inc();

                match ext {
                    OPCODE_EXT_PSB => {
                        // println!("psb");
                        self.psb()
                    }
                    OPCODE_EXT_PSBEND => {
                        //println!("psbend");
                        self.current_pos += 0;
                    }
                    OPCODE_EXT_CBR => {
                        // println!("cbr");
                        self.current_pos += pt_pl_cbr_size;
                    }
                    0x32 => {
                        // println!("ptw");

                        let payload = u64::from_ne_bytes(
                            self.data[self.current_pos..self.current_pos + 8]
                                .try_into()
                                .unwrap(),
                        );

                        self.current_pos += 8;
                        return Ok(Some(payload));
                    }
                    _ => return Err(Error::UnknownExtOpcode), //panic!("unknown ext opcode: {ext:#x}"),
                }
            }

            _ => {
                if (opcode & 0x01) == 0 {
                    //println!("tnt8");
                    //   return pt_pkt_decode_tnt_8(decoder, packet);
                } else {
                    //panic!("unknown opcode: {opcode:#x}, {:#x}", self.current_pos)
                    return Err(Error::UnknownOpcode);
                }
            }
        }

        Ok(None)
    }
}

impl<'data> Iterator for Decoder<'data> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.process_next_packet() {
                Ok(None) => (),
                Ok(Some(payload)) => break Some(payload),
                Err(_) => break None,
            }
        }
    }
}

#[repr(u8)]
enum Opcode {
    Pad = 0x00,
    Ext = 0x02,
    //    Psb = pt_opc_ext, = 0x02
    Tip = 0x0d,
    //    pt_opc_tnt_8 = 0x00,
    TipPge = 0x11,
    TipPgd = 0x01,
    Fup = 0x1d,
    Mode = 0x99,
    Tsc = 0x19,
    Mtc = 0x59,
    Cyc = 0x03,
    Trig = 0xd9,

    /* A free opcode to trigger a decode fault. */
    Bad = 0xc9,
}

// enum pt_ext_code {
// 	pt_ext_psb		= 0x82,
// 	pt_ext_tnt_64		= 0xa3,
// 	pt_ext_pip		= 0x43,
// 	pt_ext_ovf		= 0xf3,
// 	pt_ext_psbend		= 0x23,
// 	pt_ext_cbr		= 0x03,
// 	pt_ext_tma		= 0x73,
// 	pt_ext_stop		= 0x83,
// 	pt_ext_vmcs		= 0xc8,
// 	pt_ext_ext2		= 0xc3,
// 	pt_ext_exstop		= 0x62,
// 	pt_ext_exstop_ip	= 0xe2,
// 	pt_ext_mwait		= 0xc2,
// 	pt_ext_pwre		= 0x22,
// 	pt_ext_pwrx		= 0xa2,
// 	pt_ext_ptw		= 0x12,
// 	pt_ext_cfe		= 0x13,
// 	pt_ext_evd		= 0x53,

// 	pt_ext_bad		= 0x04
// };

const OPCODE_PAD: u8 = 0x00;
const OPCODE_EXT: u8 = 0x02;
const OPCODE_EXT_PSB: u8 = 0x82;
const OPCODE_EXT_PSBEND: u8 = 0x23;
const OPCODE_EXT_CBR: u8 = 0x03;
const OPCODE_EXT_PTW: u8 = 0x12;

const pt_opcs_cbr: usize = 2;
const pt_pl_cbr_size: usize = 2;
const PTPS_CBR: usize = pt_opcs_cbr + pt_pl_cbr_size;

const pt_opc_cyc: u8 = 0x3;

enum Error {
    UnknownOpcode,
    UnknownExtOpcode,
}
