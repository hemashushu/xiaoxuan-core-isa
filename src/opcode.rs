// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

// the data types that VM internal supports
// ----------------------------------------
//
// - i8
// - i16
// - i32
// - i64
// - f32
// - f64

// data memory layout
// ------------------
//
// the default implement of XiaoXuan Core VM is stack-base, which its operands are
// 8-bytes raw data, the data presentation in memory:
//
//    MSB                             LSB
// 64 |---------------------------------| 0
//    |   16     16      16     8    8  | bits
//    |-------|-------|-------|----|----|
//    |               i64               | <-- native data type
//    |---------------------------------|
//    | sign-extend   |        i32      |
//    |---------------------------------|
//    | sign-extend           |   i16   |
//    |---------------------------------|
//    | sign-extend                | i8 |
//    |---------------------------------|
//    |               f64               | <-- native data type
//    |---------------------------------|
//    | undefined     |        f32      | <-- native data type
//    |---------------------------------|
//
// all i8/i16/i32 integers loaded from memory will be signed-extended to i64

// the floating-point number
// -------------------------
//
// like most processors and VM, f32/f64 is stored with
// IEEE 754-2008 format.
// e.g. the f32 encoding:
//
//           MSB                                  LSB
//           sign    exponent (8 bits)   fraction (23 bits)                                 implicit leading number 1
//           ----    -----------------   ------------------                                 |
//           |       |                   |                                                  v
// format    0       00000000            0000000000 0000000000 000     value = (-1)^sign * (1 + fraction) * 2^(exponent-offset), offset = 127 for f32, 1023 for f64
// example   1       10000001            0100000000 0000000000 000     value = (-1)^1 * (1 + 0*2^(-1) + 1*2^(-2)) * 2^(129-127) = -1 * 1.25 * 4 = -5.0
// example   0       01111100            0100000000 0000000000 000     value = (-1)^0 * 1.25 * 2^(-3) = 0.15625
//
// support?
//  Y        -       00000001--\
//                   11111110--/         ---------- ---------- ---     normal number
//  Y        0       00000000            0000000000 0000000000 000     value = +0
//  N        1       00000000            0000000000 0000000000 000     value = -0
//  Y        -       00000000            ---------- ---------- ---     subnormal number (i.e., numbers between 0 and MIN)
//  N        0       11111111            0000000000 0000000000 000     value = +Infinity
//  N        1       11111111            0000000000 0000000000 000     value = -Infinity
//  N        -       11111111            0-not-all- -zero----- ---     NaN (SNaN)
//  N        -       11111111            1-------- ----------- ---     NaN (QNaN)
//
// ref:
// convert floating-point to decimal/hexadecimal
// https://www.h-schmidt.net/FloatConverter/IEEE754.html

// unsupported floating-point variants
// -----------------------------------
//
// in addition to the normal floating-point numbers,
// there are some special values (variants):
//
// - NaN
// - +Infinity, -Infinity
// - +0, -0
//
// these variants make the programming language become complex and
// sometimes make problems unpredictable, for example:
//
// - NaN is not comparable, everything includes itself compare with NaN
//   result false, e.g.
//   - NaN != NaN
//   - (NaN < 0) == false
//   - (NaN > 0) == false
//   - (a != b) cannot assert that !(a == b)
//
// - the 0.0, -0.0, +Inf and -Inf (Inf means infinity) are complex also, e.g.
//   - 1.0 ÷ 0.0 = +Inf
//   - -1.0 ÷ 0.0 = 1.0 ÷ -0.0 = -Inf
//   - 0.0 ÷ 0.0 = NaN
//   - 0.0 == -0.0
//   - +Inf != -Inf
//
// ref:
//
// - https://en.wikipedia.org/wiki/Floating-point_arithmetic
// - https://en.wikipedia.org/wiki/IEEE_754
// - https://en.wikipedia.org/wiki/IEEE_754-2008_revision
// - https://en.wikipedia.org/wiki/Signed_zero
// - https://en.wikipedia.org/wiki/Subnormal_number
// - https://en.wikipedia.org/wiki/Single-precision_floating-point_format
// - https://en.wikipedia.org/wiki/Half-precision_floating-point_format
// - https://doc.rust-lang.org/std/primitive.f32.html
//
// to simplify the XiaoXuan Core programming language, the f32/f64 in XiaoXuan Core VM
// only support the normal (includes subnormal) floating-point number and +0,
// the VM will simply throw an exception when
// other variants (such as NaN, -0.0, +inf, -Inf) are encountered.
//
// -  0.0 = 0x0000_0000   (Y)
// - -0.0 = 0x8000_0000   (N)
// -  nan = 0xffc0_0000   (N)
// - +inf = 0x7f80_0000   (N)
// - -inf = 0xff80_0000   (N)
//
// when load data from memory as floating-point number, there are some checking:
// 1. exponent between (00000001) and (11111110): pass
// 2. exponent is zero, if the sign bit is zero: pass
// 3. failed.
//
// in other words, the +/-Infinity, -0, NaN, will cause the VM to throw exceptions.

// the boolean type
// ----------------
//
// the boolean value is represented by a i64 number:
// - TRUE, the number is `1:i64`,
// - FALSE, the number is `0:i64`.
//
// when converts integers into boolean, 0:i32 and 0:i64 are both treated as FALSE,
// and all other i32/i64 non-zero are treated as TRUE.

pub const MAX_OPCODE_NUMBER: usize = 0x480;

// the instruction schemes
// -----------------------
//
// XiaoXuan Core VM instructions are NOT fixed-length code.
// there are 16, 32, 64, 96 and 128 bits length instructions,
//
// - 16 bits:
//   instructions without parameters, such as `eqz_i32`.
// - 32 bits:
//   instructions with 1 parameter, such as `add_imm_i32`.
//   16 bits opcode + 16 bits parameter
// - 64 bits:
//   instructions with 1 parameter, such as `imm_i32`.
//   16 bits opcode + (16 bits padding) + 32 bits parameter (ALIGN 4-byte alignment require)
// - 64 bits:
//   instructions with 2 parameters, such as `data_load_i64`.
//   16 bits opcode + 16 bits parameter 0 + 32 bits parameter 1 (ALIGN 4-byte alignment require)
// // - 64 bits:
// //   instructions with 3 parameter, such as `local_load_i64`.
// //   16 bits opcode + 16 bits parameter 1 + 16 bits parameter 2 + 16 bits parameter 3
// - 96 bits
//   instructions with 3 parameters, such as `local_load_i64`.
//   16 bits opcode + (16 bits padding) + 16 bits parameter 0 + 16 bits parameter 1 + 32 bits parameter 2 (ALIGN 4-byte alignment require)
// - 96 bits
//   instructions with 2 parameters, such as `block`.
//   16 bits opcode + (16 bits padding) + 32 bits parameter 0 + 32 bits parameter 1 (ALIGN 4-byte alignment require)
//
// DEPRECATED
// // - 128 bits
// //   instructions with 3 parameters, such as `block_alt`
// //   16 bits opcode + (16 bits padding) + 32 bits parameter 0 + 32 bits parameter 1 + 32 bits parameter 2 (ALIGN 4-byte alignment require)
//
// note that a `nop` instruction will be inserted automatically before
// an instruction which contains `i32` parameters to achieve 32 bits (4-byte) alignment.

