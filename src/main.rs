use std::env;
use std::fs::File;
use std::io::{Read, BufReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let firmware = &args[1];

    // メモリ確保 1kB
    let mut memory: [u8; 1024] = [0; 1024]; 

    // バイナリをメモリにロード
    let mut i = 0;
    for buf in BufReader::new(File::open(firmware)?).bytes() {
        memory[i] = buf.unwrap();
        i = i + 1;
    }

    let mut pc : usize = 0;
    let mut instr : u32;
    let mut regfile : [u64; 32] = [0; 32];

    // 各ビットフィールド
    let mut opcode;
    let mut rd : usize;
    let mut rs1 : usize;
    let mut rs2 : usize;
    let mut funct3;
    let mut funct7 : u64;
    let mut imm : i64;
    let mut shamt : i64;

    // 何か
    let mut mem_addr : usize;

    memory[0x20] = 0xff;
    memory[0x21] = 0xff;
    memory[0x22] = 0xff;
    memory[0x23] = 0xff;
    memory[0x24] = 0xff;
    memory[0x25] = 0xff;
    memory[0x26] = 0xff;
    memory[0x27] = 0xff;

    loop {
        // 命令フェッチ
        instr = ((memory[pc + 3] as u32) << (8*3)) | 
                ((memory[pc + 2] as u32) << (8*2)) | 
                ((memory[pc + 1] as u32) << 8) | 
                (memory[pc] as u32);

        opcode = instr & 0x7f;

        match opcode {
            // LOAD
            0b00_000_11 => {
                rd      = (instr as usize >> 7) & 0b11111;
                funct3  = (instr >> 12) & 0b111;
                rs1     = (instr as usize >> 15) & 0b11111;
                imm     = ((instr as i32) >> 20) as i64;
                mem_addr = (imm + regfile[rs1] as i64) as usize;
                println!("LOAD : rd = {:x}, rs1 = {:x}, funct3 = {:x}, mem_addr = {:x}", rd, rs1, funct3, mem_addr);
                match funct3 {
                    // Signed なので符号拡張してやらんといけない
                    // LB
                    0 => regfile[rd] = (memory[mem_addr] as i8) as u64,
                    // LH
                    1 => regfile[rd] = (((memory[mem_addr + 1] as i16) << 8) | memory[mem_addr] as i16) as u64,
                    // LW
                    2 => regfile[rd] = (((memory[mem_addr + 3] as i32) << (8*3)) | 
                                       ((memory[mem_addr + 2] as i32) << (8*2)) | 
                                       ((memory[mem_addr + 1] as i32) << 8) | 
                                       memory[mem_addr] as i32) as u64,
                    // LD
                    3 => regfile[rd] = (((memory[mem_addr + 7] as i64) << (8*7)) | 
                                       ((memory[mem_addr + 6] as i64) << (8*6)) | 
                                       ((memory[mem_addr + 5] as i64) << (8*5)) | 
                                       ((memory[mem_addr + 4] as i64) << (8*4)) | 
                                       ((memory[mem_addr + 3] as i64) << (8*3)) | 
                                       ((memory[mem_addr + 2] as i64) << (8*2)) | 
                                       ((memory[mem_addr + 1] as i64) << 8) | 
                                       memory[mem_addr] as i64) as u64,
                    // Unsigned なのでゼロ拡張してやらんといけん
                    // LBU
                    4 => regfile[rd] = memory[mem_addr] as u64,
                    // LHU
                    5 => regfile[rd] = ((memory[mem_addr + 1] as u64) << 8) | memory[mem_addr] as u64,
                    // LWU
                    6 => regfile[rd] = ((memory[mem_addr + 3] as u64) << (8*3)) | 
                                       ((memory[mem_addr + 2] as u64) << (8*2)) | 
                                       ((memory[mem_addr + 1] as u64) << 8) | 
                                        memory[mem_addr] as u64,
                    _ => {},
                }
            },
            // OP-IMM
            0b00_100_11 => {
                rd      = (instr as usize >> 7) & 0b11111;
                funct3  = (instr >> 12) & 0b111;
                rs1     = (instr as usize >> 15) & 0b11111;
                imm     = ((instr as i32) >> 20) as i64;
                match funct3 {
                    // ADDI
                    0b000 => regfile[rd] = ((regfile[rs1] as i64) + imm) as u64,
                    // SLTI
                    0b010 => regfile[rd] = ((regfile[rs1] as i64) < imm) as u64,
                    // SLTIU
                    0b011 => regfile[rd] = (regfile[rs1] < (imm as u64)) as u64,
                    // XORI
                    0b100 => regfile[rd] = regfile[rs1] ^ imm as u64,
                    // ORI
                    0b110 => regfile[rd] = regfile[rs1] | imm as u64,
                    // ANDI
                    0b111 => regfile[rd] = regfile[rs1] & imm as u64,
                    // SLLI
                    0b001 => regfile[rd] = regfile[rs1] << imm,
                    0b101 => {
                        shamt = imm & 0b11111;
                        match imm >> 5 {
                            // SRLI
                            0b0000000 => regfile[rd] = regfile[rs1] >> shamt,
                            // SRAI
                            0b0100000 => regfile[rd] = ((regfile[rs1] as i64) >> shamt) as u64,
                            _ => {},
                        }
                    },
                    _ => {},
                }
            },
            // AUIPC
            0b00_101_11 => {
            },
            // OP-IMM-32
            0b00_110_11 => {
            },
            // STORE
            0b01_000_11 => {
                funct3  = (instr >> 12) & 0b111;
                rs1     = (instr as usize >> 15) & 0b11111;
                rs2     = (instr as usize >> 20) & 0b11111;
                imm     = ((((instr as i32) >> 25) << 5) | ((instr as i32 >> 7) & 0b11111)) as i64;
                mem_addr = (imm + regfile[rs1] as i64) as usize;
                println!("STORE : rs1 = {:x}, funct3 = {:x}, mem_addr = {:x}, rs2 = {:x}", rs1, funct3, mem_addr, rs2);
                match funct3 {
                    // SB
                    0 => memory[mem_addr] = (0xff & regfile[rs2]) as u8,
                    // SH
                    1 => {
                        memory[mem_addr] = (0xff & regfile[rs2]) as u8;
                        memory[mem_addr + 1] = ((0xff00 & regfile[rs2]) >> 8) as u8;
                    },
                    // SW
                    2 => {
                        memory[mem_addr] = (0xff & regfile[rs2]) as u8;
                        memory[mem_addr + 1] = ((0xff00 & regfile[rs2]) >> 8) as u8;
                        memory[mem_addr + 2] = ((0xff0000 & regfile[rs2]) >> (8*2)) as u8;
                        memory[mem_addr + 3] = ((0xff000000 & regfile[rs2]) >> (8*3)) as u8;
                    },
                    // SD
                    3 => {
                        memory[mem_addr] = (0xff & regfile[rs2]) as u8;
                        memory[mem_addr + 1] = ((0xff00 & regfile[rs2]) >> 8) as u8;
                        memory[mem_addr + 2] = ((0xff0000 & regfile[rs2]) >> (8*2)) as u8;
                        memory[mem_addr + 3] = ((0xff000000 & regfile[rs2]) >> (8*3)) as u8;
                        memory[mem_addr + 4] = ((0xff00000000 & regfile[rs2]) >> (8*4)) as u8;
                        memory[mem_addr + 5] = ((0xff0000000000 & regfile[rs2]) >> (8*5)) as u8;
                        memory[mem_addr + 6] = ((0xff000000000000 & regfile[rs2]) >> (8*6)) as u8;
                        memory[mem_addr + 7] = ((0xff00000000000000 & regfile[rs2]) >> (8*7)) as u8;
                    },
                    _ => {},
                }
            },
            // AMO
            0b01_011_11 => {
            },
            // OP
            0b01_100_11 => {
            },
            // LUI
            0b01_101_11 => {
            },
            // OP-32
            0b01_110_11 => {
            },
            // BRANCH
            0b11_000_11 => {
            },
            // JALR
            0b11_001_11 => {
            },
            // JAL
            0b11_011_11 => {
            },
            // SYSTEM
            0b11_100_11 => {
            },
            // Default
            _ => {
                break;
            },
        }

        pc += 4;

    }

    for buf in regfile {
        println!("{:x}", buf);
    }

    Ok(())
}
