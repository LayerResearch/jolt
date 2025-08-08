use serde::{Deserialize, Serialize};

use crate::declare_riscv_instr;
use crate::emulator::cpu::{Cpu, Xlen};

use super::andi::ANDI;
use super::format::format_load::FormatLoad;
use super::format::format_r::FormatR;
use super::format::format_virtual_halfword_alignment::HalfwordAlignFormat;
use super::ld::LD;
use super::sll::SLL;
use super::slli::SLLI;
use super::srli::SRLI;
use super::virtual_assert_halfword_alignment::VirtualAssertHalfwordAlignment;
use super::virtual_lw::VirtualLW;
use super::xori::XORI;
use super::{addi::ADDI, RV32IMInstruction};
use super::RAMRead;
use common::constants::virtual_register_index;

use super::{
    format::{format_i::FormatI, InstructionFormat},
    RISCVInstruction, RISCVTrace, RV32IMCycle,
};

declare_riscv_instr!(
    name   = LHU,
    mask   = 0x0000707f,
    match  = 0x00005003,
    format = FormatLoad,
    ram    = RAMRead
);

impl LHU {
    fn exec(&self, cpu: &mut Cpu, ram_access: &mut <LHU as RISCVInstruction>::RAMAccess) {
        cpu.x[self.operands.rd as usize] = match cpu
            .mmu
            .load_halfword(cpu.x[self.operands.rs1 as usize].wrapping_add(self.operands.imm) as u64)
        {
            Ok((halfword, memory_read)) => {
                *ram_access = memory_read;
                halfword as i64
            }
            Err(_) => panic!("MMU load error"),
        };
    }
}

impl RISCVTrace for LHU {
    fn trace(&self, cpu: &mut Cpu, trace: Option<&mut Vec<RV32IMCycle>>) {
        let virtual_sequence = self.virtual_sequence(cpu.xlen);
        let mut trace = trace;
        for instr in virtual_sequence {
            // In each iteration, create a new Option containing a re-borrowed reference
            instr.trace(cpu, trace.as_deref_mut());
        }
    }

    fn virtual_sequence(&self, xlen: Xlen) -> Vec<RV32IMInstruction> {
        match xlen {
            Xlen::Bit32 => self.virtual_sequence_32(),
            Xlen::Bit64 => self.virtual_sequence_64(),
        }
    }
}

impl LHU {
    fn virtual_sequence_32(&self) -> Vec<RV32IMInstruction> {
        // Virtual registers used in sequence
        let v_address = virtual_register_index(0);
        let v_word_address = virtual_register_index(1);
        let v_word = virtual_register_index(2);
        let v_shift = virtual_register_index(3);

        let mut sequence = vec![];

        let assert_alignment = VirtualAssertHalfwordAlignment {
            address: self.address,
            operands: HalfwordAlignFormat {
                rs1: self.operands.rs1,
                imm: self.operands.imm,
            },
            virtual_sequence_remaining: Some(8),
            is_compressed: self.is_compressed,
        };
        sequence.push(assert_alignment.into());

        let add = ADDI {
            address: self.address,
            operands: FormatI {
                rd: v_address,
                rs1: self.operands.rs1,
                imm: self.operands.imm as u64,
            },
            virtual_sequence_remaining: Some(7),
            is_compressed: self.is_compressed,
        };
        sequence.push(add.into());

        let andi = ANDI {
            address: self.address,
            operands: FormatI {
                rd: v_word_address,
                rs1: v_address,
                imm: -4i64 as u64,
            },
            virtual_sequence_remaining: Some(6),
            is_compressed: self.is_compressed,
        };
        sequence.push(andi.into());

        let lw = VirtualLW {
            address: self.address,
            operands: FormatI {
                rd: v_word,
                rs1: v_word_address,
                imm: 0,
            },
            virtual_sequence_remaining: Some(5),
            is_compressed: self.is_compressed,
        };
        sequence.push(lw.into());

        let xori = XORI {
            address: self.address,
            operands: FormatI {
                rd: v_shift,
                rs1: v_address,
                imm: 2,
            },
            virtual_sequence_remaining: Some(4),
            is_compressed: self.is_compressed,
        };
        sequence.push(xori.into());

        let slli = SLLI {
            address: self.address,
            operands: FormatI {
                rd: v_shift,
                rs1: v_shift,
                imm: 3,
            },
            virtual_sequence_remaining: Some(3),
            is_compressed: self.is_compressed,
        };
        sequence.extend(slli.virtual_sequence(Xlen::Bit32));

        let sll = SLL {
            address: self.address,
            operands: FormatR {
                rd: self.operands.rd,
                rs1: v_word,
                rs2: v_shift,
            },
            virtual_sequence_remaining: Some(2),
            is_compressed: self.is_compressed,
        };
        sequence.extend(sll.virtual_sequence(Xlen::Bit32));

        let srli = SRLI {
            address: self.address,
            operands: FormatI {
                rd: self.operands.rd,
                rs1: self.operands.rd,
                imm: 16,
            },
            virtual_sequence_remaining: Some(0),
            is_compressed: self.is_compressed,
        };
        sequence.extend(srli.virtual_sequence(Xlen::Bit32));

        sequence
    }