// the simplified schemes:
//
// - [opcode i16]                                                                      ;; 16-bit
// - [opcode i16] - [  param i16  ]                                                    ;; 32-bit
// - [opcode i16] - [pading 16-bit] + [       param i32       ]                        ;; 64-bit
// - [opcode i16] - [  param i16  ] + [       param i32       ]                        ;; 64-bit
// // - [opcode i16] - [  param i16  ] + [param i16] + [param i16]                        ;; 64-bit
// - [opcode i16] - [pading 16-bit] + [param i16] + [param i16] + [    param i32    ]  ;; 96-bit
// - [opcode i16] - [pading 16-bit] + [       param i32       ] + [    param i32    ]  ;; 96-bit
//
// DEPRECATED
// // - [opcode i16] - [pading 16-bit] + [param i32              ] + [param i32] + [param i32]     ;; 128-bit
//
// the opcode scheme:
//
// MSB           LSB
// 00000000 00000000
// -------- --------
// |        |
// |        \ items
// |
// \ catalogs
//
#[repr(u16)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Opcode {
    // instruction to do nothing.
    //
    // it's usually used for padding instructions to form 32 bits (4-byte) alignment.
    //
    // () -> ()
    nop = 0x0100,

    // set an immediately number to the top of stack.
    //
    // the "i32" immediately number will be sign extend to i64
    //
    // (param immediate_number:i32) -> i64
    imm_i32 = 0x0140,

    // imm_i64, imm_f32 and imm_f64 are actually pesudo instructions,
    // because there are no i64/f32/f64 parameters for XiaoXuan Core VM instructions.
    //
    // some ISA (VM or real machine) place the immediate numbers in a list of constants
    // in the program image, and then load the constants by address to archieve the
    // purpose of loading the immediate numbers, the ARM ISA has a similar scheme, it
    // places large immediate numbers in the instruction section but outside of the current function
    // (or inside the function and use instruction 'jump' to skip this area so that they
    // are not parsed as instructions).
    //
    // however, the XiaoXuan Core VM instructions are designed to be variable length and
    // don't necessarily require the program to contain a data section or heap,
    // so the immediate numbers are just placed directly in the 'imm_xxx' instructions.
    //
    imm_i64, // (param number_low:i32, number_high:i32) -> i64
    imm_f32, // (param number:i32) -> f32
    imm_f64, // (param number_low:i32, number_high:i32) -> f64

    // loading local variables
    //
    // load the specified local variable and push it to the stack.
    //
    // note that arguments of a function or block are also local variables, the index of local variables
    // follows the arguments, e.g. suppose there are 4 local variables in a function and the function has
    // 2 parameters, the indices of them are as follows:
    //
    //        arguments    local variables
    //        [i32 i32]    [i32 i32 i64 i64]
    // idx     0   1        2   3   4   5
    //
    // in some stack-based VMs, the arguments of a function are placed on the top
    // of the stack, so it is also possible to read the arguments directly in the function
    // using instructions which imply the `pop` capability (e.g. the comparison and
    // the arithmetic instructions).
    // this feature can be used as a trick to improve performance, but the XiaoXuan Core VM doesn't
    // guarantee this feature, the local variables may be placed at an individual place entirely.
    // so you should always use the "local_load" instructions to read arguments.
    //
    // all local variables are 8-byte aligned because by default local variables are allocated
    // on the stack, and the stack is 8-byte aligned.
    local_load_i64 = 0x0180, // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i64
    local_load_i32_s, // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i32
    local_load_i32_u, // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i32
    local_load_i16_s, // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i16
    local_load_i16_u, // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i16
    local_load_i8_s,  // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i8
    local_load_i8_u,  // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i8

    // the INDEX of local variables (data, functions)
    // ----------------------------------------------
    //
    // using the 'index' instead of the 'address/pointer' to access local variables (including
    // data in the data section and functions discussed in the following sections) is the
    // security strategy of the XiaoXuan Core VM.
    // because the 'index' includes the type, length and location (the safe access range) of the object,
    // when accessing the data with index, the VM can check whether the type, and the range are legal
    // or not, so it can prevent a lot of unsafe accesses.
    // for example, the traditional method of using pointers to access a array is very easy
    // to read/write data outside the range.

    // load f64 with floating-point validity check.
    //
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> f64
    local_load_f64,

    // load f32 with floating-point validity check.
    //
    // note that the high part of operand (on the stack) is undefined.
    //
    // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> f32
    local_load_f32,

    // storing local variables
    //
    // pop one operand from the stack and set the specified local variable.
    local_store_i64, // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) (operand value:i64) -> ()
    local_store_i32, // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) (operand value:i32) -> ()
    local_store_i16, // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) (operand value:i32) -> ()
    local_store_i8, // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) (operand value:i32) -> ()
    local_store_f64, // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) (operand value:f64) -> ()
    local_store_f32, // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) (operand value:f32) -> ()

    local_load_extend_i64, // (param reversed_index:i16 local_variable_index:i16) (operand offset_bytes:i64) -> i64
    local_load_extend_i32_s, // (param reversed_index:i16 local_variable_index:i16) (operand offset_bytes:i64) -> i32
    local_load_extend_i32_u, // (param reversed_index:i16 local_variable_index:i16) (operand offset_bytes:i64) -> i32
    local_load_extend_i16_s, // (param reversed_index:i16 local_variable_index:i16) (operand offset_bytes:i64) -> i16
    local_load_extend_i16_u, // (param reversed_index:i16 local_variable_index:i16) (operand offset_bytes:i64) -> i16
    local_load_extend_i8_s, // (param reversed_index:i16 local_variable_index:i16) (operand offset_bytes:i64) -> i8
    local_load_extend_i8_u, // (param reversed_index:i16 local_variable_index:i16) (operand offset_bytes:i64) -> i8
    local_load_extend_f64, // (param reversed_index:i16 local_variable_index:i16) (operand offset_bytes:i64) -> f64
    local_load_extend_f32, // (param reversed_index:i16 local_variable_index:i16) (operand offset_bytes:i64) -> f32

    local_store_extend_i64, // (param reversed_index:i16 local_variable_index:i16) (operand offset_bytes:i64 value:i64) -> ()
    local_store_extend_i32, // (param reversed_index:i16 local_variable_index:i16) (operand offset_bytes:i64 value:i32) -> ()
    local_store_extend_i16, // (param reversed_index:i16 local_variable_index:i16) (operand offset_bytes:i64 value:i32) -> ()
    local_store_extend_i8, // (param reversed_index:i16 local_variable_index:i16) (operand offset_bytes:i64 value:i32) -> ()
    local_store_extend_f64, // (param reversed_index:i16 local_variable_index:i16) (operand offset_bytes:i64 value:f64) -> ()
    local_store_extend_f32, // (param reversed_index:i16 local_variable_index:i16) (operand offset_bytes:i64 value:f32) -> ()

    // loading data
    //
    // i32/i64/f32/f64 data load/store instructions require the address and offset alignment:
    //
    // | data type | align (bytes) |
    // |-----------|---------------|
    // | i8        | 1             |
    // | i16       | 2             |
    // | i32       | 4             |
    // | i64       | 8             |
    // | f32       | 4             |
    // | f64       | 8             |
    //
    // note that all loaded data except the i64 will be signed-extended to i64
    data_load_i64 = 0x01c0, // (param offset_bytes:i16 data_public_index:i32) -> i64
    data_load_i32_s,        // (param offset_bytes:i16 data_public_index:i32) -> i32
    data_load_i32_u,        // (param offset_bytes:i16 data_public_index:i32) -> i32
    data_load_i16_s,        // (param offset_bytes:i16 data_public_index:i32) -> i16
    data_load_i16_u,        // (param offset_bytes:i16 data_public_index:i32) -> i16
    data_load_i8_s,         // (param offset_bytes:i16 data_public_index:i32) -> i8
    data_load_i8_u,         // (param offset_bytes:i16 data_public_index:i32) -> i8

    // data_public_index
    // -----------------
    //
    // the data public index is mixed the following items (and are sorted by the following order):
    //
    // - imported read-only data items
    // - imported read-write data items
    // - imported uninitilized data items
    // - internal read-only data items
    // - internal read-write data items
    // - internal uninitilized data items

    // load f64 with floating-point validity check.
    //
    // (param offset_bytes:i16 data_public_index:i32) -> f64
    data_load_f64,

    // load f32 with floating-point validity check.
    //
    // note that the high part of operand (on the stack) is undefined.
    //
    // (param offset_bytes:i16 data_public_index:i32) -> f32
    data_load_f32,

    // storing data
    data_store_i64, // (param offset_bytes:i16 data_public_index:i32) (operand value:i64) -> ()
    data_store_i32, // (param offset_bytes:i16 data_public_index:i32) (operand value:i32) -> ()
    data_store_i16, // (param offset_bytes:i16 data_public_index:i32) (operand value:i32) -> ()
    data_store_i8,  // (param offset_bytes:i16 data_public_index:i32) (operand value:i32) -> ()
    data_store_f64, // (param offset_bytes:i16 data_public_index:i32) (operand value:f64) -> ()
    data_store_f32, // (param offset_bytes:i16 data_public_index:i32) (operand value:f32) -> ()

    data_load_extend_i64, // (param data_public_index:i32) (operand offset_bytes:i64) -> i64
    data_load_extend_i32_s, // (param data_public_index:i32) (operand offset_bytes:i64) -> i32
    data_load_extend_i32_u, // (param data_public_index:i32) (operand offset_bytes:i64) -> i32
    data_load_extend_i16_s, // (param data_public_index:i32) (operand offset_bytes:i64) -> i16
    data_load_extend_i16_u, // (param data_public_index:i32) (operand offset_bytes:i64) -> i16
    data_load_extend_i8_s, // (param data_public_index:i32) (operand offset_bytes:i64) -> i8
    data_load_extend_i8_u, // (param data_public_index:i32) (operand offset_bytes:i64) -> i8
    data_load_extend_f64, // (param data_public_index:i32) (operand offset_bytes:i64) -> f64
    data_load_extend_f32, // (param data_public_index:i32) (operand offset_bytes:i64) -> f32

    data_store_extend_i64, // (param data_public_index:i32) (operand offset_bytes:i64 value:i64) -> ()
    data_store_extend_i32, // (param data_public_index:i32) (operand offset_bytes:i64 value:i32) -> ()
    data_store_extend_i16, // (param data_public_index:i32) (operand offset_bytes:i64 value:i32) -> ()
    data_store_extend_i8, // (param data_public_index:i32) (operand offset_bytes:i64 value:i32) -> ()
    data_store_extend_f64, // (param data_public_index:i32) (operand offset_bytes:i64 value:f64) -> ()
    data_store_extend_f32, // (param data_public_index:i32) (operand offset_bytes:i64 value:f32) -> ()

    // note:
    // both local variables and data have NO internal data type at all,
    // they are both just bytes in the memory.
    // so you can call 'local_store_i8' and 'local_load_i8_u'
    // even if the local variable is defined as i64.

    // loading heap memory
    //
    // note that the address of heap is a 64-bit integer number.
    heap_load_i64 = 0x0200, // (param offset_bytes:i16) (operand heap_addr:i64) -> i64
    heap_load_i32_s,        // (param offset_bytes:i16) (operand heap_addr:i64) -> i32
    heap_load_i32_u,        // (param offset_bytes:i16) (operand heap_addr:i64) -> i32
    heap_load_i16_s,        // (param offset_bytes:i16) (operand heap_addr:i64) -> i16
    heap_load_i16_u,        // (param offset_bytes:i16) (operand heap_addr:i64) -> i16
    heap_load_i8_s,         // (param offset_bytes:i16) (operand heap_addr:i64) -> i8
    heap_load_i8_u,         // (param offset_bytes:i16) (operand heap_addr:i64) -> i8

    // load f64 with floating-point validity check.
    //
    // (param offset_bytes:i16) (operand heap_addr:i64) -> f64
    heap_load_f64,

    // load f32 with floating-point validity check.
    //
    // note that the high part of operand (on the stack) is undefined
    //
    // (param offset_bytes:i16) (operand heap_addr:i64) -> f32
    heap_load_f32,

    // storing heap memory
    heap_store_i64, // (param offset_bytes:i16) (operand heap_addr:i64 value:i64) -> ()
    heap_store_i32, // (param offset_bytes:i16) (operand heap_addr:i64 value:i32) -> ()
    heap_store_i16, // (param offset_bytes:i16) (operand heap_addr:i64 value:i32) -> ()
    heap_store_i8,  // (param offset_bytes:i16) (operand heap_addr:i64 value:i32) -> ()
    heap_store_f64, // (param offset_bytes:i16) (operand heap_addr:i64 value:f64) -> ()
    heap_store_f32, // (param offset_bytes:i16) (operand heap_addr:i64 value:f32) -> ()

    // loading heap memory with boundary checking
    //     heap_load_bound_i64,   // (operand addr:i64 length_bytes:i32 offset_bytes:i64) -> i64
    //     heap_load_bound_i32_s, // (operand addr:i64 length_bytes:i32 offset_bytes:i64) -> i32
    //     heap_load_bound_i32_u, // (operand addr:i64 length_bytes:i32 offset_bytes:i64) -> i32
    //     heap_load_bound_i16_s, // (operand addr:i64 length_bytes:i32 offset_bytes:i64) -> i16
    //     heap_load_bound_i16_u, // (operand addr:i64 length_bytes:i32 offset_bytes:i64) -> i16
    //     heap_load_bound_i8_s,  // (operand addr:i64 length_bytes:i32 offset_bytes:i64) -> i8
    //     heap_load_bound_i8_u,  // (operand addr:i64 length_bytes:i32 offset_bytes:i64) -> i8
    //     heap_load_bound_f64,   // (operand addr:i64 length_bytes:i32 offset_bytes:i64) -> f64
    //     heap_load_bound_f32,   // (operand addr:i64 length_bytes:i32 offset_bytes:i64) -> f32
    //
    // storing heap memory with boundary checking
    //     heap_store_bound_i64,  // (operand addr:i64 length_bytes:i32 offset_bytes:i64 value:i64) -> ()
    //     heap_store_bound_i32,  // (operand addr:i64 length_bytes:i32 offset_bytes:i64 value:i32) -> ()
    //     heap_store_bound_i16,  // (operand addr:i64 length_bytes:i32 offset_bytes:i64 value:i32) -> ()
    //     heap_store_bound_i8,   // (operand addr:i64 length_bytes:i32 offset_bytes:i64 value:i32) -> ()
    //     heap_store_bound_f64,  // (operand addr:i64 length_bytes:i32 offset_bytes:i64 value:f64) -> ()
    //     heap_store_bound_f32,  // (operand addr:i64 length_bytes:i32 offset_bytes:i64 value:f32) -> ()

    // fill the specified memory region with the specified (i8) value
    //
    // () (operand addr:i64 value:i8 count:i64) -> ()
    heap_fill = 0x0240,

    // copy the specified memory region to the specified location
    //
    // () (operand dst_addr:i64 src_addr:i64 count:i64) -> ()
    heap_copy,

    // return the amount of pages of the heap, by default the size of page
    // is MEMORY_PAGE_SIZE_IN_BYTES (64 KiB).
    //
    // () -> pages:i64
    heap_capacity,

    // increase or decrease the heap size and return the new capacity (in pages)
    //
    // () (operand pages:i64) -> new_pages:i64
    heap_resize,

    // truncate i64 to i32
    //
    // discard the high 32 bits of an i64 number directly
    //
    // () (operand number:i64) -> i32
    truncate_i64_to_i32 = 0x0280,

    // extend i32 to i64
    extend_i32_s_to_i64, // () (operand number:i32) -> i64
    extend_i32_u_to_i64, // () (operand number:i32) -> i64

    // demote f64 to f32
    demote_f64_to_f32, // () (operand number:f64) -> f32

    // promote f32 to f64
    promote_f32_to_f64, // () (operand number:f32) -> f64

    // convert float to int
    //
    // truncate fractional part
    //
    // () (operand number:f32) -> i32
    convert_f32_to_i32_s,

    // note -x.xx(float) -> 0(int)
    // () (operand number:f32) -> i32
    convert_f32_to_i32_u,

    // () (operand number:f64) -> i32
    convert_f64_to_i32_s,

    // note -x.xx(float) -> 0(int)
    // () (operand number:f64) -> i32
    convert_f64_to_i32_u,

    // () (operand number:f32) -> i64
    convert_f32_to_i64_s,

    // note -x.xx(float) -> 0(int)
    // () (operand number:f32) -> i64
    convert_f32_to_i64_u,

    // () (operand number:f64) -> i64
    convert_f64_to_i64_s,

    // note -x.xx(float) -> 0(int)
    // () (operand number:f64) -> i64
    convert_f64_to_i64_u,

    // convert int to float
    convert_i32_s_to_f32, // () (operand number:i32) -> f32
    convert_i32_u_to_f32, // () (operand number:i32) -> f32
    convert_i64_s_to_f32, // () (operand number:i64) -> f32
    convert_i64_u_to_f32, // () (operand number:i64) -> f32
    convert_i32_s_to_f64, // () (operand number:i32) -> f64
    convert_i32_u_to_f64, // () (operand number:i32) -> f64
    convert_i64_s_to_f64, // () (operand number:i64) -> f64
    convert_i64_u_to_f64, // () (operand number:i64) -> f64

    // comparison
    //
    // note that for the binary operations, the first operand pops up from the
    // stack is the right-hand-side value, e.g.
    //
    // |                 | --> stack end
    // | right hand side | --> the 1st pop: RHS
    // | left hand side  | --> the 2nd pop: LHS
    // \-----------------/ --> stack start
    //
    // this is the same order as the function parameters, e.g.
    // the parameters of the function `add (a, b)` on the stack are as follows:
    //
    //  |   | --> stack end
    //  | b |
    //  | a |
    //  \---/ --> stack start
    //
    // two operands MUST be of the same data type for the comparison instructions.
    // the result of the comparison is a logical TRUE or FALSE (i.e., of the data type 'boolean'),
    //
    // if the result is TRUE, the number `1:i64` is pushed onto the stack,
    // and vice versa the number is `0:i64`.
    //
    // example of instruction `lt_i32_u`:
    //
    // ```
    // ;; load 2 numbers on to the stack
    // imm_i32(11)
    // imm_i32(22)
    //
    // ;; now the stack layout is:
    // ;;
    // ;; |    |
    // ;; | 22 |
    // ;; | 11 |
    // ;; \----/
    //
    // ;; check if '11' is less then '22', i.e. `11 < 22 ?`
    // ;; `1:i64` will be pushed on to the stack.
    // lt_i32_u()
    //
    // ;; now the stack layout is:
    // ;;
    // ;; |    |
    // ;; | 1  |
    // ;; \----/
    // ```
    eqz_i32 = 0x02c0, // () (operand number:i32) -> i64
    nez_i32,          // () (operand number:i32) -> i64
    eq_i32,           // () (operand left:i32 right:i32) -> i64
    ne_i32,           // () (operand left:i32 right:i32) -> i64
    lt_i32_s,         // () (operand left:i32 right:i32) -> i64
    lt_i32_u,         // () (operand left:i32 right:i32) -> i64
    gt_i32_s,         // () (operand left:i32 right:i32) -> i64
    gt_i32_u,         // () (operand left:i32 right:i32) -> i64
    le_i32_s,         // () (operand left:i32 right:i32) -> i64, redundant
    le_i32_u,         // () (operand left:i32 right:i32) -> i64, redundant
    ge_i32_s,         // () (operand left:i32 right:i32) -> i64, redundant
    ge_i32_u,         // () (operand left:i32 right:i32) -> i64, redundant

    eqz_i64,  // () (operand number:i64) -> i64
    nez_i64,  // () (operand number:i64) -> i64
    eq_i64,   // () (operand left:i64 right:i64) -> i64
    ne_i64,   // () (operand left:i64 right:i64) -> i64
    lt_i64_s, // () (operand left:i64 right:i64) -> i64
    lt_i64_u, // () (operand left:i64 right:i64) -> i64
    gt_i64_s, // () (operand left:i64 right:i64) -> i64
    gt_i64_u, // () (operand left:i64 right:i64) -> i64
    le_i64_s, // () (operand left:i64 right:i64) -> i64, redundant
    le_i64_u, // () (operand left:i64 right:i64) -> i64, redundant
    ge_i64_s, // () (operand left:i64 right:i64) -> i64, redundant
    ge_i64_u, // () (operand left:i64 right:i64) -> i64, redundant

    eq_f32, // () (operand left:f32 right:f32) -> i64
    ne_f32, // () (operand left:f32 right:f32) -> i64
    lt_f32, // () (operand left:f32 right:f32) -> i64
    gt_f32, // () (operand left:f32 right:f32) -> i64
    le_f32, // () (operand left:f32 right:f32) -> i64
    ge_f32, // () (operand left:f32 right:f32) -> i64
    eq_f64, // () (operand left:f64 right:f64) -> i64
    ne_f64, // () (operand left:f64 right:f64) -> i64
    lt_f64, // () (operand left:f64 right:f64) -> i64
    gt_f64, // () (operand left:f64 right:f64) -> i64
    le_f64, // () (operand left:f64 right:f64) -> i64
    ge_f64, // () (operand left:f64 right:f64) -> i64

    // arithmetic addition
    //
    // wrapping add, e.g. 0xffff_ffff + 2 = 1 (-1 + 2 = 1)
    //
    // () (operand left:i32 right:i32) -> i32
    add_i32 = 0x0300,

    // wrapping sub, e.g. 11 - 211 = -200
    //
    // () (operand left:i32 right:i32) -> i32
    sub_i32,

    // wrapping inc, e.g. 0xffff_ffff inc 2 = 1
    //
    // (param imm:i16) (operand number:i32) -> i32
    add_imm_i32,

    // wrapping dec, e.g. 0x1 dec 2 = 0xffff_ffff
    //
    // (param imm:i16) (operand number:i32) -> i32
    sub_imm_i32,

    // wrapping mul, e.g. 0xf0e0d0c0 * 2 = 0xf0e0d0c0 << 1
    //
    // () (operand left:i32 right:i32) -> i32
    mul_i32,

    div_i32_s, // () (operand left:i32 right:i32) -> i32
    div_i32_u, // () (operand left:i32 right:i32) -> i32
    rem_i32_s, // () (operand left:i32 right:i32) -> i32

    // calculate the remainder
    //
    // () (operand left:i32 right:i32) -> i32
    rem_i32_u,

    // remainder vs modulus
    // --------------------
    //
    // The remainder (%) operator returns the remainder left over when one operand is
    // divided by a second operand. It always takes the sign of the dividend.
    // For the operation n % d, n is called the dividend and d is called the divisor.
    //
    // (13 % 5) = 3
    //  ^    ^
    //  |    |divisor
    //  |dividend <--------- the result always takes the sign of the dividend.
    //
    // (-13 % 5) = -3
    // (4 % 2) = 0
    // (-4 % 2) = -0
    //
    // ref: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Remainder
    //
    // Example of Modulus:
    //
    // 5 % 3 = 2 [here divisible is 5 which is positively signed so the remainder will also be
    // positively signed and the divisor is also positively signed. As both remainder and divisor
    // are of same sign the result will be same as remainder]
    //
    // -5 % 3 = 1 [here divisible is -5 which is negatively signed so the remainder will also be
    // negatively signed and the divisor is positively signed. As both remainder and divisor are
    // of opposite sign the result will be sum of remainder and divisor -2 + 3 = 1]
    //
    // 5 % -3 = -1 [here divisible is 5 which is positively signed so the remainder will also
    // be positively signed and the divisor is negatively signed. As both remainder and divisor
    // are of opposite sign the result will be sum of remainder and divisor 2 + -3 = -1]
    //
    // -5 % -3 = -2 [here divisible is -5 which is negatively signed so the remainder will also
    // be negatively signed and the divisor is also negatively signed. As both remainder and
    // divisor are of same sign the result will be same as remainder]
    //
    // ref: https://stackoverflow.com/questions/13683563/whats-the-difference-between-mod-and-remainder
    // ref: https://en.wikipedia.org/wiki/Euclidean_division
    // ref: https://en.wikipedia.org/wiki/Modulo

    // example of instruction `add_i32`:
    //
    // ```
    // ;; load two numbers onto the stack
    // imm_i32(10)
    // imm_i32(3)
    //
    // ;; subtract one number from the other
    // ;; the top item on the stack will be 7 (10 - 3 = 7)
    // sub_i32()
    // ```

    // example of instruction `rem_i32_u`:
    //
    // ```
    // ;; load two numbers onto the stack
    // imm_i32(10)
    // imm_i32(3)
    //
    // ;; calculate the remainder of dividing one number by the other
    // ;; the top item on the stack will be 1 (10 % 3 = 1)
    // rem_i32_u()
    // ```

    // wrapping add
    //
    // () (operand left:i64 right:i64) -> i64
    add_i64,

    // wrapping sub
    //
    // () (operand left:i64 right:i64) -> i64
    sub_i64,

    // wrapping inc
    //
    // (param imm:i16) (operand number:i64) -> i64
    add_imm_i64,

    // wrapping dec
    //
    // (param imm:i16) (operand number:i64) -> i64
    sub_imm_i64,

    // wrapping mul
    //
    // () (operand left:i64 right:i64) -> i64
    mul_i64,

    div_i64_s, // () (operand left:i64 right:i64) -> i64
    div_i64_u, // () (operand left:i64 right:i64) -> i64
    rem_i64_s, // () (operand left:i64 right:i64) -> i64
    rem_i64_u, // () (operand left:i64 right:i64) -> i64

    add_f32, // () (operand left:f32 right:f32) -> f32
    sub_f32, // () (operand left:f32 right:f32) -> f32
    mul_f32, // () (operand left:f32 right:f32) -> f32
    div_f32, // () (operand left:f32 right:f32) -> f32
    add_f64, // () (operand left:f64 right:f64) -> f64
    sub_f64, // () (operand left:f64 right:f64) -> f64
    mul_f64, // () (operand left:f64 right:f64) -> f64
    div_f64, // () (operand left:f64 right:f64) -> f64

    // bitwise
    //
    // ref:
    // https://en.wikipedia.org/wiki/Bitwise_operation
    and = 0x0340, // bitwise AND     () (operand left:i64 right:i64) -> i64
    or,           // bitwise OR      () (operand left:i64 right:i64) -> i64
    xor,          // bitwise XOR     () (operand left:i64 right:i64) -> i64
    not,          // bitwise NOT     () (operand number:i64) -> i64

    // example of instruction `shift_left_i32`:
    //
    // ```
    // ;; load two numbers onto the stack
    // imm_i32(7)              ;; 00000111
    //
    // ;; perform a bitwise left-shift
    // ;; left shift one spot
    // ;; the top item on the stack will be 14 (00001110)
    // shift_left_i32(1)
    // ```
    shift_left_i32, // left shift                   () (operand number:i32 move_bits:i32) -> i32, move_bits = [0,32)
    shift_right_i32_s, // arithmetic right shift    () (operand number:i32 move_bits:i32) -> i32, move_bits = [0,32)
    shift_right_i32_u, // logical right shift       () (operand number:i32 move_bits:i32) -> i32, move_bits = [0,32)
    rotate_left_i32, // left rotate                 () (operand number:i32 move_bits:i32) -> i32, move_bits = [0,32)
    rotate_right_i32, // right rotate               () (operand number:i32 move_bits:i32) -> i32, move_bits = [0,32)

    // example of instruction `count_leading_zeros_i32`
    //
    // ```
    // ;; load a number onto the stack
    // imm_i32(8_388_608)      ;; 00000000_10000000_00000000_00000000
    //
    // ;; count leading zeros
    // ;; the top item on the stack will be 8
    // count_leading_zeros_i32()
    //
    // example of instruction `count_trailing_zeros_i32`
    //
    // ;; load a number onto the stack
    // imm_i32(8_388_608)      ;; 00000000_10000000_00000000_00000000
    //
    // ;; count trailing zeros
    // ;; the top item on the stack will be 23
    // count_trailing_zeros_i32()
    count_leading_zeros_i32,  // count leading zeros      () (operand number:i32) -> i32
    count_leading_ones_i32,   // count leading ones       () (operand number:i32) -> i32
    count_trailing_zeros_i32, // count trailing zeros     () (operand number:i32) -> i32

    // count the number of ones in the binary representation
    //
    // ;; load a number onto the stack
    // imm_i32(130)            ;; 10000010
    //
    // ;; count the 1s
    // ;; the top item on the stack will be 2
    // count_ones_i32()
    // ```
    //
    // () (operand number:i32) -> i32
    count_ones_i32,

    shift_left_i64, // left shift                   () (operand number:i64 move_bits:i32) -> i64, move_bits = [0,64)
    shift_right_i64_s, // arithmetic right shift    () (operand number:i64 move_bits:i32) -> i64, move_bits = [0,64)
    shift_right_i64_u, // logical right shift       () (operand number:i64 move_bits:i32) -> i64, move_bits = [0,64)
    rotate_left_i64, // left rotate                 () (operand number:i64 move_bits:i32) -> i64, move_bits = [0,64)
    rotate_right_i64, // right rotate               () (operand number:i64 move_bits:i32) -> i64, move_bits = [0,64)
    count_leading_zeros_i64, // () (operand number:i64) -> i32
    count_leading_ones_i64, // () (operand number:i64) -> i32
    count_trailing_zeros_i64, // () (operand number:i64) -> i32
    count_ones_i64,   // () (operand number:i64) -> i32

    // math
    //
    abs_i32 = 0x0380, // () (operand number:i32) -> i32
    neg_i32,          // () (operand number:i32) -> i32
    abs_i64,          // () (operand number:i64) -> i64
    neg_i64,          // () (operand number:i64) -> i64
    abs_f32,          // () (operand number:f32) -> f32
    neg_f32,          // () (operand number:f32) -> f32
    copysign_f32,     // () (operand num:f32 sign:f32) -> f32

    // sqrt(x)
    // () (operand number:f32) -> f32
    sqrt_f32,

    min_f32,                       // () (operand left:f32 right:f32) -> f32
    max_f32,                       // () (operand left:f32 right:f32) -> f32
    ceil_f32,                      // () (operand number:f32) -> f32
    floor_f32,                     // () (operand number:f32) -> f32
    round_half_away_from_zero_f32, // () (operand number:f32) -> f32
    round_half_to_even_f32,        // () (operand number:f32) -> f32

    // the integer part of x
    // () (operand number:f32) -> f32
    trunc_f32,

    // the fractional part of  x
    // () (operand number:f32) -> f32
    fract_f32,

    // cbrt(x), the cube root of x
    // () (operand number:f32) -> f32
    cbrt_f32,

    // e^x
    // () (operand number:f32) -> f32
    exp_f32,

    // 2^x
    // () (operand number:f32) -> f32
    exp2_f32,

    // log_e(x)
    // () (operand number:f32) -> f32
    ln_f32,

    // log_2(x)
    // () (operand number:f32) -> f32
    log2_f32,

    // log_10(x)
    // () (operand number:f32) -> f32
    log10_f32,

    sin_f32,  // () (operand number:f32) -> f32
    cos_f32,  // () (operand number:f32) -> f32
    tan_f32,  // () (operand number:f32) -> f32
    asin_f32, // () (operand number:f32) -> f32
    acos_f32, // () (operand number:f32) -> f32
    atan_f32, // () (operand number:f32) -> f32

    // base ^ exponent
    // () (operand base:f32 exponent:f32) -> f32
    pow_f32,

    // log _ base(number)
    // () (operand number:f32 base:f32) -> f32
    log_f32,

    // examples of 'round_half_away_from_zero':
    // round(2.4) = 2.0
    // round(2.6) = 3.0
    // round(2.5) = 3.0
    // round(-2.5) = -3.0
    //
    // ref:
    // https://en.wikipedia.org/wiki/Rounding#Rounding_half_away_from_zero
    abs_f64,                       // () (operand number:f64) -> f64
    neg_f64,                       // () (operand number:f64) -> f64
    copysign_f64,                  // () (operand num:f64 sign:f64) -> f64
    sqrt_f64,                      // () (operand number:f64) -> f64
    min_f64,                       // () (operand left:f64 right:f64) -> f64
    max_f64,                       // () (operand left:f64 right:f64) -> f64
    ceil_f64,                      // () (operand number:f64) -> f64
    floor_f64,                     // () (operand number:f64) -> f64
    round_half_away_from_zero_f64, // () (operand number:f64) -> f64
    round_half_to_even_f64,        // () (operand number:f64) -> f64
    trunc_f64,                     // () (operand number:f64) -> f64
    fract_f64,                     // () (operand number:f64) -> f64
    cbrt_f64,                      // () (operand number:f64) -> f64
    exp_f64,                       // () (operand number:f64) -> f64
    exp2_f64,                      // () (operand number:f64) -> f64
    ln_f64,                        // () (operand number:f64) -> f64
    log2_f64,                      // () (operand number:f64) -> f64
    log10_f64,                     // () (operand number:f64) -> f64
    sin_f64,                       // () (operand number:f64) -> f64
    cos_f64,                       // () (operand number:f64) -> f64
    tan_f64,                       // () (operand number:f64) -> f64
    asin_f64,                      // () (operand number:f64) -> f64
    acos_f64,                      // () (operand number:f64) -> f64
    atan_f64,                      // () (operand number:f64) -> f64

    // base ^ exponent
    // () (operand base:f64 exponent:f64) -> f64
    pow_f64,

    // log _ base(number)
    // () (operand number:f64 base:f64) -> f64
    log_f64,

    // control flow "end"
    //
    // when the instruction 'end' is executed, a stack frame is removed and
    // the results of the current block or function are placed at the top of the stack.
    //
    // ()->()
    end = 0x03c0,

    // create a block scope.
    //
    // a block is similar to a function, it also has
    // parameters, results and type (it shares the type list with functions).
    //
    // when this instruction is called, a stack frame is created which
    // calls 'block frame', 'block frame' is similar to 'function frame'.
    //
    // note that this instruction is different from the WebAssembly 'block' instruction,
    // its parameters are not 'local variables' and the values are placed on the
    // operand stack, they cannot be accessed with 'local_load/local_store' instructions.
    //
    // (param type_index:i32, local_list_index:i32)
    block,

    // the 'break' instruction is similar to the 'end' instruction, it is
    // used to end a block or a function.
    //
    // - for a block:
    //   removes a block stack frame and jumps to the NEXT instruction
    //   that AFTER the 'end' instruction. the value of the parameter
    //   'next_inst_offset' should be `addr of next inst after 'end'` - `addr of break`
    //
    // - for a function:
    //   the value of the parameter 'next_inst_offset' is ignored.
    //   a function stack frame will be removed and returned to the the
    //   instruction NEXT to the instruction 'call'.
    //
    // note:
    // - the purpose of parameter 'next_inst_offset' is to simplify the execution
    //   of instructions. in face, the value can be set to 0 for all
    //   instructions, and then calculated by the VM at runtime. however,
    //   the current VM implementation still relies on assembler calculations.
    // - the 'end' and 'break' instructions are the same, except that
    //   the 'break' instruction can specify the 'reversed_index'
    //   and 'next_inst_offset'.
    //   so `end` == `break reversed_index=0 next_inst_offset=2`
    //
    //
    // e.g.
    //
    // ```bytecode
    // 0d0000 block(0)          ;; the size of instruction 'block' is 8 bytes
    // 0d0008   nop             ;;
    // 0d0010   break(0,14)     ;; the size of instruction 'break' is 8 bytes, (14 = 24 - 10) ---\
    // 0d0018   nop             ;;                                                               |
    // 0d0020   nop             ;;                                                               |
    // 0d0022 end               ;;                                                               |
    // 0d0024 nop               ;; <-- jump to here (the instruction that next to the 'end') <---/
    // ```
    //
    // instruction 'break' not only just finish a block or a function, but also
    // brings the operands out of the block or function, e.g.
    //
    // 0d0000 block(0)          ;; assumes the block type is '()->(i32,i32)'
    // 0d0008   imm_i32(11)     ;;
    // 0d0016   imm_i32(13)     ;;                 | 17                 | -----\ operands '17' and '13' were
    // 0d0024   imm_i32(17)     ;; --------------> | 13                 | -----| taken out of the block frame
    // 0d0032   break(0,14)     ;; ---\            | 11                 |      |
    // 0d0040   nop             ;;    |            | [block frame info] |      v
    // 0d0042   nop             ;;    | jump       | ..                 |    | 17                 |
    // 0d0044   nop             ;;    |            | [func frame info]  |    | 13                 |
    // 0d0046   nop             ;;    |            \____________________/    | ..                 |
    // 0d0048 end               ;;    v               the stack layout       | [func frame info]  |
    // 0d0050 nop               ;; <---------------------------------------- \____________________/
    //                                                                        the stack layout
    //
    // the instruction 'break' can cross over multiple block layers.
    // when the parameter 'reversed_index' is 0, it simply end the current block.
    // when the value is greater than 0, multiple block stack frames will be removed,
    // as well as the operands will be taken out of the block.
    // the amount of the operands is determined by the type of the target block.
    //
    // ```bytecode
    // 0d0000 block 0           ;; assumes the block type is '()->(i32,i32,i32)'
    // 0d0008   block 0         ;; assumes the block type is '()->(i32,i32)'
    // 0d0016     block 0       ;; assumes the block type is '()->(i32)'
    // 0d0024       nop         ;;
    // 0d0026       break(1,14) ;; (18 = 44 - 26) --------\
    // 0d0034       nop         ;;                        |
    // 0d0036     end           ;;                        |
    // 0d0038     nop           ;;                        |
    // 0d0040   end             ;;                        | carries operands (i32, i32) and
    // 0d0042   nop             ;; <----------------------/ jumps to here
    // 0d0044 end
    // ```
    //
    // background:
    //
    // there is a similar instruction in WASM called 'br/break', it is used
    // to make the PC jump to the address of the 'end' instruction.
    // it is more elegant than the XiaoXuan Core instruction 'break',
    // but less efficient because it has to carry the 'return values (operands)' twice.
    // to balance performance and elegance, the XiaoXuan Core instruction
    // 'break' implies 'end' as well as jumps directly to the next instruction
    // after the 'end' instruction.
    //
    // (param reversed_index:i16, next_inst_offset:i32)
    break_,

    // the 'recur' instruction allows the VM to jump to the instruction next to the
    // instruction 'block...' or the first instruction of the current function,
    // and all the operands in the current stack frame are removed, except
    // the operands for the 'parameters of the target block or function' are reserved and
    // placed at the top of the stack.
    //
    // it is commonly used to construct the 'loop/for' structures in general programming languages,
    // it is also used to implement the TCO (Tail Call Optimization).
    //
    // if the target frame is the function frame itself, the 'start_inst_offset' parameter is ignored
    // and all local variables are reset to 0 (except the arguments).
    //
    // note that the value of 'start_inst_offset' is a positive number.
    //
    // 0d0000 block(0)          ;; assumes the block type is '()->(i32,i32)'
    // 0d0008   imm_i32(11)     ;; <------\
    // 0d0016   imm_i32(13)     ;;        |         | 17                 | -----\ operands '17' and '13' were
    // 0d0024   imm_i32(17)     ;; ---------------> | 13                 | -----| taken out of the block frame
    // 0d0032   nop             ;;        |         | 11                 |      |
    // 0d0034   nop             ;;        |         | [block frame info] |      v
    // 0d0036   nop             ;;        |         | ..                 |    | 17                 |
    // 0d0038   nop             ;;  jump  |         | [func frame info]  |    | 13                 |
    // 0d0040   recur(0,14)     ;; -------/         \____________________/    | [block frame info] |
    // 0d0048 end               ;;        |            the stack layout       | ..                 |
    // 0d0050 nop               ;;        \-------<-------------------------- | [func frame info]  |
    //                                                                        \____________________/
    //                                                                           the stack layout
    //
    // the 'recur' instruction can cross over multiple block layers also.
    //
    // ```bytecode
    // 0d0000 block(0)          ;; assumes the block type is '()->(i32,i32)'
    // 0d0008   nop             ;; <--------------------------------------------\ carries operands (i32,i32) and
    // 0d0010   block(0)        ;; assumes the block type is '()->(i32)'        | jumps to here
    // 0d0018     nop           ;;                                              |
    // 0d0020     recur(1,12)   ;; (12 = 20 - 8) ---------------->--------------/
    // 0d0028     nop
    // 0d0030   end
    // 0d0032 end
    // ```
    //
    // 'start_inst_offset' is the address of the next instruction after 'block'.
    //
    // (param reversed_index:i16, start_inst_offset:i32)
    recur,

    // the instruction 'block_alt' is similar to the 'block', it also creates a new block scope
    // and a block stack frame. but it jumps to the NEXT instruction that AFTER the 'break_alt'
    // instruction if the operand on the top of stack is equals to ZERO (logic FALSE).
    //
    // note:
    // 0:i32 and 0:i64 are both treated as logical FALSE and all other
    // i32/i64 non-zero are treated as logical TRUE.
    //
    // e.g.
    //
    // ```c
    // if (i != 0) {
    //     /* the 'then' part */
    // } else {
    //     /* the 'else' part */
    // }
    // ```
    //
    // ```bytecode
    //                          ;; TRUE             | FALSE
    //                          ;; |                | |
    //                          ;; |                | \-->--\
    // 0d0000 block_alt(0,158)  ;; V                |       | jump to 0d0158 when FALSE
    // 0d0008 ...               ;; |+               |       |-
    // ;; the 'then' part       ;; |+               |       |-
    // 0d0150 break_alt(200)    ;; \-->--\+         |       |-
    // 0d0158 ...               ;;       |-         | /--<--/+
    // ;; the 'else' part       ;;       |-         | |+
    // 0d0350 end               ;;       |-         | |+
    // 0d0352 nop               ;; <-----/          | |
    // ```
    //
    // (+ => execute, - => pass)
    //
    // the 'block_alt' instruction is mainly used to construct 'if' control flow
    // structures, and since there are branches within its scope, it should not
    // have input parameters as well as local variables, although it should have
    // return values. therefore this instruction has parameter 'type_index' and
    // not 'local_list_index'.
    //
    // note that the stack frame created by this instructon still has the local
    // varialbe area (it is just empty) even though no 'local_list_index' is specified.
    //
    // // (param type_index:i32, local_list_index:i32, next_inst_offset:i32)
    // (param type_index:i32, next_inst_offset:i32)
    block_alt,

    // an instruction to jump out of the current 'block_alt' scope.
    //
    // it can only exist within the scope of the 'block_alt'.
    // it is equivalent to 'break 0, next_inst_offset'.
    // (param next_inst_offset:i32)
    break_alt,

    // create a block scope only if the operand on the top of stack is
    // NOT equals to ZERO (logic TRUE).
    //
    // the value of 'next_inst_offset' should be the address of the next instruction
    // AFTER the instruction 'end'.
    //
    // instruction 'block_nez' is commonly used to construct the 'if' structures
    // in general programming languages.
    //
    // e.g
    //
    // ```c
    // if (i != 0) {
    //     ...
    // }
    // ```
    //
    // ```bytecode
    // 0d0000 block_nez(0,100)  ;; -----\
    // ....                     ;;      |
    // 0d0100 end               ;;      |
    // 0d0102 nop               ;; <----/ jump to here when FALSE
    // ```
    //
    // unlike the 'block' instruction, although the instruction 'block_nez' also
    // create a block, it should not have parameters and return values (i.e., the
    // type is `()->()`), so the instruction has no 'type_index' parameter.
    // However, the instruction still supports local variables, so there is a
    // parameter 'local_list_index'.
    //
    // (param local_list_index:i32, next_inst_offset:i32)
    block_nez,

    // the 'break_nez' and 'recur_nez' instructions are used to optimize the 'break'
    // and 'recur' with conditions.
    //
    // example of 'break_nez':
    //
    // ```rust
    // let i = loop {
    //   ...
    //   if ... break 100;
    //   ...
    // }
    // ```
    //
    // the unoptimized bytecode is:
    //
    // ```bytecode
    // 0d0000 block 0
    // 0d0008   ...             ;; <-------------\
    //          ...             ;;               |
    // 0d0100   block_nez(0,28) ;; ----\         |
    // 0d0112     imm_i32(100)  ;;     |         |
    // 0d0120     break(1,88)   ;; ----|----\    |
    // 0d0128   end             ;; <---/    |    |
    //          ...             ;;          |    |
    // 0d0200   recur(0,192)    ;; ---------|----/
    // 0d0208 end               ;;          |
    // 0d0210 ...               ;; <--------/
    // ```
    //
    // optimized with instruction 'break_nez':
    //
    // ```bytecode
    // 0d0000 block(0)
    // 0d0008   ...             ;; <-------------\
    //          ...             ;;               |
    // 0d0100   imm_i32(100)    ;;               |
    //          ...             ;;               |
    // 0d0120   break_nez(0,88) ;; ---------\    |
    // 0d0128   local_store..   ;; drop 100 |    |
    //          ...             ;;          |    |
    // 0d0200   recur(0,192)    ;; ---------|----/
    // 0d0208 end               ;;          |
    // 0d0210 ...               ;; <--------/
    // ```
    //
    // instruction 'recur_nez' is commonly used to implement the TCO (Tail Call Optimization).
    //
    // consider the following function:
    //
    // ```rust
    // /* calculate '3+2+1', the result should be '6' */
    //
    // let s = accumulate(0, 3);
    //
    // fn accumulate(sum: i32, number: i32) -> i32 {    // /-----\
    //     let new_sum = sum + number;                  // | +   |
    //     if number == 0 {                             // | --> | branch then
    //         new_sum                                  // | <-- | return case 0
    //     } else {                                     // | --> | branch else
    //         accumulate(new_sum, number - 1)          // | <-- | return case 1
    //     }                                            // |     |
    // }                                                // \-----/
    // ```
    //
    // when invoke function call 'accumulate(0, 3)', the calling path is as follows:
    //
    // (0,3)--\
    //        |    /-----\     (3,2)  /-----\     (5,1)  /-----\     (6,0)  /-----\
    //        \--> | +   |   /------> | +   |   /------> | +   |   /------> | +   |
    //             |=====|   |        |=====|   |        |=====|   |        |=====|
    //             | --> |   |        | --> |   |        | --> |   |        | --> | -----\
    //             | <-- |   |        | <-- |   |        | <-- |   |  /---- | <-- | <----/
    //             |=====|   |        |=====|   |        |=====|   |  |     |=====|
    //             | --> | --/        | --> | --/        | --> | --/  |     | --> |
    //        /--- | <-- | <--------- | <-- | <--------- | <-- | <----/     | <-- |
    //        |    \-----/            \-----/            \-----/            \-----/
    //   6 <--/
    //
    //
    // the function 'accumulate' is called 4 times and 4 stack frames are created.
    // since there is no other operation after the statement 'accumulate(new_sum, number - 1)',
    // and only one value is returned afterwards,the call path can be optimized as follows:
    //
    // (0,3)--\
    //        |    /-----\     (3,2)  /-----\     (5,1)  /-----\     (6,0)  /-----\
    //        \--> | +   |   /------> | +   |   /------> | +   |   /------> | +   |
    //             |=====|   |        |=====|   |        |=====|   |        |=====|
    //             | --> |   |        | --> |   |        | --> |   |        | --> | ---\
    //             | <-- |   |        | <-- |   |        | <-- |   |    /-- | <-- | <--/
    //             |=====|   |        |=====|   |        |=====|   |    |   |=====|
    //             | --> | --/        | --> | --/        | --> | --/    |   | --> |
    //             | <-- |            | <-- |            | <-- |        |   | <-- |
    //             \-----/            \-----/            \-----/        |   \-----/
    //   6 <------------------------------------------------------------/
    //
    // the instruction 'recur_nez' can be used to further optimized the call path:
    //
    // (0,3)--\
    //        |    /-----\
    //        \--> | +   | <----\  <----\  <----\
    //             |=====|      |       |       |
    //             | --> | --\  |       |       |
    //        /--- | <-- | <-/  |       |       |
    //        |    |=====|      |       |       |
    //        |    | --> | -----/  -----/  -----/
    //        |    |     | (3,2)  (5,1)  (6,0)
    //        |    \-----/
    //   6 <--/
    //
    // what is shown above is TCO (tail call optimization), this optimization saves us from
    // creating and destroying call stack frames multiple times, which saves resources
    // to improve program efficiency.
    //
    // an important prerequisite for TCO is that the 'self call' statement
    // (in the genernal programming language) must be the last statement of the function,
    // or the last statement in the branch, otherwise the logical error will occur. e.g.
    //
    // ```rust
    // fn factorial(number: i32) -> i32 {
    //     if number == 0 {
    //         1
    //     } else {
    //         number * factorial(number - 1)
    //     }
    // }
    // ```
    //
    // expanding the statement 'number * factorial(number - 1)':
    //
    // ```rust
    // let i = number;
    // let j = factorial(number - 1);
    // i * j
    // ```
    //
    // obviously the function call statement 'factorial(number - 1)' is neither the last statement
    // of the function nor the last statement of the branch. the last statement
    // is 'i * j', so this code can not apply TCO.
    //
    // of course we can modify this function so that it can apply TCO, e.g.
    //
    // ```rust
    // /* calculate '5*4*3*2*1', the result should be '120' */
    //
    // let s = factorial_tco(1, 5);
    //
    // fn factorial_tco(sum: i32, number: i32) -> i32 {
    //     if number == 0 {
    //         sum
    //     } else {
    //         let new_sum = sum * number;
    //         factorial_tco(new_sum, number - 1)
    //     }
    // }
    // ```
    //
    // (param reversed_index:i16, next_inst_offset:i32)
    break_nez,
    //
    // (param reversed_index:i16, start_inst_offset:i32)
    recur_nez,

    // control flow structures and instructions
    // ----------------------------------------
    //
    // ## branch
    //
    // | structure         | assembly          | instruction(s)     |
    // |-------------------|-------------------|--------------------|
    // |                   |                   | ..a..              |
    // | if ..a.. {        | (when (a)         | block_nez -\       |
    // |    ..b..          |       (b)         |   ..b..    |       |
    // | }                 | )                 | end        |       |
    // |                   |                   | ...    <---/       |
    // |-------------------|-------------------|--------------------|
    // |                   |                   | ..a..              |
    // | if ..a.. {        | (if (a)           | block_alt ---\     |
    // |    ..b..          |     (b)           |   ..b..      |     |
    // | } else {          |     (c)           |   break_alt -|-\   |
    // |    ..c..          | )                 |   ..c..  <---/ |   |
    // | }                 |                   | end            |   |
    // |                   |                   | ...      <-----/   |
    // |-------------------|-------------------|--------------------|
    // |                   |                   | ..a..              |
    // | if ..a.. {        | (if (a)           | block_alt ---\     |
    // |    ..b..          |     (b)           |   ..b..      |     |
    // | } else if ..c.. { |     (if (c)       |   break_alt--|---\ |
    // |    ..d..          |         (d)       |   ..c..  <---/   | |
    // | } else {          |         (e)       |   block_alt --\  | |
    // |    ..e..          |     )             |     ..d..     |  | |
    // | }                 | )                 |     break_alt-|-\| |
    // |                   |                   |     ..e..  <--/ || |
    // |                   |                   |   end           || |
    // |                   |                   | end        <----/| |
    // |                   |                   | ...        <-----/ |
    // |                   |                   |                    |
    // |                   | ----------------- | ------------------ |
    // |                   |                   |                    |
    // |                   | (branch           | block              |
    // |                   |   (case (a) (b))  |   ..a..            |
    // |                   |   (case (c) (d))  |   block_nez -\     |
    // |                   |   (default (e))   |     ..b..    |     |
    // |                   | )                 |     break 1 -|--\  |
    // |                   |                   |   end        |  |  |
    // |                   |                   |   ..c..  <---/  |  |
    // |                   |                   |   block_nez -\  |  |
    // |                   |                   |     ..d..    |  |  |
    // |                   |                   |     break 1 -|--|  |
    // |                   |                   |   end        |  |  |
    // |                   |                   |   ..e..  <---/  |  |
    // |                   |                   | end             |  |
    // |                   |                   | ...        <----/  |
    // |-------------------|-------------------|--------------------|
    //
    // ## loop
    //
    // | structure         | assembly          | instructions(s)    |
    // |-------------------|-------------------|--------------------|
    // | loop {            | (for              | block              |
    // |    ...            |   ...             |   ...   <--\       |
    // | }                 |   (recur ...)     |   recur 0 -/       |
    // |                   | ))                | end                |
    // |-------------------|-------------------|--------------------|
    // | while ..a.. {     |                   | block              |
    // |    ...            |                   |   ..a..   <----\   |
    // | }                 |                   |   break_nez 0 -|-\ |
    // |                   |                   |   ...          | | |
    // | for {...}         |                   |   recur 0 -----/ | |
    // |                   |                   | end              | |
    // |                   |                   | ...        <-----/ |
    // |                   |                   |                    |
    // |-------------------| ----------------- | ------------------ |
    // |                   |                   |                    |
    // |                   | (for              | block              |
    // |                   |   (when (a)       |   ..a..    <---\   |
    // |                   |     ( ...         |   block_nez    |   |
    // |                   |       (recur ...) |     ...        |   |
    // |                   |     )             |     recur 1 ---/   |
    // |                   |   )               |   end              |
    // |                   | ))                | end                |
    // |                   |                   |                    |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    // | do {              |                   | block              |
    // |    ...            |                   |   ...      <---\   |
    // | }while(..a..)     |                   |   ..a..        |   |
    // |                   |                   |   recur_nez 0 -/   |
    // |                   |                   | end                |
    // |                   |                   |                    |
    // |                   | ----------------- | ------------------ |
    // |                   |                   |                    |
    // |                   | (for              | block              |
    // |                   |   ...             |   ...      <---\   |
    // |                   |   (when (a)       |   ..a..        |   |
    // |                   |     (recur ...)   |   block_nez    |   |
    // |                   |   )               |     recur 1 ---/   |
    // |                   | ))                |   end              |
    // |                   |                   | end                |
    // |                   |                   |                    |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    //
    //
    // ## TCO
    //
    // | structure         | assembly          | instructions(s)    |
    // |-------------------|-------------------|--------------------|
    // | func foo {        | (function         | -- func begin --   |
    // |    ...            |   ( ...           |   ...   <-------\  |
    // |    if ..a.. {     |     (when (a)     |   ..a..         |  |
    // |      foo()        |       (selfcall.) |   block_nez --\ |  |
    // |    }              |     )             |     recur 1 --|-/  |
    // | }                 |   )               |   end         |    |
    // |                   | )                 | end      <----/    |
    // |                   |                   |                    |
    // |                   | ----------------- | ------------------ |
    // |                   |                   |                    |
    // |                   |                   | -- func begin --   |
    // |                   |                   |   ...   <-------\  |
    // |                   |                   |   ..a..         |  | // note:
    // |                   |                   |   recur_nez 0 --/  | // optimized with 'recur_nez'
    // |                   |                   | end                |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    // | func foo {        | (function         | -- func begin --   |
    // |    if ..a.. {     |   (if (a)         |   ..a.. <------\   |
    // |       ..b..       |     (b)           |   block_alt -\ |   |
    // |    } else {       |     ( ...         |     ..b..    | |   |
    // |       ..c..       |       ..c..       |   break_alt -|-|-\ |
    // |       foo()       |       (selfcall.) |     ..c.. <--/ | | |
    // |    }              |     )             |     recur 1 ---/ | |
    // | }                 |   )               |   end            | |
    // |                   | ))                | end         <----/ |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    //
    //
    // ## break
    //
    // | structure         | assembly          | instructions(s)    |
    // |-------------------|-------------------|--------------------|
    // |                   |                   |                    |
    // | loop {            | (for              | block              |
    // |    ...            |    ...            |   ...        <---\ |
    // |    if ..a.. {     |    (when (a)      |   ..a..          | |
    // |      break;       |                   |   block_nez      | |
    // |    }              |       (break ...) |     break 1 ---\ | |
    // |    ...            |    )              |   end          | | |
    // | }                 |    ...            |   ...          | | |
    // |                   |    (recur ...)    |   recur 0  ----|-/ |
    // |                   | ))                | end            |   |
    // |                   |                   | ...      <-----/   |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    // | func foo {        | (function         | -- func begin --   |
    // |    ...            |   ...             |   ...              |
    // |    if ..a.. {     |   (when (a)       |   ..a..            |
    // |      return ...;  |                   |   block_nez        |
    // |    }              |       (return ..) |     break 1  ---\  |
    // |    ...            |   )               |   end           |  |
    // | }                 |   ...             |   ...           |  |
    // |                   | ))                | end         <---/  |
    // |                   |                   |                    |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|

    // general function call
    //
    // (param function_public_index:i32) (operand args...) -> (...)
    call = 0x0400,

    // function_public_index
    // ---------------------
    //
    // `function_public_index` is mixed with
    // imported functions and internal functions, its value is equivalent:
    // `function_public_index` == "the amount of imported functions" + "function internal index"

    // dynamic function call
    //
    // calling a function that is specified at runtime.
    //
    // the "anonymous functions" (closure functions) of the XiaoXuan Core language are implemented
    // by "dynamic function call". when passing a regular or anonymous function to another
    // function as a parameter, it actually passes a pointer to a struct 'closure_function_item':
    //
    // closure_function_item {
    //     function_public_index: i32
    //     captured_data_pointer: i64
    // }
    //
    // "captured_data" is a dynamically-created structure that contains the data
    // captured by the function, for example, if an anonymous function captured an i32 number
    // and a string, then the "captured data" struct would be as follows:
    //
    // captured_data_1: {
    //     value_0: i32
    //     value_1: i64
    // }
    //
    // the target function can be 'anonymous functions' or 'regular functions',
    // if the target is an anonymous function, an additional parameter is automatically appended
    // by the compiler, e.g.
    //
    // `let a = fn (a:i32, b:i32) {...}`
    //
    // will be compiled as:
    //
    // `let a = fn (a:i32, b:i32, captured_data_pointer:i64) {...}`
    //
    // the following is a demo of passing this anonymous function to the function "filter":
    //
    // ```text
    //                              /--> function_public_index --> fn (a, b, captured_data_pointer) {...}
    //                         /--->|
    //                         |    \--> captured_data_pointer -> captured_data_1
    //                         |
    // let a = filter(list, predicate)
    // ```
    //
    // if the target is a regular function, the compiler also generates the "closure_function_item"
    // structure, as well as a wrapper function, e.g.
    //
    // ```text
    //                              /--> function_public_index --> fn wrapper (a, b, captured_data_pointer) --> fn original (a, b)
    //                         /--->|
    //                         |    \--> captured_data_pointer -> 0
    //                         |
    // let a = filter(list, predicate)
    // ```
    //
    // () (operand function_public_index:i32, args...) -> (...)
    dyncall,

    // environment function call
    //
    // call VM built-in functions, such as getting environment variables,
    // runtime information, creating threads, etc.
    //
    // (param envcall_num:i32) (operand args...) -> (...)
    envcall,

    // call 'syscall' directly
    //
    // the syscall arguments should be pushed on the stack first,
    // followed by the syscall number, e.g.
    //
    // | params_count   | <-- stack end
    // | syscall_num    |
    // | arg6           |
    // | arg5           |
    // | arg4           |
    // | arg3           |
    // | arg2           |                  | error no       |
    // | arg1           |     return -->   | return value   |
    // | ...            |                  | ...            |
    // \----------------/ <-- stack start  \----------------/
    //
    // when a syscall completes, the return value is stored in the 'rax' register,
    // if the operation fails, the value is a negative value (rax < 0).
    //
    // note that unlike the C standard library, there is no 'errno' when
    // calling syscall directly from the assembly.
    //
    // () (operand args..., syscall_num:i32, params_count: i32) -> (return_value:i64, error_no:i32)
    syscall,

    // external function call
    //
    // note that both the 'syscall' and 'extcall' instructions are optional
    // and may not be available in some environments.
    // the list of VM supported features can be obtained using the
    // instruction 'envcall' with call number 'runtime_features'.
    //
    // (param external_function_index:i32) (operand args...)-> void/i32/i64/f32/f64
    extcall,

    // put the function public index to stack
    //
    // (param function_public_index:i32) -> i32
    pub_index_function,

    // terminate VM
    //
    // (param reason_code:u32) -> ()
    panic = 0x0440,

    // get the memory address of VM data
    //
    // it is not safe to access VM data using the host-side memory addresses,
    // but these addresses are necessary to talk to external functions.
    //
    // note that the host-side address of local variables is only valid in the
    // scope of the current function and its sub-functions.
    // when a function exits, the function stack frame
    // is destroyed (or modified), as are the local variables.
    //
    // |--------------------|--------------|------------------|-----------------|
    // |                    | by indice    | by VM mem alloc  | by host address |
    // |--------------------|--------------|------------------|-----------------|
    // | local variables    | safe         | N/A              | unsafe          |
    // |--------------------|--------------|------------------|-----------------|
    // | read-only data     |              |                  |                 |
    // | read-write data    | safe         | N/A              | not recommended |
    // | uninitilized data  |              |                  |                 |
    // |--------------------|--------------|------------------|-----------------|
    // | heap               | N/A          | safe             | not recommended |
    // |--------------------|--------------|------------------|-----------------|
    //
    //
    host_addr_local, // (param reversed_index:i16 offset_bytes:i16 local_variable_index:i16) -> i64
    host_addr_local_extend, // (param reversed_index:i16 local_variable_index:i16) (operand offset_bytes:i64) -> i64
    host_addr_data,         // (param offset_bytes:i16 data_public_index:i32) -> i64
    host_addr_data_extend,  // (param data_public_index:i32) (operand offset_bytes:i64) -> i64
    host_addr_heap,         // (param offset_bytes:i16) (operand heap_addr:i64) -> i64

    // this instruction is used to create a pointer to a callback function
    // for an external C function.
    //
    // note:
    // - a "bridge callback function" (it's a host side function) will be created when
    //   this instruction is executed,
    // - the specified VM function will be appended to the
    //   "bridge callback function table" to prevent duplicate creation.
    // - the body of "bridge callback function" is created via JIT codegen.
    //
    // (param function_public_index:i32) -> i64/i32
    host_addr_function,

    // copy data from VM heap to host memory
    //
    // () (operand dst_pointer:i64 src_addr:i64 count:i64) -> ()
    host_copy_heap_to_memory,

    // copy data from host memory to VM heap
    //
    // () (operand dst_addr:i64 src_pointer:i64 count:i64) -> ()
    host_copy_memory_to_heap,

    // copy data between host memory
    // this is a rather strange instruction because it operates on
    // the host's memory (i.e., memory external to the VM).
    // This instruction is used to speed up collaboration between
    // external functions.
    //
    // () (operand dst_pointer:i64 src_pointer:i64 count:i64) -> ()
    host_memory_copy,

    // OTHER OPCODES:
    //
    // (addr, value) -> old_value
    //
    // atomic_rmw_add_i32 = 0xd00,
    // atomic_rmw_sub_i32,
    // atomic_rmw_and_i32,
    // atomic_rmw_or_i32,
    // atomic_rmw_xor_i32,
    // atomic_rmw_exchange_i32,
    //
    // (addr, expect_value, new_value) -> old_value
    // atomic_cas_i32,
    //
    // (addr, value) -> old_value
    //
    // atomic_rmw_add_i64,
    // atomic_rmw_sub_i64,
    // atomic_rmw_and_i64,
    // atomic_rmw_or_i64,
    // atomic_rmw_xor_i64,
    // atomic_rmw_exchange_i64,
    //
    // (addr, expect_value, new_value) -> old_value
    // atomic_cas_i64,
    //
    // SIMD/Vectorization
    //
    // ref:
    // - https://github.com/rust-lang/portable-simd
    // - https://doc.rust-lang.org/std/simd/index.html
    // - https://github.com/rust-lang/packed_simd
    // - https://github.com/rust-lang/rfcs/blob/master/text/2325-stable-simd.md
}

impl Opcode {
    pub fn get_name(&self) -> &'static str {
        match self {
            Opcode::nop => "nop",
            //
            Opcode::imm_i32 => "imm_i32",
            Opcode::imm_i64 => "imm_i64",
            Opcode::imm_f32 => "imm_f32",
            Opcode::imm_f64 => "imm_f64",
            //
            Opcode::data_load_i64 => "data_load_i64",
            Opcode::data_load_i32_s => "data_load_i32_s",
            Opcode::data_load_i32_u => "data_load_i32_u",
            Opcode::data_load_i16_s => "data_load_i16_s",
            Opcode::data_load_i16_u => "data_load_i16_u",
            Opcode::data_load_i8_s => "data_load_i8_s",
            Opcode::data_load_i8_u => "data_load_i8_u",
            Opcode::data_load_f64 => "data_load_f64",
            Opcode::data_load_f32 => "data_load_f32",
            Opcode::data_store_i64 => "data_store_i64",
            Opcode::data_store_i32 => "data_store_i32",
            Opcode::data_store_i16 => "data_store_i16",
            Opcode::data_store_i8 => "data_store_i8",
            Opcode::data_store_f64 => "data_store_f64",
            Opcode::data_store_f32 => "data_store_f32",
            //
            Opcode::data_load_extend_i64 => "data_load_extend_i64",
            Opcode::data_load_extend_i32_s => "data_load_extend_i32_s",
            Opcode::data_load_extend_i32_u => "data_load_extend_i32_u",
            Opcode::data_load_extend_i16_s => "data_load_extend_i16_s",
            Opcode::data_load_extend_i16_u => "data_load_extend_i16_u",
            Opcode::data_load_extend_i8_s => "data_load_extend_i8_s",
            Opcode::data_load_extend_i8_u => "data_load_extend_i8_u",
            Opcode::data_load_extend_f64 => "data_load_extend_f64",
            Opcode::data_load_extend_f32 => "data_load_extend_f32",
            //
            Opcode::data_store_extend_i64 => "data_store_extend_i64",
            Opcode::data_store_extend_i32 => "data_store_extend_i32",
            Opcode::data_store_extend_i16 => "data_store_extend_i16",
            Opcode::data_store_extend_i8 => "data_store_extend_i8",
            Opcode::data_store_extend_f64 => "data_store_extend_f64",
            Opcode::data_store_extend_f32 => "data_store_extend_f32",
            //
            Opcode::local_load_i64 => "local_load_64",
            Opcode::local_load_i32_s => "local_load_i32_s",
            Opcode::local_load_i32_u => "local_load_i32_u",
            Opcode::local_load_i16_s => "local_load_i16_s",
            Opcode::local_load_i16_u => "local_load_i16_u",
            Opcode::local_load_i8_s => "local_load_i8_s",
            Opcode::local_load_i8_u => "local_load_i8_u",
            Opcode::local_load_f64 => "local_load_f64",
            Opcode::local_load_f32 => "local_load_f32",
            Opcode::local_store_i64 => "local_store_i64",
            Opcode::local_store_i32 => "local_store_i32",
            Opcode::local_store_i16 => "local_store_i16",
            Opcode::local_store_i8 => "local_store_i8",
            Opcode::local_store_f64 => "local_store_f64",
            Opcode::local_store_f32 => "local_store_f32",
            //
            Opcode::local_load_extend_i64 => "local_load_extend_i64",
            Opcode::local_load_extend_i32_s => "local_load_extend_i32_s",
            Opcode::local_load_extend_i32_u => "local_load_extend_i32_u",
            Opcode::local_load_extend_i16_s => "local_load_extend_i16_s",
            Opcode::local_load_extend_i16_u => "local_load_extend_i16_u",
            Opcode::local_load_extend_i8_s => "local_load_extend_i8_s",
            Opcode::local_load_extend_i8_u => "local_load_extend_i8_u",
            Opcode::local_load_extend_f64 => "local_load_extend_f64",
            Opcode::local_load_extend_f32 => "local_load_extend_f32",
            //
            Opcode::local_store_extend_i64 => "local_store_extend_i64",
            Opcode::local_store_extend_i32 => "local_store_extend_i32",
            Opcode::local_store_extend_i16 => "local_store_extend_i16",
            Opcode::local_store_extend_i8 => "local_store_extend_i8",
            Opcode::local_store_extend_f64 => "local_store_extend_f64",
            Opcode::local_store_extend_f32 => "local_store_extend_f32",
            //
            Opcode::heap_load_i64 => "heap_load_i64",
            Opcode::heap_load_i32_s => "heap_load_i32_s",
            Opcode::heap_load_i32_u => "heap_load_i32_u",
            Opcode::heap_load_i16_s => "heap_load_i16_s",
            Opcode::heap_load_i16_u => "heap_load_i16_u",
            Opcode::heap_load_i8_s => "heap_load_i8_s",
            Opcode::heap_load_i8_u => "heap_load_i8_u",
            Opcode::heap_load_f64 => "heap_load_f64",
            Opcode::heap_load_f32 => "heap_load_f32",
            //
            Opcode::heap_store_i64 => "heap_store_i64",
            Opcode::heap_store_i32 => "heap_store_i32",
            Opcode::heap_store_i16 => "heap_store_i16",
            Opcode::heap_store_i8 => "heap_store_i8",
            Opcode::heap_store_f64 => "heap_store_f64",
            Opcode::heap_store_f32 => "heap_store_f32",
            //
            Opcode::heap_fill => "heap_fill",
            Opcode::heap_copy => "heap_copy",
            Opcode::heap_capacity => "heap_capacity",
            Opcode::heap_resize => "heap_resize",
            //
            Opcode::truncate_i64_to_i32 => "truncate_i64_to_i32",
            Opcode::extend_i32_s_to_i64 => "extend_i32_s_to_i64",
            Opcode::extend_i32_u_to_i64 => "extend_i32_u_to_i64",
            Opcode::demote_f64_to_f32 => "demote_f64_to_f32",
            Opcode::promote_f32_to_f64 => "promote_f32_to_f64",
            //
            Opcode::convert_f32_to_i32_s => "convert_f32_to_i32_s",
            Opcode::convert_f32_to_i32_u => "convert_f32_to_i32_u",
            Opcode::convert_f64_to_i32_s => "convert_f64_to_i32_s",
            Opcode::convert_f64_to_i32_u => "convert_f64_to_i32_u",
            Opcode::convert_f32_to_i64_s => "convert_f32_to_i64_s",
            Opcode::convert_f32_to_i64_u => "convert_f32_to_i64_u",
            Opcode::convert_f64_to_i64_s => "convert_f64_to_i64_s",
            Opcode::convert_f64_to_i64_u => "convert_f64_to_i64_u",
            //
            Opcode::convert_i32_s_to_f32 => "convert_i32_s_to_f32",
            Opcode::convert_i32_u_to_f32 => "convert_i32_u_to_f32",
            Opcode::convert_i64_s_to_f32 => "convert_i64_s_to_f32",
            Opcode::convert_i64_u_to_f32 => "convert_i64_u_to_f32",
            Opcode::convert_i32_s_to_f64 => "convert_i32_s_to_f64",
            Opcode::convert_i32_u_to_f64 => "convert_i32_u_to_f64",
            Opcode::convert_i64_s_to_f64 => "convert_i64_s_to_f64",
            Opcode::convert_i64_u_to_f64 => "convert_i64_u_to_f64",
            //
            Opcode::eqz_i32 => "eqz_i32",
            Opcode::nez_i32 => "nez_i32",
            Opcode::eq_i32 => "eq_i32",
            Opcode::ne_i32 => "ne_i32",
            Opcode::lt_i32_s => "lt_i32_s",
            Opcode::lt_i32_u => "lt_i32_u",
            Opcode::gt_i32_s => "gt_i32_s",
            Opcode::gt_i32_u => "gt_i32_u",
            Opcode::le_i32_s => "le_i32_s",
            Opcode::le_i32_u => "le_i32_u",
            Opcode::ge_i32_s => "ge_i32_s",
            Opcode::ge_i32_u => "ge_i32_u",
            //
            Opcode::eqz_i64 => "eqz_i64",
            Opcode::nez_i64 => "nez_i64",
            Opcode::eq_i64 => "eq_i64",
            Opcode::ne_i64 => "ne_i64",
            Opcode::lt_i64_s => "lt_i64_s",
            Opcode::lt_i64_u => "lt_i64_u",
            Opcode::gt_i64_s => "gt_i64_s",
            Opcode::gt_i64_u => "gt_i64_u",
            Opcode::le_i64_s => "le_i64_s",
            Opcode::le_i64_u => "le_i64_u",
            Opcode::ge_i64_s => "ge_i64_s",
            Opcode::ge_i64_u => "ge_i64_u",
            //
            Opcode::eq_f32 => "eq_f32",
            Opcode::ne_f32 => "ne_f32",
            Opcode::lt_f32 => "lt_f32",
            Opcode::gt_f32 => "gt_f32",
            Opcode::le_f32 => "le_f32",
            Opcode::ge_f32 => "ge_f32",
            //
            Opcode::eq_f64 => "eq_f64",
            Opcode::ne_f64 => "ne_f64",
            Opcode::lt_f64 => "lt_f64",
            Opcode::gt_f64 => "gt_f64",
            Opcode::le_f64 => "le_f64",
            Opcode::ge_f64 => "ge_f64",
            //
            Opcode::add_i32 => "add_i32",
            Opcode::sub_i32 => "sub_i32",
            Opcode::add_imm_i32 => "add_imm_i32",
            Opcode::sub_imm_i32 => "sub_imm_i32",
            Opcode::mul_i32 => "mul_i32",
            Opcode::div_i32_s => "div_i32_s",
            Opcode::div_i32_u => "div_i32_u",
            Opcode::rem_i32_s => "rem_i32_s",
            Opcode::rem_i32_u => "rem_i32_u",
            //
            Opcode::add_i64 => "add_i64",
            Opcode::sub_i64 => "sub_i64",
            Opcode::add_imm_i64 => "add_imm_i64",
            Opcode::sub_imm_i64 => "sub_imm_i64",
            Opcode::mul_i64 => "mul_i64",
            Opcode::div_i64_s => "div_i64_s",
            Opcode::div_i64_u => "div_i64_u",
            Opcode::rem_i64_s => "rem_i64_s",
            Opcode::rem_i64_u => "rem_i64_u",
            //
            Opcode::add_f32 => "add_f32",
            Opcode::sub_f32 => "sub_f32",
            Opcode::mul_f32 => "mul_f32",
            Opcode::div_f32 => "div_f32",
            //
            Opcode::add_f64 => "add_f64",
            Opcode::sub_f64 => "sub_f64",
            Opcode::mul_f64 => "mul_f64",
            Opcode::div_f64 => "div_f64",
            //
            Opcode::and => "and",
            Opcode::or => "or",
            Opcode::xor => "xor",
            Opcode::not => "not",
            //
            Opcode::count_leading_zeros_i32 => "count_leading_zeros_i32",
            Opcode::count_leading_ones_i32 => "count_leading_ones_i32",
            Opcode::count_trailing_zeros_i32 => "count_trailing_zeros_i32",
            Opcode::count_ones_i32 => "count_ones_i32",
            Opcode::shift_left_i32 => "shift_left_i32",
            Opcode::shift_right_i32_s => "shift_right_i32_s",
            Opcode::shift_right_i32_u => "shift_right_i32_u",
            Opcode::rotate_left_i32 => "rotate_left_i32",
            Opcode::rotate_right_i32 => "rotate_right_i32",
            Opcode::count_leading_zeros_i64 => "count_leading_zeros_i64",
            Opcode::count_leading_ones_i64 => "count_leading_ones_i64",
            Opcode::count_trailing_zeros_i64 => "count_trailing_zeros_i64",
            Opcode::count_ones_i64 => "count_ones_i64",
            Opcode::shift_left_i64 => "shift_left_i64",
            Opcode::shift_right_i64_s => "shift_right_i64_s",
            Opcode::shift_right_i64_u => "shift_right_i64_u",
            Opcode::rotate_left_i64 => "rotate_left_i64",
            Opcode::rotate_right_i64 => "rotate_right_i64",
            //
            Opcode::abs_i32 => "abs_i32",
            Opcode::neg_i32 => "neg_i32",
            Opcode::abs_i64 => "abs_i64",
            Opcode::neg_i64 => "neg_i64",
            //
            Opcode::abs_f32 => "abs_f32",
            Opcode::neg_f32 => "neg_f32",
            Opcode::copysign_f32 => "copysign_f32",
            Opcode::sqrt_f32 => "sqrt_f32",
            Opcode::min_f32 => "min_f32",
            Opcode::max_f32 => "max_f32",
            Opcode::ceil_f32 => "ceil_f32",
            Opcode::floor_f32 => "floor_f32",
            Opcode::round_half_away_from_zero_f32 => "round_half_away_from_zero_f32",
            Opcode::round_half_to_even_f32 => "round_half_to_even_f32",
            Opcode::trunc_f32 => "trunc_f32",
            Opcode::fract_f32 => "fract_f32",
            Opcode::cbrt_f32 => "cbrt_f32",
            Opcode::exp_f32 => "exp_f32",
            Opcode::exp2_f32 => "exp2_f32",
            Opcode::ln_f32 => "ln_f32",
            Opcode::log2_f32 => "log2_f32",
            Opcode::log10_f32 => "log10_f32",
            Opcode::sin_f32 => "sin_f32",
            Opcode::cos_f32 => "cos_f32",
            Opcode::tan_f32 => "tan_f32",
            Opcode::asin_f32 => "asin_f32",
            Opcode::acos_f32 => "acos_f32",
            Opcode::atan_f32 => "atan_f32",
            Opcode::pow_f32 => "pow_f32",
            Opcode::log_f32 => "log_f32",
            //
            Opcode::abs_f64 => "abs_f64",
            Opcode::neg_f64 => "neg_f64",
            Opcode::copysign_f64 => "copysign_f64",
            Opcode::sqrt_f64 => "sqrt_f64",
            Opcode::min_f64 => "min_f64",
            Opcode::max_f64 => "max_f64",
            Opcode::ceil_f64 => "ceil_f64",
            Opcode::floor_f64 => "floor_f64",
            Opcode::round_half_away_from_zero_f64 => "round_half_away_from_zero_f64",
            Opcode::round_half_to_even_f64 => "round_half_to_even_f64",
            Opcode::trunc_f64 => "trunc_f64",
            Opcode::fract_f64 => "fract_f64",
            Opcode::cbrt_f64 => "cbrt_f64",
            Opcode::exp_f64 => "exp_f64",
            Opcode::exp2_f64 => "exp2_f64",
            Opcode::ln_f64 => "ln_f64",
            Opcode::log2_f64 => "log2_f64",
            Opcode::log10_f64 => "log10_f64",
            Opcode::sin_f64 => "sin_f64",
            Opcode::cos_f64 => "cos_f64",
            Opcode::tan_f64 => "tan_f64",
            Opcode::asin_f64 => "asin_f64",
            Opcode::acos_f64 => "acos_f64",
            Opcode::atan_f64 => "atan_f64",
            Opcode::pow_f64 => "pow_f64",
            Opcode::log_f64 => "log_f64",
            //
            Opcode::end => "end",
            Opcode::block => "block",
            Opcode::break_ => "break",
            Opcode::recur => "recur",
            Opcode::block_alt => "block_alt",
            Opcode::break_alt => "break_alt",
            Opcode::block_nez => "block_nez",
            Opcode::break_nez => "break_nez",
            Opcode::recur_nez => "recur_nez",
            //
            Opcode::call => "call",
            Opcode::dyncall => "dyncall",
            Opcode::envcall => "envcall",
            Opcode::syscall => "syscall",
            Opcode::extcall => "extcall",
            //
            Opcode::pub_index_function => "pub_index_function",
            //
            Opcode::panic => "panic",
            //
            Opcode::host_addr_local => "host_addr_local",
            Opcode::host_addr_local_extend => "host_addr_local_extend",
            Opcode::host_addr_data => "host_addr_data",
            Opcode::host_addr_data_extend => "host_addr_data_extend",
            Opcode::host_addr_heap => "host_addr_heap",
            Opcode::host_copy_heap_to_memory => "host_copy_heap_to_memory",
            Opcode::host_copy_memory_to_heap => "host_copy_memory_to_heap",
            Opcode::host_memory_copy => "host_memory_copy",
            Opcode::host_addr_function => "host_addr_function",
        }
    }
}