    fn virtual_sequence_64(&self) -> Vec<RV32IMInstruction> {
        // Virtual registers used in sequence
        let v_address = virtual_register_index(6);
        let v_dword_address = virtual_register_index(7);
        let v_dword = virtual_register_index(8);
        let v_shift = virtual_register_index(9);

        let mut sequence = vec![];

        let assert_alignment = VirtualAssertHalfwordAlignment {
            address: self.address,
            operands: HalfwordAlignFormat {
                rs1: self.operands.rs1,
                imm: self.operands.imm,
            },
            virtual_sequence_remaining: Some(8),
            is_compressed: self.is_compressed,
        };
        sequence.push(assert_alignment.into());

        let add = ADDI {
            address: self.address,
            operands: FormatI {
                rd: v_address,
                rs1: self.operands.rs1,
                imm: self.operands.imm as u64,
            },
            virtual_sequence_remaining: Some(7),
            is_compressed: self.is_compressed,
        };
        sequence.push(add.into());

        let andi = ANDI {
            address: self.address,
            operands: FormatI {
                rd: v_dword_address,
                rs1: v_address,
                imm: -8i64 as u64,
            },
            virtual_sequence_remaining: Some(6),
            is_compressed: self.is_compressed,
        };
        sequence.push(andi.into());

        let ld = LD {
            address: self.address,
            operands: FormatLoad {
                rd: v_dword,
                rs1: v_dword_address,
                imm: 0,
            },
            virtual_sequence_remaining: Some(5),
            is_compressed: self.is_compressed,
        };
        sequence.push(ld.into());

        let xori = XORI {
            address: self.address,
            operands: FormatI {
                rd: v_shift,
                rs1: v_address,
                imm: 6,
            },
            virtual_sequence_remaining: Some(4),
            is_compressed: self.is_compressed,
        };
        sequence.push(xori.into());

        let slli = SLLI {
            address: self.address,
            operands: FormatI {
                rd: v_shift,
                rs1: v_shift,
                imm: 3,
            },
            virtual_sequence_remaining: Some(3),
            is_compressed: self.is_compressed,
        };
        sequence.extend(slli.virtual_sequence(Xlen::Bit64));

        let sll = SLL {
            address: self.address,
            operands: FormatR {
                rd: self.operands.rd,
                rs1: v_dword,
                rs2: v_shift,
            },
            virtual_sequence_remaining: Some(2),
            is_compressed: self.is_compressed,
        };
        sequence.extend(sll.virtual_sequence(Xlen::Bit64));

        let srli = SRLI {
            address: self.address,
            operands: FormatI {
                rd: self.operands.rd,
                rs1: self.operands.rd,
                imm: 48,
            },
            virtual_sequence_remaining: Some(0),
            is_compressed: self.is_compressed,
        };
        sequence.extend(srli.virtual_sequence(Xlen::Bit64));

        sequence
    }
}
