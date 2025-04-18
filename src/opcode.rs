// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

// XiaoXuan Core VM Data Types
// ---------------------------
//
// The XiaoXuan Core VM supports the following primitive data types:
//
// - i32: 32-bit integer
// - i64: 64-bit integer
// - f32: 32-bit floating-point number
// - f64: 64-bit floating-point number
//
// These data types are used for arguments, return values, local variables, and instruction parameters.
//
// Inside the VM:
// - i32 values are sign-extended to i64 when necessary.
// - Instruction parameters may use smaller types like i8 or i16, but these are also sign-extended to i64.

// Memory Representation of Data Types
// -----------------------------------
//
//
//    MSB                             LSB
// 64 |---------------------------------------| 0
//    |   16        16         16     8    8  | bits
//    |---------|---------|---------|----|----|
//    |                  i64                  | <-- native data type
//    |---------------------------------------|
//    | sign-extend       |        i32        |
//    |---------------------------------------|
//    | sign-extend                 |   i16   |
//    |---------------------------------------|
//    | sign-extend                      | i8 |
//    |---------------------------------------|
//    |                  f64                  | <-- native data type
//    |---------------------------------------|
//    | undefined         |          f32      | <-- native data type
//    |---------------------------------------|

// Floating-Point Numbers
// -----------------------
//
// Like most processors and VMs, f32 and f64 are stored using the IEEE 754-2008 format.
//
// Example of f32 encoding:
//
//           MSB                                  LSB
//           sign    exponent (8 bits)   fraction (23 bits)                                 implicit leading number 1
//           ----    -----------------   ------------------                                 |
//           |       |                   |                                                  v
// format    0       00000000            0000000000 0000000000 000     value = (-1)^sign * (1 + fraction) * 2^(exponent-offset), offset = 127 for f32, 1023 for f64
// example   1       10000001            0100000000 0000000000 000     value = (-1)^1 * (1 + 0*2^(-1) + 1*2^(-2)) * 2^(129-127) = -1 * 1.25 * 4 = -5.0
// example   0       01111100            0100000000 0000000000 000     value = (-1)^0 * 1.25 * 2^(-3) = 0.15625
//
// Supported Variants:
//  Yes      -       00000001--\
//                   11111110--/         ---------- ---------- ---     Normal number
//  Yes      0       00000000            0000000000 0000000000 000     Value = +0
//  Yes      1       00000000            0000000000 0000000000 000     Value = -0
//  Yes      -       00000000            ---------- ---------- ---     Subnormal number (i.e., numbers between 0 and MIN)
//  No       0       11111111            0000000000 0000000000 000     Value = +Infinity
//  No       1       11111111            0000000000 0000000000 000     Value = -Infinity
//  No       -       11111111            0-not-all- -zero----- ---     NaN (SNaN)
//  No       -       11111111            1-------- ----------- ---     NaN (QNaN)
//
// Reference:
// Convert floating-point to decimal/hexadecimal: https://www.h-schmidt.net/FloatConverter/IEEE754.html

// Unsupported Floating-Point Variants
// -----------------------------------
//
// In addition to normal floating-point numbers, there are special values (variants):
//
// - NaN
// - +Infinity, -Infinity
// - +0, -0
//
// These variants can confuse programmers, increase language complexity, and lead to unpredictable behavior. For example:
//
// - NaN is not comparable; any comparison involving NaN results in false:
//   * NaN != NaN
//   * (NaN < 0) is false
//   * (NaN > 0) is also false
//   * even if (a != b), you cannot assert !(a == b)
//
// - The values 0.0, -0.0, +Inf, and -Inf are also complex:
//   * 1.0 ÷ 0.0 = +Inf instead of throwing an exception
//   * -1.0 ÷ 0.0 = -Inf, and 1.0 ÷ -0.0 = -Inf instead of throwing an exception
//   * 0.0 ÷ 0.0 = NaN instead of throwing an exception
//   * +Inf != -Inf, 0.0 == -0.0
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
// To simplify the XiaoXuan Core programming language, the f32 and f64 types in the XiaoXuan Core VM
// only support normal (including subnormal) floating-point numbers +0 and -0.
// Other variants (such as NaN, +Inf, -Inf) are not supported.
//
// | variant | memory presentation | support? |
// |---------|---------------------|----------|
// |  0.0    | 0x0000_0000         | Yes      |
// | -0.0    | 0x8000_0000         | Yes      |
// |  NaN    | 0xffc0_0000         | No       |
// | +Inf    | 0x7f80_0000         | No       |
// | -Inf    | 0xff80_0000         | No       |
//
// When loading data from memory as a floating-point number, the following checks are performed:
//
// 1. If the exponent is between (00000001) and (11111110): Pass
// 2. If the exponent is zero: Pass
// 3. Otherwise: Fail
//
// In other words, when loading `+/-Infinity`, or `NaN` from memory, the VM will throw exceptions.

// Boolean Type
// ------------
//
// Boolean values are represented as i64 numbers:
// - TRUE is represented by the number 1 (type i64).
// - FALSE is represented by the number 0 (type i64).
//
// When converting integers to booleans:
// - 0 (type i32 or i64) is treated as FALSE.
// - Any non-zero i32 or i64 value is treated as TRUE.

// Instruction Encoding
// --------------------
//
// VM instructions are NOT fixed-length. They can be 16, 32, 64, 96, or 128 bits long.
//
// Each instruction consists of an opcode and zero or more parameters.
// Parameters only support i16 and i32.
//
// Instruction lengths:
//
// - 16 bits:
//   Instructions without parameters, such as `eqz_i32`.
// - 32 bits:
//   Instructions with 1 parameter, such as `add_imm_i32`.
//   16 bits opcode + 16 bits parameter
// - 64 bits:
//   Instructions with 1 parameter, such as `imm_i32`.
//   16 bits opcode + (16 bits padding) + 32 bits parameter (aligned to 4 bytes)
// - 64 bits:
//   Instructions with 2 parameters, such as `data_load_i64`.
//   16 bits opcode + 16 bits parameter 0 + 32 bits parameter 1 (aligned to 4 bytes)
// - 96 bits:
//   Instructions with 2 parameters, such as `block`.
//   16 bits opcode + (16 bits padding) + 32 bits parameter 0 + 32 bits parameter 1 (aligned to 4 bytes)
// - 128 bits:
//   Instructions with 3 parameters, such as `block_alt`.
//   16 bits opcode + (16 bits padding) + 32 bits parameter 0 + 32 bits parameter 1 + 32 bits parameter 2 (aligned to 4 bytes)
//
// Note: When an instruction contains i32 parameters, it will be aligned to 32 bits (4 bytes).
// If alignment is required, a `nop` instruction will be automatically inserted before such instructions.
//
// Instruction encoding table:
//
// | length  | encoding layout                                                             |
// |---------|-----------------------------------------------------------------------------|
// | 16-bit  | [opcode 16-bit]                                                             |
// | 32-bit  | [opcode 16-bit] - [param i16    ]                                           |
// | 64-bit  | [opcode 16-bit] - [pading 16-bit] + [param i32]                             |
// | 64-bit  | [opcode 16-bit] - [param i16    ] + [param i32]                             |
// | 64-bit  | [opcode 16-bit] - [param i16    ] + [param i16] + [param i16]               |
// | 96-bit  | [opcode 16-bit] - [pading 16-bit] + [param i32] + [param i32]               |
// | 128-bit | [opcode 16-bit] - [pading 16-bit] + [param i32] + [param i32] + [param i32] |

// Opcode Encoding
// ----------------
//
// The opcode consists of two parts: categories and items, both of which are 8-bit numbers.
//
// MSB           LSB
// 00000000 00000000
// -------- --------
// ^        ^
// |        | items
// |
// | categorys

// Object Index in the VM
// -----------------------
//
// The XiaoXuan Core VM uses 'index' instead of 'address/pointer' to access functions, types (signatures),
// data, memory, and local variables.
// This is a security strategy to prevent unsafe memory access.
//
// The 'index' carries information about the kind, data type, length (boundary), and other properties of the object.
// For example, when accessing data using an index, the VM can verify the type and range to ensure safety.

pub const MAX_OPCODE_NUMBER: usize = 0x0c_00;

#[repr(u16)]
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Opcode {
    // Category: Fundamental
    // ----------------------

    // Instruction to do nothing.
    //
    // This is typically used as a padding instruction to ensure 32-bit (4-byte) alignment.
    //
    // () -> ()
    nop = 0x01_00,

    // Pushes an immediate number onto the top of the operand stack.
    //
    // Note: The i32 immediate number will be internally sign-extended to i64 automatically.
    //
    // (param immediate_number:i32) -> i32
    imm_i32,

    // `imm_i64`, `imm_f32`, and `imm_f64` are pseudo-instructions because the VM instructions
    // do not directly support i64, f32, or f64 parameters.
    //
    // Note:
    // - Some ISAs (Instruction Set Architectures), such as ARM, place immediate numbers in a constant
    //   section (".rodata") of the application image and load them by address. Alternatively, they may
    //   place large immediate numbers in the instruction section (".text") but outside the current function.
    // - The XiaoXuan Core VM instructions are variable-length and do not require a dedicated data section.
    //   Immediate numbers are placed directly within the `imm_xxx` instructions.
    //
    imm_i64, // (param number_low:i32 number_high:i32) -> i64
    imm_f32, // (param number:i32) -> f32
    imm_f64, // (param number_low:i32 number_high:i32) -> f64

    // Category: Local Variables
    // --------------------------

    // Local Variables Data Types
    // --------------------------
    // Local variables in the XiaoXuan Core VM only support the following primitive data types:
    // - i32: 32-bit integer
    // - i64: 64-bit integer
    // - f32: 32-bit floating-point number
    // - f64: 64-bit floating-point number
    //
    // Unlike some programming languages that support complex data types (e.g., structs, arrays),
    // the XiaoXuan Core VM does not natively support such types for local variables.
    // Instead, complex data structures can be represented using the "Data" object within the VM.

    // Loading Local Variables
    // -----------------------
    // Loads the specified local variable and pushes it onto the operand stack.
    //
    // Indexing Function Arguments and Local Variables
    // -----------------------------------------------
    // Function arguments are treated as local variables. Local variables follow the arguments.
    // For example, if a function has 2 parameters and 4 local variables, their indices are:
    //
    //
    //        arguments    local variables
    //        [i32 i32]    [i32 i32 i64 i64]
    // index    0   1        2   3   4   5
    //
    // Note:
    // - In some stack-based VMs, function arguments are placed at the top of the stack, allowing direct
    //   access using instructions with implicit "pop" behavior (e.g., `eq_i32`, `add_i32`).
    // - The XiaoXuan Core VM does not guarantee this behavior. Local variables may be stored separately.
    //   Always use `local_load_xxx` instructions to access arguments.
    //
    // Alignment:
    // - In the default VM implementation, all local variables are 8-byte aligned. This is because local
    //   variables are allocated on the stack, which is also 8-byte aligned.

    // Layers
    // ------
    // The parameter `layers` specifies the depth of the frame relative to the current block frame.
    //
    // For example:
    // - `0` for the current frame.
    // - `1` for the parent frame.
    // - `n` for the nth parent frame.
    //
    // ```diagram
    // fn {
    //   ;; frame 0 (function frame)
    //   block
    //     ;; frame 1 (block frame)
    //     block
    //       ;; frame 2 (block frame)
    //       block
    //         ;; frame 3 (block frame)
    //         ;;
    //         ;; Assuming this is the current stack frame, then:
    //         ;; - to get local variables of frame 3 (the current frame): layers = 0
    //         ;; - to get local variables of frame 2: layers = 1
    //         ;; - to get local variables of frame 0 (the function frame): layers = 3
    //       end
    //     end
    //   end
    // }
    // ```
    local_load_i64 = 0x02_00, // (param layers:i16 local_variable_index:i32) -> i64
    local_load_i32_s,         // (param layers:i16 local_variable_index:i32) -> i32
    local_load_i32_u,         // (param layers:i16 local_variable_index:i32) -> i32
    local_load_i16_s,         // (param layers:i16 local_variable_index:i32) -> i16
    local_load_i16_u,         // (param layers:i16 local_variable_index:i32) -> i16
    local_load_i8_s,          // (param layers:i16 local_variable_index:i32) -> i8
    local_load_i8_u,          // (param layers:i16 local_variable_index:i32) -> i8

    // Loads an f64 value with floating-point validity checks.
    //
    // (param layers:i16 local_variable_index:i32) -> f64
    local_load_f64,

    // Loads an f32 value with floating-point validity checks.
    //
    // Note: The high part of the f32 operand (on the stack) is undefined.
    //
    // (param layers:i16 local_variable_index:i32) -> f32
    local_load_f32,

    // Storing Local Variables
    // ------------------------
    // Pops one operand from the operand stack and assigns it to the specified local variable.
    //
    // Return Value: "remain_values"
    // -----------------------------
    // If there are multiple operands on the operand stack, the instruction removes the first operand
    // and leaves the remaining ones.
    //
    // Thinking of instruction "xxx_store_xxx" is a function and the operands is a data list,
    // then this function will consume a centain number of data and return a new list that consists of the remaining elements.
    //
    // It's like the following Rust code:
    //
    // ```rust
    // let mut operands = vec![1,2,3,4,5,6] // corresponsed to the current operands on the operand stack
    // let remains = store(&mut operands, &mut local_var_a) // corresponsed to the "xxx_store_xxx" instruction
    // assert!(remains, vec![2,3])
    // ```
    //
    // Note:
    // - The VM does not support instructions that store multiple operands simultaneously.
    // - If an instruction (e.g., `call`) returns multiple operands, use "xxx_store_xxx" instructions
    //   multiple times to store all return values if necessary.
    //
    local_store_i64, // (param layers:i16 local_variable_index:i32) (operand value:i64) -> (remain_values)
    local_store_i32, // (param layers:i16 local_variable_index:i32) (operand value:i32) -> (remain_values)
    local_store_i16, // (param layers:i16 local_variable_index:i32) (operand value:i32) -> (remain_values)
    local_store_i8, // (param layers:i16 local_variable_index:i32) (operand value:i32) -> (remain_values)
    local_store_f64, // (param layers:i16 local_variable_index:i32) (operand value:f64) -> (remain_values)
    local_store_f32, // (param layers:i16 local_variable_index:i32) (operand value:f32) -> (remain_values)

    // Category: Data
    // --------------

    // Offset Alignment
    // -----------------
    // Data load/store instructions require the "offset" parameter to be a multiple of the following alignments:

    //
    // | Data Type | Alignment (Bytes) |
    // |-----------|-------------------|
    // | i8        | 1                 |
    // | i16       | 2                 |
    // | i32       | 4                 |
    // | i64       | 8                 |
    // | f32       | 4                 |
    // | f64       | 8                 |

    // The Data Public Index
    // ----------------------
    //
    // The data public index is a unified index that includes the following items:
    //
    // - Imported read-only data items
    // - Imported read-write data items
    // - Imported uninitialized data items
    // - Internal read-only data items
    // - Internal read-write data items
    // - Internal uninitialized data items
    // - Dynamically allocated memory
    //
    // In the default VM implementation, the data public index is sorted in the order listed above.
    //
    // Note:
    // - For objects determined before compilation (e.g., functions, types/signatures, data, and local variables),
    //   the index values are generally continuous numbers. However, the index of dynamically allocated memory
    //   is not necessarily sequential. Its value is determined by the VM's implementation,
    //   making this index more like an identifier than a sequential number.
    // - The index should be unique within the VM scope, but it maybe reused for dynamically allocated memory.

    // Load Data
    // ---------
    // Note: All loaded data, except i64, will be sign-extended to i64.
    //
    data_load_i64 = 0x03_00, // (param offset_bytes:i16 data_public_index:i32) -> i64
    data_load_i32_s,         // (param offset_bytes:i16 data_public_index:i32) -> i32
    data_load_i32_u,         // (param offset_bytes:i16 data_public_index:i32) -> i32
    data_load_i16_s,         // (param offset_bytes:i16 data_public_index:i32) -> i16
    data_load_i16_u,         // (param offset_bytes:i16 data_public_index:i32) -> i16
    data_load_i8_s,          // (param offset_bytes:i16 data_public_index:i32) -> i8
    data_load_i8_u,          // (param offset_bytes:i16 data_public_index:i32) -> i8

    // Load a 64-bit floating-point number (f64) with a floating-point validity check.
    //
    // (param offset_bytes:i16 data_public_index:i32) -> f64
    data_load_f64,

    // Load a 32-bit floating-point number (f32) with a floating-point validity check.
    //
    // Note:
    // - The high part of the operand (on the stack) is undefined.
    //
    // (param offset_bytes:i16 data_public_index:i32) -> f32
    data_load_f32,

    data_store_i64, // (param offset_bytes:i16 data_public_index:i32) (operand value:i64) -> (remain_values)
    data_store_i32, // (param offset_bytes:i16 data_public_index:i32) (operand value:i32) -> (remain_values)
    data_store_i16, // (param offset_bytes:i16 data_public_index:i32) (operand value:i32) -> (remain_values)
    data_store_i8, // (param offset_bytes:i16 data_public_index:i32) (operand value:i32) -> (remain_values)
    data_store_f64, // (param offset_bytes:i16 data_public_index:i32) (operand value:f64) -> (remain_values)
    data_store_f32, // (param offset_bytes:i16 data_public_index:i32) (operand value:f32) -> (remain_values)

    // Extended load instructions for various data types with a 64-bit offset.
    data_load_extend_i64, // (param data_public_index:i32) (operand offset_bytes:i64) -> i64
    data_load_extend_i32_s, // (param data_public_index:i32) (operand offset_bytes:i64) -> i32
    data_load_extend_i32_u, // (param data_public_index:i32) (operand offset_bytes:i64) -> i32
    data_load_extend_i16_s, // (param data_public_index:i32) (operand offset_bytes:i64) -> i16
    data_load_extend_i16_u, // (param data_public_index:i32) (operand offset_bytes:i64) -> i16
    data_load_extend_i8_s, // (param data_public_index:i32) (operand offset_bytes:i64) -> i8
    data_load_extend_i8_u, // (param data_public_index:i32) (operand offset_bytes:i64) -> i8
    data_load_extend_f64, // (param data_public_index:i32) (operand offset_bytes:i64) -> f64
    data_load_extend_f32, // (param data_public_index:i32) (operand offset_bytes:i64) -> f32

    // Extended store instructions for various data types with a 64-bit offset.
    data_store_extend_i64, // (param data_public_index:i32) (operand value:i64 offset_bytes:i64) -> (remain_values)
    data_store_extend_i32, // (param data_public_index:i32) (operand value:i32 offset_bytes:i64) -> (remain_values)
    data_store_extend_i16, // (param data_public_index:i32) (operand value:i32 offset_bytes:i64) -> (remain_values)
    data_store_extend_i8, // (param data_public_index:i32) (operand value:i32 offset_bytes:i64) -> (remain_values)
    data_store_extend_f64, // (param data_public_index:i32) (operand value:f64 offset_bytes:i64) -> (remain_values)
    data_store_extend_f32, // (param data_public_index:i32) (operand value:f32 offset_bytes:i64) -> (remain_values)

    // Dynamic data load instructions which support dynamic module index, data public index and 64-bit offset.
    data_load_dynamic_i64, // () (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> i64
    data_load_dynamic_i32_s, // () (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> i32
    data_load_dynamic_i32_u, // () (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> i32
    data_load_dynamic_i16_s, // () (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> i16
    data_load_dynamic_i16_u, // () (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> i16
    data_load_dynamic_i8_s, // () (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> i8
    data_load_dynamic_i8_u, // () (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> i8
    data_load_dynamic_f64, // () (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> f64
    data_load_dynamic_f32, // () (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> f32

    // Dynamic data store instructions which support dynamic module index, data public index and 64-bit offset.
    data_store_dynamic_i64, // () (operand value:i64 module_index:i32 data_public_index:i32 offset_bytes:i64) -> (remain_values)
    data_store_dynamic_i32, // () (operand value:i32 module_index:i32 data_public_index:i32 offset_bytes:i64) -> (remain_values)
    data_store_dynamic_i16, // () (operand value:i32 module_index:i32 data_public_index:i32 offset_bytes:i64) -> (remain_values)
    data_store_dynamic_i8, // () (operand value:i32 module_index:i32 data_public_index:i32 offset_bytes:i64) -> (remain_values)
    data_store_dynamic_f64, // () (operand value:f64 module_index:i32 data_public_index:i32 offset_bytes:i64) -> (remain_values)
    data_store_dynamic_f32, // () (operand value:f32 module_index:i32 data_public_index:i32 offset_bytes:i64) -> (remain_values)

    // Category: Arithmetic
    // --------------------

    // Examples of arithmetic instructions
    // -----------------------------------
    //
    // Instruction `sub_i32`:
    //
    // ```assembly
    // ;; Load two numbers onto the operand stack
    // imm_i32(10)
    // imm_i32(3)
    //
    // ;; Subtract one number from the other
    // ;; The top operand on the operand stack is 7 (10 - 3 = 7)
    // sub_i32()
    // ```
    //
    // Instruction `rem_i32_u`:
    //
    // ```assembly
    // ;; Load two numbers onto the operand stack
    // imm_i32(10)
    // imm_i32(3)
    //
    // ;; Calculate the remainder of dividing one number by the other
    // ;; The top operand on the operand stack is 1 (10 % 3 = 1)
    // rem_i32_u()
    // ```

    // Wrapping addition, e.g., 0xffff_ffff + 2 = 1 (-1 + 2 = 1)
    //
    // () (operand left:i32 right:i32) -> i32
    add_i32 = 0x04_00,

    // Wrapping subtraction, e.g., 11 - 211 = -200
    //
    // () (operand left:i32 right:i32) -> i32
    sub_i32,

    // Wrapping increment with an immediate value, e.g., 0xffff_ffff + 2 = 1
    //
    // (param imm:i16) (operand number:i32) -> i32
    add_imm_i32,

    // Wrapping decrement with an immediate value, e.g., 0x1 - 2 = 0xffff_ffff
    //
    // (param imm:i16) (operand number:i32) -> i32
    sub_imm_i32,

    // Wrapping multiplication, e.g., 0xf0e0d0c0 * 2 = 0xf0e0d0c0 << 1
    //
    // () (operand left:i32 right:i32) -> i32
    mul_i32,

    // Signed division
    //
    // () (operand left:i32 right:i32) -> i32
    div_i32_s,

    // Unsigned division
    //
    // () (operand left:i32 right:i32) -> i32
    div_i32_u,

    // Signed remainder
    //
    // () (operand left:i32 right:i32) -> i32
    rem_i32_s,

    // Unsigned remainder
    //
    // () (operand left:i32 right:i32) -> i32
    rem_i32_u,

    // Remainder and modulus
    // ----------------------
    //
    // The remainder (%) operator returns the remainder left over when one operand is
    // divided by another. It always takes the sign of the dividend (for the operation `n % d`,
    // operand `n` is called the dividend and `d` is called the divisor).
    //
    // Example calculations:
    // (13 % 5) = 3
    //  ^    ^
    //  |    | divisor
    //  | dividend <--------- The result of the remainder always takes the sign of the dividend.
    //
    // Definition: `remainder(a, b) = a − b × trunc(a / b)`, where `trunc(x)` rounds `x` toward zero.
    //
    // Examples:
    // *  5 rem  3 =  2   ( 5 - 3 * 1  = 2)
    // * -5 rem  3 = -2   (-5 - 3 * (-1) = -2)
    // *  5 rem -3 =  2   ( 5 - (-3) * (-1) = 2)
    // * -5 rem -3 = -2   (-5 - (-3) * (1) = -2)
    //
    // Reference: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Remainder
    //
    // The modulus operator, in contrast, always has the same sign as the divisor.
    // Definition: `a mod b = a − b × floor(a / b)`, where `floor(x)` rounds `x` down to negative infinity.
    //
    // Examples:
    // *  5 mod  3 =  2   ( 5 - 3 * 1  = 2)
    // * -5 mod  3 =  1   (-5 - 3 * (-2) = 1)
    // *  5 mod -3 = -1   ( 5 - (-3) * (-2) = -1)
    // * -5 mod -3 = -2   (-5 - (-3) * (1) = -2)
    //
    // References:
    // - https://stackoverflow.com/questions/13683563/whats-the-difference-between-mod-and-remainder
    // - https://en.wikipedia.org/wiki/Euclidean_division
    // - https://en.wikipedia.org/wiki/Modulo
    // Wrapping addition for i64
    //
    // () (operand left:i64 right:i64) -> i64
    add_i64,

    // Wrapping subtraction for i64
    //
    // () (operand left:i64 right:i64) -> i64
    sub_i64,

    // Wrapping increment with an immediate value for i64
    //
    // (param imm:i16) (operand number:i64) -> i64
    add_imm_i64,

    // Wrapping decrement with an immediate value for i64
    //
    // (param imm:i16) (operand number:i64) -> i64
    sub_imm_i64,

    // Wrapping multiplication for i64
    //
    // () (operand left:i64 right:i64) -> i64
    mul_i64,

    // Signed division for i64
    //
    // () (operand left:i64 right:i64) -> i64
    div_i64_s,

    // Unsigned division for i64
    //
    // () (operand left:i64 right:i64) -> i64
    div_i64_u,

    // Signed remainder for i64
    //
    // () (operand left:i64 right:i64) -> i64
    rem_i64_s,

    // Unsigned remainder for i64
    //
    // () (operand left:i64 right:i64) -> i64
    rem_i64_u,

    // Floating-point addition for f32
    //
    // () (operand left:f32 right:f32) -> f32
    add_f32,

    // Floating-point subtraction for f32
    //
    // () (operand left:f32 right:f32) -> f32
    sub_f32,

    // Floating-point multiplication for f32
    //
    // () (operand left:f32 right:f32) -> f32
    mul_f32,

    // Floating-point division for f32
    //
    // () (operand left:f32 right:f32) -> f32
    div_f32,

    // Floating-point addition for f64
    //
    // () (operand left:f64 right:f64) -> f64
    add_f64,

    // Floating-point subtraction for f64
    //
    // () (operand left:f64 right:f64) -> f64
    sub_f64,

    // Floating-point multiplication for f64
    //
    // () (operand left:f64 right:f64) -> f64
    mul_f64,

    // Floating-point division for f64
    //
    // () (operand left:f64 right:f64) -> f64
    div_f64,

    // Category: Bitwise
    // -----------------
    //
    // Reference:
    // https://en.wikipedia.org/wiki/Bitwise_operation

    // Examples of Bitwise Instructions
    // ---------------------------------
    //
    // Instruction `shift_left_i32`:
    //
    // ```assembly
    // ;; Load a number onto the operand stack
    // imm_i32(7)              ;; 0b0000_0111
    //
    // ;; Perform a bitwise left shift.
    //
    // ;; The top operand on the operand stack is 14 (0b0000_1110)
    // shift_left_i32(1)
    // ```
    //
    // Instruction `count_leading_zeros_i32`:
    //
    // ```assembly
    // ;; Load a number onto the operand stack
    // imm_i32(8_388_608)      ;; 00000000_10000000_00000000_00000000
    //
    // ;; Count leading zeros
    // ;; The top operand on the operand stack is 8
    // count_leading_zeros_i32()
    // ```
    //
    // Example of instruction `count_trailing_zeros_i32`:
    //
    // ```assembly
    // ;; Load a number onto the stack
    // imm_i32(8_388_608)      ;; 00000000_10000000_00000000_00000000
    //
    // ;; Count trailing zeros
    // ;; The top operand on the stack will be 23
    // count_trailing_zeros_i32()
    // ```
    //
    // Instruction `count_ones_i32`:
    //
    // ```assembly
    // ;; Load a number onto the stack
    // imm_i32(130)            ;; 0b1000_0010
    //
    // ;; Count the number of 1s in the binary representation.
    // ;; The top operand on the operand stack is 2
    // count_ones_i32()
    // ```
    and = 0x05_00, // Bitwise AND operation: () (operand left:i64, right:i64) -> i64
    or,            // Bitwise OR operation: () (operand left:i64, right:i64) -> i64
    xor,           // Bitwise XOR operation: () (operand left:i64, right:i64) -> i64
    not,           // Bitwise NOT operation: () (operand number:i64) -> i64

    shift_left_i32, // Left shift: () (operand number:i32, move_bits:i32) -> i32, move_bits = [0, 32)
    shift_right_i32_s, // Arithmetic right shift: () (operand number:i32, move_bits:i32) -> i32, move_bits = [0, 32)
    shift_right_i32_u, // Logical right shift: () (operand number:i32, move_bits:i32) -> i32, move_bits = [0, 32)
    rotate_left_i32, // Left rotate: () (operand number:i32, move_bits:i32) -> i32, move_bits = [0, 32)
    rotate_right_i32, // Right rotate: () (operand number:i32, move_bits:i32) -> i32, move_bits = [0, 32)

    count_leading_zeros_i32, // Count leading zeros: () (operand number:i32) -> i32
    count_leading_ones_i32,  // Count leading ones: () (operand number:i32) -> i32
    count_trailing_zeros_i32, // Count trailing zeros: () (operand number:i32) -> i32
    count_ones_i32, // Count the number of 1s in the binary representation: () (operand number:i32) -> i32

    shift_left_i64, // Left shift: () (operand number:i64, move_bits:i32) -> i64, move_bits = [0, 64)
    shift_right_i64_s, // Arithmetic right shift: () (operand number:i64, move_bits:i32) -> i64, move_bits = [0, 64)
    shift_right_i64_u, // Logical right shift: () (operand number:i64, move_bits:i32) -> i64, move_bits = [0, 64)
    rotate_left_i64, // Left rotate: () (operand number:i64, move_bits:i32) -> i64, move_bits = [0, 64)
    rotate_right_i64, // Right rotate: () (operand number:i64, move_bits:i32) -> i64, move_bits = [0, 64)

    count_leading_zeros_i64, // Count leading zeros: () (operand number:i64) -> i32
    count_leading_ones_i64,  // Count leading ones: () (operand number:i64) -> i32
    count_trailing_zeros_i64, // Count trailing zeros: () (operand number:i64) -> i32
    count_ones_i64, // Count the number of 1s in the binary representation: () (operand number:i64) -> i32

    // Category: Math
    // --------------
    //
    // Mathematical operations for integers and floating-point numbers.

    // Absolute value for i32
    //
    // () (operand number:i32) -> i32
    abs_i32 = 0x06_00,

    // Negation for i32
    //
    // () (operand number:i32) -> i32
    neg_i32,

    // Absolute value for i64
    //
    // () (operand number:i64) -> i64
    abs_i64,

    // Negation for i64
    //
    // () (operand number:i64) -> i64
    neg_i64,

    // Absolute value for f32
    //
    // () (operand number:f32) -> f32
    abs_f32,

    // Negation for f32
    //
    // () (operand number:f32) -> f32
    neg_f32,

    // Copy the sign of one floating-point number to another for f32
    //
    // () (operand num:f32 sign:f32) -> f32
    copysign_f32,

    // Square root for f32
    //
    // () (operand number:f32) -> f32
    sqrt_f32,

    // Minimum of two f32 values
    //
    // () (operand left:f32 right:f32) -> f32
    min_f32,

    // Maximum of two f32 values
    //
    // () (operand left:f32 right:f32) -> f32
    max_f32,

    // Ceiling of an f32 value (round up to the nearest integer)
    //
    // () (operand number:f32) -> f32
    ceil_f32,

    // Floor of an f32 value (round down to the nearest integer)
    //
    // () (operand number:f32) -> f32
    floor_f32,

    // Rounding examples for `round_half_away_from_zero`:
    //
    // * round_half_away_from_zero_f32(2.4) = 2.0
    // * round_half_away_from_zero_f32(2.6) = 3.0
    // * round_half_away_from_zero_f32(2.5) = 3.0
    // * round_half_away_from_zero_f32(-2.5) = -3.0
    //
    // Reference:
    // https://en.wikipedia.org/wiki/Rounding#Rounding_half_away_from_zero
    round_half_away_from_zero_f32, // () (operand number:f32) -> f32

    // Rounding to the nearest even number for f32
    //
    // () (operand number:f32) -> f32
    round_half_to_even_f32,

    // Truncate an f32 value to its integer part
    //
    // () (operand number:f32) -> f32
    trunc_f32,

    // Extract the fractional part of an f32 value
    //
    // () (operand number:f32) -> f32
    fract_f32,

    // Cube root for f32
    //
    // () (operand number:f32) -> f32
    cbrt_f32,

    // Exponential function (e^x) for f32
    //
    // () (operand number:f32) -> f32
    exp_f32,

    // Exponential function (2^x) for f32
    //
    // () (operand number:f32) -> f32
    exp2_f32,

    // Natural logarithm (log_e) for f32
    //
    // () (operand number:f32) -> f32
    ln_f32,

    // Base-2 logarithm (log_2) for f32
    //
    // () (operand number:f32) -> f32
    log2_f32,

    // Base-10 logarithm (log_10) for f32
    //
    // () (operand number:f32) -> f32
    log10_f32,

    // Sine function for f32
    //
    // () (operand number:f32) -> f32
    sin_f32,

    // Cosine function for f32
    //
    // () (operand number:f32) -> f32
    cos_f32,

    // Tangent function for f32
    //
    // () (operand number:f32) -> f32
    tan_f32,

    // Arcsine function for f32
    //
    // () (operand number:f32) -> f32
    asin_f32,

    // Arccosine function for f32
    //
    // () (operand number:f32) -> f32
    acos_f32,

    // Arctangent function for f32
    //
    // () (operand number:f32) -> f32
    atan_f32,

    // Power function (base^exponent) for f32
    //
    // () (operand base:f32 exponent:f32) -> f32
    pow_f32,

    // Logarithm with a custom base for f32
    //
    // () (operand number:f32 base:f32) -> f32
    log_f32,

    // Absolute value for f64
    //
    // () (operand number:f64) -> f64
    abs_f64,

    // Negation for f64
    //
    // () (operand number:f64) -> f64
    neg_f64,

    // Copy the sign of one floating-point number to another for f64
    //
    // () (operand num:f64 sign:f64) -> f64
    copysign_f64,

    // Square root for f64
    //
    // () (operand number:f64) -> f64
    sqrt_f64,

    // Minimum of two f64 values
    //
    // () (operand left:f64 right:f64) -> f64
    min_f64,

    // Maximum of two f64 values
    //
    // () (operand left:f64 right:f64) -> f64
    max_f64,

    // Ceiling of an f64 value (round up to the nearest integer)
    //
    // () (operand number:f64) -> f64
    ceil_f64,

    // Floor of an f64 value (round down to the nearest integer)
    //
    // () (operand number:f64) -> f64
    floor_f64,

    // Rounding examples for `round_half_away_from_zero`:
    //
    // * round_half_away_from_zero_f64(2.4) = 2.0
    // * round_half_away_from_zero_f64(2.6) = 3.0
    // * round_half_away_from_zero_f64(2.5) = 3.0
    // * round_half_away_from_zero_f64(-2.5) = -3.0
    round_half_away_from_zero_f64, // () (operand number:f64) -> f64

    // Rounding to the nearest even number for f64
    //
    // () (operand number:f64) -> f64
    round_half_to_even_f64,

    // Truncate an f64 value to its integer part
    //
    // () (operand number:f64) -> f64
    trunc_f64,

    // Extract the fractional part of an f64 value
    //
    // () (operand number:f64) -> f64
    fract_f64,

    // Cube root for f64
    //
    // () (operand number:f64) -> f64
    cbrt_f64,

    // Exponential function (e^x) for f64
    //
    // () (operand number:f64) -> f64
    exp_f64,

    // Exponential function (2^x) for f64
    //
    // () (operand number:f64) -> f64
    exp2_f64,

    // Natural logarithm (log_e) for f64
    //
    // () (operand number:f64) -> f64
    ln_f64,

    // Base-2 logarithm (log_2) for f64
    //
    // () (operand number:f64) -> f64
    log2_f64,

    // Base-10 logarithm (log_10) for f64
    //
    // () (operand number:f64) -> f64
    log10_f64,

    // Sine function for f64
    //
    // () (operand number:f64) -> f64
    sin_f64,

    // Cosine function for f64
    //
    // () (operand number:f64) -> f64
    cos_f64,

    // Tangent function for f64
    //
    // () (operand number:f64) -> f64
    tan_f64,

    // Arcsine function for f64
    //
    // () (operand number:f64) -> f64
    asin_f64,

    // Arccosine function for f64
    //
    // () (operand number:f64) -> f64
    acos_f64,

    // Arctangent function for f64
    //
    // () (operand number:f64) -> f64
    atan_f64,

    // Power function (base^exponent) for f64
    //
    // () (operand base:f64 exponent:f64) -> f64
    pow_f64,

    // Logarithm with a custom base for f64
    //
    // () (operand number:f64 base:f64) -> f64
    log_f64,

    // Category: Conversion
    // --------------------

    // Truncate a 64-bit integer (i64) to a 32-bit integer (i32).
    // Discards the high 32 bits of the i64 value.
    //
    // () (operand number:i64) -> i32
    truncate_i64_to_i32 = 0x07_00,

    // Sign-extend a 32-bit integer (i32) to a 64-bit integer (i64).
    extend_i32_s_to_i64, // () (operand number:i32) -> i64

    // Zero-extend a 32-bit integer (i32) to a 64-bit integer (i64).
    extend_i32_u_to_i64, // () (operand number:i32) -> i64

    // Convert a 64-bit floating-point number (f64) to a 32-bit floating-point number (f32).
    // This operation may lose precision.
    demote_f64_to_f32, // () (operand number:f64) -> f32

    // Convert a 32-bit floating-point number (f32) to a 64-bit floating-point number (f64).
    //
    // () (operand number: f32) -> f64
    promote_f32_to_f64, // () (operand number:f32) -> f64

    // Convert a 32-bit floating-point number (f32) to a signed 32-bit integer (i32).
    // The fractional part is truncated.
    //
    // () (operand number:f32) -> i32
    convert_f32_to_i32_s,

    // Convert a 32-bit floating-point number (f32) to an unsigned 32-bit integer (i32).
    // The fractional part is truncated.
    // Note: Negative values (-x.xx) will result in 0.
    //
    // () (operand number:f32) -> i32
    convert_f32_to_i32_u,

    // Convert a 64-bit floating-point number (f64) to a signed 32-bit integer (i32).
    // The fractional part is truncated.
    //
    // () (operand number:f64) -> i32
    convert_f64_to_i32_s,

    // Convert a 64-bit floating-point number (f64) to an unsigned 32-bit integer (i32).
    // The fractional part is truncated.
    // Note: Negative values (-x.xx) will result in 0.
    //
    // () (operand number: f64) -> i32
    convert_f64_to_i32_u,

    // Convert a 32-bit floating-point number (f32) to a signed 64-bit integer (i64).
    // The fractional part is truncated.
    //
    // () (operand number: f32) -> i64
    convert_f32_to_i64_s,

    // Convert a 32-bit floating-point number (f32) to an unsigned 64-bit integer (i64).
    // The fractional part is truncated.
    // Note: Negative values (-x.xx) will result in 0.
    //
    // () (operand number: f32) -> i64
    convert_f32_to_i64_u,

    // Convert a 64-bit floating-point number (f64) to a signed 64-bit integer (i64).
    // The fractional part is truncated.
    //
    // () (operand number: f64) -> i64
    convert_f64_to_i64_s,

    // Convert a 64-bit floating-point number (f64) to an unsigned 64-bit integer (i64).
    // The fractional part is truncated.
    // Note: Negative values (-x.xx) will result in 0.
    //
    // () (operand number: f64) -> i64
    convert_f64_to_i64_u,

    // Convert a signed 32-bit integer (i32) to a 32-bit floating-point number (f32).
    //
    // () (operand number: i32) -> f32
    convert_i32_s_to_f32,

    // Convert an unsigned 32-bit integer (i32) to a 32-bit floating-point number (f32).
    //
    // () (operand number: i32) -> f32
    convert_i32_u_to_f32,

    // Convert a signed 64-bit integer (i64) to a 32-bit floating-point number (f32).
    //
    // () (operand number: i64) -> f32
    convert_i64_s_to_f32,

    // Convert an unsigned 64-bit integer (i64) to a 32-bit floating-point number (f32).
    //
    // () (operand number: i64) -> f32
    convert_i64_u_to_f32,

    // Convert a signed 32-bit integer (i32) to a 64-bit floating-point number (f64).
    //
    // () (operand number: i32) -> f64
    convert_i32_s_to_f64,

    // Convert an unsigned 32-bit integer (i32) to a 64-bit floating-point number (f64).
    //
    // () (operand number: i32) -> f64
    convert_i32_u_to_f64,

    // Convert a signed 64-bit integer (i64) to a 64-bit floating-point number (f64).
    //
    // () (operand number: i64) -> f64
    convert_i64_s_to_f64,

    // Convert an unsigned 64-bit integer (i64) to a 64-bit floating-point number (f64).
    //
    // () (operand number: i64) -> f64
    convert_i64_u_to_f64,

    // Category: Comparison
    // --------------------

    // Note: For binary operations, the first operand popped from the operand stack
    // is the right-hand-side value, and the second operand is the left-hand-side value.
    // For example:
    //
    // |                 | --> stack end
    // | right-hand side | --> the 1st pop (right-hand-side value)
    // | left-hand side  | --> the 2nd pop (left-hand-side value)
    // \-----------------/ --> stack start
    //
    // This order matches the function parameter order. For example, the parameters
    // of the function `add(a, b, c)` on the stack are as follows:
    //
    //  |   | --> stack end
    //  | c |
    //  | b |
    //  | a |
    //  \---/ --> stack start
    //
    // The two operands MUST be of the same data type for comparison instructions.
    // The result of the comparison is a logical TRUE or FALSE. Specifically:
    // - If the result is TRUE, the number `1 (type i64)` is pushed onto the operand stack.
    // - If the result is FALSE, the number `0 (type i64)` is pushed onto the operand stack.
    //
    // Example of the `lt_i32_u` instruction:
    //
    // ```assembly
    // ;; Load two numbers onto the stack
    // imm_i32(11)
    // imm_i32(22)
    //
    // ;; Now the layout of the operand stack is:
    // ;; |    | --> stack end
    // ;; | 22 |
    // ;; | 11 |
    // ;; \----/ --> stack start
    //
    // ;; `lt_i32_u` checks if '11' is less than '22'.
    // ;; If true, `1 (type i64)` is pushed onto the operand stack.
    // lt_i32_u()
    //
    // ;; Now the layout of the operand stack is:
    // ;; |    | --> stack end
    // ;; | 1  |
    // ;; \----/ --> stack start
    // ```
    //
    eqz_i32 = 0x08_00, // Checks if the operand is zero. () (operand number: i32) -> i64
    nez_i32,           // Checks if the operand is non-zero. () (operand number: i32) -> i64
    eq_i32, // Compares two i32 values for equality. () (operand left: i32, right: i32) -> i64
    ne_i32, // Compares two i32 values for inequality. () (operand left: i32, right: i32) -> i64
    lt_i32_s, // Checks if the left i32 value is less than the right (signed). () (operand left: i32, right: i32) -> i64
    lt_i32_u, // Checks if the left i32 value is less than the right (unsigned). () (operand left: i32, right: i32) -> i64
    gt_i32_s, // Checks if the left i32 value is greater than the right (signed). () (operand left: i32, right: i32) -> i64
    gt_i32_u, // Checks if the left i32 value is greater than the right (unsigned). () (operand left: i32, right: i32) -> i64
    le_i32_s, // Checks if the left i32 value is less than or equal to the right (signed). () (operand left: i32, right: i32) -> i64
    le_i32_u, // Checks if the left i32 value is less than or equal to the right (unsigned). () (operand left: i32, right: i32) -> i64
    ge_i32_s, // Checks if the left i32 value is greater than or equal to the right (signed). () (operand left: i32, right: i32) -> i64
    ge_i32_u, // Checks if the left i32 value is greater than or equal to the right (unsigned). () (operand left: i32, right: i32) -> i64

    eqz_i64,  // Checks if the operand is zero. () (operand number: i64) -> i64
    nez_i64,  // Checks if the operand is non-zero. () (operand number: i64) -> i64
    eq_i64,   // Compares two i64 values for equality. () (operand left: i64, right: i64) -> i64
    ne_i64,   // Compares two i64 values for inequality. () (operand left: i64, right: i64) -> i64
    lt_i64_s, // Checks if the left i64 value is less than the right (signed). () (operand left: i64, right: i64) -> i64
    lt_i64_u, // Checks if the left i64 value is less than the right (unsigned). () (operand left: i64, right: i64) -> i64
    gt_i64_s, // Checks if the left i64 value is greater than the right (signed). () (operand left: i64, right: i64) -> i64
    gt_i64_u, // Checks if the left i64 value is greater than the right (unsigned). () (operand left: i64, right: i64) -> i64
    le_i64_s, // Checks if the left i64 value is less than or equal to the right (signed). () (operand left: i64, right: i64) -> i64
    le_i64_u, // Checks if the left i64 value is less than or equal to the right (unsigned). () (operand left: i64, right: i64) -> i64
    ge_i64_s, // Checks if the left i64 value is greater than or equal to the right (signed). () (operand left: i64, right: i64) -> i64
    ge_i64_u, // Checks if the left i64 value is greater than or equal to the right (unsigned). () (operand left: i64, right: i64) -> i64

    eq_f32, // Compares two f32 values for equality. () (operand left: f32, right: f32) -> i64
    ne_f32, // Compares two f32 values for inequality. () (operand left: f32, right: f32) -> i64
    lt_f32, // Checks if the left f32 value is less than the right. () (operand left: f32, right: f32) -> i64
    gt_f32, // Checks if the left f32 value is greater than the right. () (operand left: f32, right: f32) -> i64
    le_f32, // Checks if the left f32 value is less than or equal to the right. () (operand left: f32, right: f32) -> i64
    ge_f32, // Checks if the left f32 value is greater than or equal to the right. () (operand left: f32, right: f32) -> i64
    eq_f64, // Compares two f64 values for equality. () (operand left: f64, right: f64) -> i64
    ne_f64, // Compares two f64 values for inequality. () (operand left: f64, right: f64) -> i64
    lt_f64, // Checks if the left f64 value is less than the right. () (operand left: f64, right: f64) -> i64
    gt_f64, // Checks if the left f64 value is greater than the right. () (operand left: f64, right: f64) -> i64
    le_f64, // Checks if the left f64 value is less than or equal to the right. () (operand left: f64, right: f64) -> i64
    ge_f64, // Checks if the left f64 value is greater than or equal to the right. () (operand left: f64, right: f64) -> i64

    // Category: Control flow
    // ----------------------

    // Control flow instruction "end".
    //
    // When the "end" instruction is executed, the current stack frame is removed,
    // and the results of the current block or function are placed at the top of the operand stack.
    //
    // () -> NO_RETURN
    end = 0x09_00,

    // The "block" instruction creates a new block scope.
    //
    // A block is similar to a function in that it has parameters and results (referred to as `type`),
    // which are the same as function types.
    //
    // When the "block" instruction is executed, a new stack frame, called a "block stack frame," is created.
    // This frame is similar to a "function stack frame."
    //
    // Note: This instruction differs from the WebAssembly "block" instruction. In WebAssembly,
    // parameters are not treated as local variables (their values are placed directly on the operand stack),
    // and they cannot be accessed using "local_load_xxx/local_store_xxx" instructions.
    //
    // (param type_index:i32 local_variable_list_index:i32) -> NO_RETURN
    block,

    // The "break" instruction is used to exit a block or function, similar to the "end" instruction.
    //
    // - For a block:
    //   Removes the block stack frame and jumps to the instruction immediately following the "end" instruction.
    //   The parameter `next_inst_offset` should be calculated as:
    //   `address of the instruction after "end" - address of "break"`.
    //
    // - For a function:
    //   The parameter `next_inst_offset` is ignored. The function stack frame is removed,
    //   and execution resumes at the instruction following the "call" instruction.
    //
    // Notes:
    // - The `next_inst_offset` parameter simplifies instruction execution. In some VMs, similar parameters
    //   are calculated at runtime or during the loading stage.
    // - The "end" and "break" instructions are almost identical, except that "break" allows specifying
    //   the `layers` and `next_inst_offset` parameters. In essence, "end" is equivalent to
    //   "break" with `layers=0` and `next_inst_offset=8`.
    //
    // Example:
    //
    //
    // ```bytecode
    // 0d0000 block(0)          ;; the size of instruction "block" is 8 bytes
    // 0d0008   nop             ;;
    // 0d0010   break(0,14)     ;; the size of instruction "break" is 8 bytes, (14 = 24 - 10) --------\
    // 0d0018   nop             ;;                                                                    |
    // 0d0020   nop             ;;                                                                    |
    // 0d0022 end               ;;                                                                    |
    // 0d0024 nop               ;; <-- execution jumps here (the instruction after "end") <-----------/
    // ```
    //
    // The "break" instruction not only exits a block or function but also transfers operands
    // out of the block or function. For example:
    //
    // ```bytecode
    // 0d0000 block(0)          ;; assumes the block type is '() -> (i32,i32)'
    // 0d0008   imm_i32(11)     ;;
    // 0d0016   imm_i32(13)     ;;                 | 17                 | -----\ operands 13 and 17 are
    // 0d0024   imm_i32(17)     ;; --------------> | 13                 | -----| taken out of the block frame
    // 0d0032   break(0,18)     ;; ---\            | 11                 |      |
    // 0d0040   nop             ;;    |            | [block frame info] |      v
    // 0d0042   nop             ;;    | jump       | ..                 |    | 17                 |
    // 0d0044   nop             ;;    |            | [func frame info]  |    | 13                 |
    // 0d0046   nop             ;;    |            \____________________/    | ..                 |
    // 0d0048 end               ;;    v                 stack layout         | [func frame info]  |
    // 0d0050 nop               ;; <--/----------<-------------------------- \____________________/
    //                          ;;                                                stack layout
    //                          ;;
    //                          ;; Note: 18 = 50 - 32
    // ```
    //
    // Additionally, the "break" instruction can cross multiple block layers.
    // When `layers` is 0, it exits the current block. If `layers` is greater than 0,
    // it removes multiple block stack frames and transfers the corresponding operands out of the blocks.
    // The number of operands is determined by the target block's type.
    //
    // ```bytecode
    // 0d0000 block 0           ;; assumes the block type is '()->(i32,i32,i32)'
    // 0d0008   block 0         ;; assumes the block type is '()->(i32,i32)'
    // 0d0016     block 0       ;; assumes the block type is '()->(i32)'
    // 0d0024       imm_i32(11) ;;
    // 0d0032       imm_i32(13) ;;
    // 0d0040       imm_i32(17) ;;
    // 0d0048       break(1,16) ;; (16 = 64 - 48) --------\
    // 0d0056       nop         ;;                        |
    // 0d0058     end           ;;                        |
    // 0d0060     nop           ;;                        |
    // 0d0062   end             ;;                        | transfers operands (13, 17) and
    // 0d0064   nop             ;; <----------------------/ jumps to here
    // 0d0066 end
    // ```
    //
    // Note: WebAssembly has a similar instruction called "br/break," which jumps to the "end" instruction's address.
    // While more elegant, it is less efficient because it transfers return values (operands) twice.
    // The XiaoXuan Core "break" instruction balances performance and elegance by implying "end"
    // and directly jumping to the instruction after "end."
    //
    // (param layers:i16 next_inst_offset:i32) NO_RETURN
    break_,

    // The "recur" instruction allows the VM to jump to the instruction immediately following
    // the "block" instruction or the first instruction of the current function.
    // All operands in the current stack frame are removed, except for the operands
    // corresponding to the parameters of the target block or function, which are preserved
    // and placed at the top of the operand stack.
    //
    // The "recur" instruction is commonly used to construct "loop" or "for" structures
    // in general programming languages. It is also used to implement Tail Call Optimization (TCO).
    //
    // If the target frame is the function itself, the `start_inst_offset` parameter is ignored,
    // and all local variables are reset to 0 (except for the arguments).
    //
    // Note: The value of `start_inst_offset` is a positive number and is calculated as:
    // (address of the "recur" instruction - address of the target instruction).
    //
    // Example:
    //
    // ```bytecode
    // 0d0000 block(0)          ;; assumes the block type is "(i32,i32)->()"
    // 0d0008   imm_i32(11)     ;; <------\
    // 0d0016   imm_i32(13)     ;;        |         | 17                 | -----\ operands 13 and 17 are
    // 0d0024   imm_i32(17)     ;; ---------------> | 13                 | -----| taken out of the block frame
    // 0d0032   nop             ;;        |         | 11                 |      |
    // 0d0034   nop             ;;        ^         | [block frame info] |      v
    // 0d0036   nop             ;;        |         | ..                 |    | 17                 |
    // 0d0038   nop             ;;  jump  |         | [func frame info]  |    | 13                 |
    // 0d0040   recur(0,32)     ;; -------/         \____________________/    | [block frame info] |
    // 0d0048 end               ;;        |             stack layout          | ..                 |
    // 0d0050 nop               ;;        \-------<-------------------------- | [func frame info]  |
    //                          ;;                                            \____________________/
    //                          ;;                                                 stack layout
    // ```
    //
    // The "recur" instruction can also cross multiple block layers:
    //
    // ```bytecode
    // 0d0000 block(0)          ;; assumes the block type is "(i32,i32)->()"
    // 0d0008   nop             ;; <--------------------------------------------\ carries operands 13 and 17 and
    // 0d0010   block(0)        ;; assumes the block type is "(i32)->()"        | jumps to here
    // 0d0018     imm_i32(11)   ;;                                              |
    // 0d0024     imm_i32(13)   ;;                                              |
    // 0d0032     imm_i32(17)   ;;                                              ^
    // 0d0040     nop           ;;                                              |
    // 0d0042     recur(1,34)   ;; (34 = 42 - 8) ---------------->--------------/
    // 0d0050   end             ;;
    // 0d0052 end               ;;
    // ```
    //
    // Parameter `start_inst_offset` is the address offset to the next instruction after "block". It is calculated as:
    // (address of "recur" - address of "block" + length of the "block" instruction).
    //
    // (param layers:i16 start_inst_offset:i32) -> NO_RETURN
    recur,

    // The "block_alt" instruction is similar to the "block" instruction. It creates a new block scope
    // and a block stack frame. However, it jumps to the **next** instruction following the "break_alt"
    // instruction if the operand at the top of the stack equals ZERO (i.e., logical FALSE).
    //
    // For example, the following C code can be translated into bytecode:
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
    //                          ;; TRUE        FALSE
    //                          ;; v           v
    // 0d0998 nop               ;; |+          |+
    // 0d1000 block_alt(0,158)  ;; |+          \-->--\+ <-- jump to 0d1158 when FALSE
    // 0d1008 ...               ;; |+                |-     (158 = 1158 - 1000)
    // ;; the 'then' part       ;; |+                |-
    // 0d1150 break_alt(202)    ;; \-->--\+          |-
    // 0d1158 ...               ;;       |-    /--<--/+
    // ;; the 'else' part       ;;       |-    |+
    // 0d1350 end               ;;       |-    |+
    // 0d1352 nop               ;; <-----/+    |+
    //                          ;;
    //                          ;;       ^
    //                          ;;       |
    //                          ;; jump to 0d1352, (202 = 1352 - 1150)
    //                          ;;
    //                          ;; legend: "+" for execute, "-" for skip.
    // ```
    //
    // The "block_alt" instruction is used to construct "if...else..." control flow structures.
    // Since there are branches within its scope, it should not have input parameters or local variables.
    // However, this instruction still has parameters `type_index` and `local_variable_list_index`,
    // leaving the user with a choice.
    //
    // (param type_index:i32 local_variable_list_index:i32 next_inst_offset:i32) -> NO_RETURN
    block_alt,

    // The "break_alt" instruction is used to exit the current "block_alt" scope.
    //
    // It can only exist within the scope of a "block_alt" instruction.
    // It is equivalent to the instruction `break 0, next_inst_offset`.
    //
    // (param next_inst_offset:i32) -> NO_RETURN
    break_alt,

    // The "block_nez" instruction creates a block scope only if the operand at the top of the operand stack
    // is **not** equal to ZERO (i.e., logical TRUE).
    //
    // The `next_inst_offset` parameter specifies the address of the instruction immediately following the "end" instruction.
    //
    // The "block_nez" instruction is commonly used to construct "if" control flow which without "else" structures.
    //
    // For example, the following C code can be translated into bytecode:
    //
    // ```c
    // if (i != 0) {
    //     ...
    // }
    // ```
    //
    // ```bytecode
    // 0d1000 block_nez(0,102)  ;; -----\
    // ....                     ;;      |
    // 0d1100 end               ;;      |
    // 0d1102 nop               ;; <----/ jump to here when FALSE
    //                          ;;        (102 = 1102 - 1000)
    // ```
    //
    // Unlike the "block" instruction, although the "block_nez" instruction also creates a block,
    // it should not have parameters or return values (i.e., the type should be `()->()`).
    // Therefore, this instruction does not include the `type_index` parameter.
    //
    // However, the instruction supports local variables, so it includes the `local_variable_list_index` parameter.
    //
    // (param local_variable_list_index:i32 next_inst_offset:i32) NO_RETURN
    block_nez,

    // TCO (Tail Call Optimization)
    // ----------------------------
    // The "recur" instruction is also used to implement Tail Call Optimization (TCO).
    //
    // Consider the following function:
    //
    // ```rust
    // /* Calculate '3 + 2 + 1', the result should be '6'. */
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
    // When the function is called with parameters `(0, 3)`, the call stack is:
    //
    // call
    // (0,3)--\                call               call               call
    //        |    /-----\     (3,2)  /-----\     (5,1)  /-----\     (6,0)  /-----\
    //        \--> | +   |   /------> | +   |   /------> | +   |   /------> | +   |
    //             |=====|   |        |=====|   |        |=====|   |        |=====|
    //             | --> |   |        | --> |   |        | --> |   |        | --> | -----\
    //             | <-- |   |        | <-- |   |        | <-- |   |  /---- | <-- | <----/
    //             |=====|   |        |=====|   |        |=====|   |  |     |=====|
    //             | --> | --/        | --> | --/        | --> | --/  |     | --> |
    //        /--- | <-- | <--------- | <-- | <--------- | <-- | <----/     | <-- |
    //        |    \-----/  return    \-----/  return    \-----/  return    \-----/
    // 6 <----/
    // return
    //
    // The function "accumulate" is executed 4 times, and 4 function stack frames are created.
    // Since there is no other operation after the statement `accumulate(new_sum, number - 1)`,
    // and only one value is returned afterward, the call stack can be optimized as:
    //
    // call
    // (0,3)--\                recur              recur              recur
    //        |    /-----\     (3,2)  /-----\     (5,1)  /-----\     (6,0)  /-----\
    //        \--> | +   |   /------> | +   |   /------> | +   |   /------> | +   |
    //             |=====|   |        |=====|   |        |=====|   |        |=====|
    //             | --> |   |        | --> |   |        | --> |   |        | --> | ---\
    //             | <-- |   |        | <-- |   |        | <-- |   |    /-- | <-- | <--/
    //             |=====|   |        |=====|   |        |=====|   |    |   |=====|
    //             | --> | --/        | --> | --/        | --> | --/    |   | --> |
    //             | <-- |            | <-- |            | <-- |        |   | <-- |
    //             \-----/            \-----/            \-----/        |   \-----/
    // 6 <--------------------------------------------------------------/
    // return
    //
    // This optimization is known as TCO (Tail Call Optimization). It eliminates the need to create
    // and destroy call stack frames, saving resources and improving performance.
    //
    // An important prerequisite for TCO is that the "self-call" statement
    // (in general programming languages) must be the last statement of the function or branch.
    // Otherwise, a logical error will occur. For example:
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
    // Expanding the statement `number * factorial(number - 1)` results in:
    //
    // ```rust
    // let i = number;
    // let j = factorial(number - 1);
    // i * j
    // ```
    //
    // Clearly, the statement `factorial(number - 1)` is neither the last statement
    // of the function nor the last statement of the branch. The last statement
    // is `i * j`, so this code cannot apply TCO directly.
    //
    // However, we can modify this function to make it TCO-compatible, for example:
    //
    // ```rust
    // /* Calculate '5 * 4 * 3 * 2 * 1', the result should be '120'. */
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

    // Implements Control Flow Structures
    // ----------------------------------
    //
    // ## Branch
    //
    // | IR                | Assembly          | Instructions       |
    // |-------------------|-------------------|--------------------|
    // |                   |                   | ..a..              |
    // | when ..a..        | when (a)          | block_nez -\       |
    // | then ..b..        |      (b)          |   ..b..    |       |
    // |                   |                   | end        |       |
    // |                   |                   | ...    <---/       |
    // |-------------------|-------------------|--------------------|
    // |                   |                   | ..a..              |
    // | if ..a..          | if (a)            | block_alt ---\     |
    // | then ..b..        |    (b)            |   ..b..      |     |
    // | else ..c..        |    (c)            |   break_alt -|-\   |
    // |                   |                   |   ..c..  <---/ |   |
    // |                   |                   | end            |   |
    // |                   |                   | ...      <-----/   |
    // |-------------------|-------------------|--------------------|
    // |                   |                   | ..a..              |
    // | if ..a..          | if (a)            | block_alt ---\     |
    // | then ..b..        |    (b)            |   ..b..      |     |
    // | else if ..c..     |     if (c)        |   break_alt--|---\ |
    // |    then ..d..     |        (d)        |   ..c..  <---/   | |
    // |    else ..e..     |        (e)        |   block_alt --\  | |
    // |                   |                   |     ..d..     |  | |
    // |                   |                   |     break_alt-|-\| |
    // |                   |                   |     ..e..  <--/ || |
    // |                   |                   |   end           || |
    // |                   |                   | end        <----/| |
    // |                   |                   | ...        <-----/ |
    // |                   |                   |                    |
    // | ----------------- | ----------------- | ------------------ |
    // | match (v) {       |                   | block              |
    // |   case ..a..:     |                   |   ..a..            |
    // |        ..b..      |                   |   block_nez -\     |
    // |   case ..c..:     |                   |     ..b..    |     |
    // |        ..d..:     |                   |     break 1 -|--\  |
    // |   default:        |                   |   end        |  |  |
    // |        ..e..      |                   |   ..c..  <---/  |  |
    // | }                 |                   |   block_nez -\  |  |
    // |                   |                   |     ..d..    |  |  |
    // |                   |                   |     break 1 -|--|  |
    // |                   |                   |   end        |  |  |
    // |                   |                   |   ..e..  <---/  |  |
    // |                   |                   | end             |  |
    // |                   |                   | ...        <----/  |
    // |-------------------|-------------------|--------------------|
    //
    // ## Loop
    //
    // | IR                | Assembly          | Instructions       |
    // |-------------------|-------------------|--------------------|
    // | for(let v=) {     | block {           | block              |
    // |   ...             |   ...             |   ...      <---\   |
    // |   when (a)        |   when (a)        |   ..a..        |   |
    // |   then recur      |   recur           |   block_nez    |   |
    // | }                 | }                 |     recur 1 ---/   |
    // |                   |                   |   end              |
    // |                   |                   | end                |
    // |                   |                   |                    |
    // |                   |                   |                    |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    //
    // ## Break
    //
    // | IR                | Assembly          | Instructions       |
    // |-------------------|-------------------|--------------------|
    // | for(let v=) {     | block {           | block              |
    // |    ...            |    ...            |   ...        <---\ |
    // |    when ..a..     |    when (a)       |   ..a..          | |
    // |    then break     |    break          |   block_nez      | |
    // |    ...            |    ...            |     break 1 ---\ | |
    // |    recur          |    recur()        |   end          | | |
    // | }                 | }                 |   ...          | | |
    // |                   |                   |   recur 0  ----|-/ |
    // |                   |                   | end            |   |
    // |                   |                   | ...      <-----/   |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    // | fn foo() {        | fn foo()          | -- func begin --   |
    // |    ...            |   ...             |   ...              |
    // |    when ..a..     |   when (a)        |   ..a..            |
    // |    then return    |   break_fn        |   block_nez        |
    // |    ...            |   ...             |     break 1  ---\  |
    // | }                 | }                 |   end           |  |
    // |                   |                   |   ...           |  |
    // |                   |                   | end         <---/  |
    // |                   |                   |                    |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    //
    // ## TCO
    //
    // | IR                | Assembly          | instructions       |
    // |-------------------|-------------------|--------------------|
    // | fn foo() {        | fn foo() {        | -- func begin --   |
    // |    ...            |    ...            |   ...   <-------\  |
    // |    when ..a..     |    when (a) {     |   ..a..         |  |
    // |    then {         |      ...          |   block_nez --\ |  |
    // |      ...          |      recur_fn()   |     ...       | |  |
    // |      tailcall()   |    }              |     recur 1 --|-/  |
    // |    }              | }                 |   end         |    |
    // | }                 |                   | end      <----/    |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    // | fn foo() {        | fn foo() {        | -- func begin --   |
    // |   if ..a..        |   if (a)          |   ..a.. <------\   |
    // |   then ..b..      |      (b)          |   block_alt -\ |   |
    // |   else {          |      {            |     ..b..    | |   |
    // |     ..c..         |        c          |     brk_alt -|-|-\ |
    // |     tailcall()    |        recur_fn() |     ..c.. <--/ | | |
    // |   }               |      }            |     recur 1 ---/ | |
    // | }                 | }                 |   end            | |
    // |                   |                   | end         <----/ |
    // |                   |                   |                    |
    // |-------------------|-------------------|--------------------|
    //

    // General Function Call
    //
    // (param function_public_index:i32) (operand args...) -> (values)
    call,

    // Note about the `function_public_index`
    // --------------------------------------
    //
    // The `function_public_index` is a unified index that includes both imported and internal functions.
    // Its value is calculated as:
    // (number of imported functions + internal function index).

    // Dynamic Function Call
    //
    // The "call_dynamic" instruction is used to call a function specified at runtime.
    //
    // In the default VM implementation, the "anonymous functions" (closures) in the XiaoXuan Core language
    // are implemented using dynamic function calls. When passing a regular or anonymous function as a parameter
    // to another function, a pointer to a struct called `closure_function_item` is passed:
    //
    // ```rust
    // struct closure_function_item {
    //     function_public_index: i32
    //     captured_data_pointer: i64
    // }
    // ```
    //
    // The `captured_data_pointer` points to a dynamically created structure containing the data captured by the function.
    // For example, if an anonymous function captures an `i32` and a string, the captured data structure might look like:
    //
    // ```rust
    // struct captured_data_1 {
    //     value_0: i32,
    //     value_1: i64,
    // }
    // ```
    //
    // The target function can be either an anonymous function or a regular function. If the target is an anonymous function,
    // the compiler automatically appends an additional parameter for the captured data pointer. For example:
    //
    // ```rust
    // let a = fn (a: i32, b: i32) { ... };
    // ```
    //
    // This will be compiled as:
    //
    // ```rust
    // let a = fn (a: i32, b: i32, captured_data_pointer: i64) { ... };
    // ```
    //
    // Example of passing an anonymous function to the `filter` function:
    //
    // ```text
    //                              /--> function_public_index --> fn (a, b, captured_data_pointer) {...}
    //                         /--->|
    //                         |    \--> captured_data_pointer -> captured_data_1
    //                         |
    // let a = filter(list, predicate)
    // ```
    //
    // If the target function is a regular function, the compiler generates a `closure_function_item` structure
    // and a wrapper function. For example:
    //
    // ```text
    //                              /--> function_public_index --> fn wrapper (a, b, captured_data_pointer) --> fn original (a, b)
    //                         /--->|
    //                         |    \--> captured_data_pointer -> nullptr
    //                         |
    // let a = filter(list, predicate)
    // ```
    //
    // () (operand args... function_module_index:i32 function_public_index:i32) -> (values)
    call_dynamic,

    // Environment Function Call
    //
    // The "envcall" instruction is used to call VM built-in functions, such as retrieving environment variables,
    // obtaining runtime information, manipulating threads, etc.
    //
    // (param envcall_num:i32) (operand args...) -> (values)
    envcall,

    // System Call
    //
    // The "syscall" instruction invokes a system call on Unix-like operating systems.
    //
    // The syscall arguments must be pushed onto the stack first, followed by the syscall number, and the number of parameters.
    //
    // For example:
    //
    // | params_count   | <-- stack end
    // | syscall_num    |
    // | arg6           |
    // | arg5           |
    // | arg4           |
    // | arg3           |
    // | arg2           |                     | error number   |
    // | arg1           |    -- returns -->   | return value   |
    // | ...            |                     | ...            |
    // \----------------/ <-- stack start --> \----------------/
    //
    // When the syscall completes, the return value is stored in the "rax" register. If the operation fails,
    // the value is negative (i.e., rax < 0).
    //
    // Note: Unlike the C standard library, there is no "errno" when calling syscalls directly from assembly.
    //
    // () (operand args... params_count:i32 syscall_num:i32) -> (return_value:i64 error_number:i32)
    syscall,

    // External Function Call
    //
    // The "extcall" instruction is used to call external functions.
    //
    // Note: Both the "syscall" and "extcall" instructions are optional and may not be available in some environments.
    // The supported VM features can be queried using the "envcall" instruction with the call number `runtime_features`.
    //
    // (param external_function_index:i32) (operand args...) -> return_value:void/i32/i64/f32/f64
    extcall,

    // Category: Memory
    // -----------------

    // Allocate a new memory chunk and return a data public index.
    //
    // Notes:
    // - The index of the memory chunk is not necessarily sequential.
    // - Both alignment and size must be multiples of 8.
    // - If the `align` parameter is 0, the default value of 8 is used.
    // - The `module_index` of allocated memory is always 0.
    //
    // () (operand align_in_bytes:i16 size_in_bytes:i64) -> i32
    memory_allocate = 0x0a_00,

    // Resize an existing memory chunk.
    //
    // () (operand data_public_index:i32 new_size_in_bytes:i64) -> i32
    memory_resize,

    // Free an existing memory chunk.
    //
    // () (operand data_public_index:i32) -> ()
    memory_free,

    // Fill a memory chunk with a specified value.
    //
    // () (operand
    // data_module_index:i32 data_public_index:i32 offset_in_bytes:i64
    // size_in_bytes:i64 value:i8) -> ()
    memory_fill,

    // Copy a memory chunk from one location to another.
    //
    // Note: The source and destination memory chunks must not overlap.
    //
    // () (operand
    // source_data_module_index:i32 source_data_public_index:i32 source_offset_in_bytes:i64
    // dest_data_module_index:i32 dest_data_public_index:i32 dest_offset_in_bytes:i64
    // size_in_bytes:i64 value:i8) -> ()
    memory_copy,

    // Category: Machine
    // ------------------

    // Terminates the current process (program) immediately.
    // This is generally used in cases where an unrecoverable error is encountered.
    //
    // (param terminate_code:i32) -> NERVER_RETURN
    terminate,

    // Pushes the module index and function public index onto the operand stack.
    //
    // (param function_public_index:i32) -> (function_module_index:i32 function_public_index:i32)
    get_function = 0x0b_00,

    // Pushes the module index and data public index onto the operand stack.
    //
    // (param data_public_index:i32) -> (data_module_index:i32 data_public_index:i32)
    get_data,

    // Creates a native function that wraps a VM function, allowing the host side or
    // external libraries to call the VM function.
    //
    // Notes:
    // - A "bridge callback function" (a native function) is created when this instruction is executed.
    // - The body of the "bridge callback function" is generated via JIT codegen.
    // - The specified VM function is added to the "bridge callback function table" to prevent duplicate creation.
    //
    // (param function_public_index:i32) -> pointer
    host_addr_function,

    // () (operand function_module_index:i32 function_public_index:i32) -> pointer
    host_addr_function_dynamic,

    // Retrieves the memory address of VM data.
    //
    // Accessing VM data using host-side memory addresses is unsafe, but these addresses
    // are necessary for interacting with external functions.
    //
    // Note:
    // - The host-side address of local variables is only valid within the scope of the current function
    //   and its sub-functions. Once a function finishes, its stack frame (and local variables) is destroyed or modified.
    //
    // |                      | by index | by host address    |
    // |----------------------|----------|--------------------|
    // | local variables      | safe     | limited and unsafe |
    // |----------------------|----------|--------------------|
    // | read-only data       |          |                    |
    // | read-write data      | safe     | unsafe             |
    // | uninitilized data    |          |                    |
    // | dynamic alloc memory |          |                    |
    //
    //
    host_addr_data,        // (param offset_bytes:i16 data_public_index:i32) -> pointer
    host_addr_data_extend, // (param data_public_index:i32) (operand offset_bytes:i64) -> pointer
    host_addr_data_dynamic, // () (operand module_index:i32 data_public_index:i32 offset_bytes:i64) -> pointer
}

impl Opcode {
    pub fn get_name(&self) -> &'static str {
        match self {
            // Category: Fundamental
            Opcode::nop => "nop",
            Opcode::imm_i32 => "imm_i32",
            Opcode::imm_i64 => "imm_i64",
            Opcode::imm_f32 => "imm_f32",
            Opcode::imm_f64 => "imm_f64",
            // Category: Local Variables
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
            // Category: Data
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
            Opcode::data_load_extend_i64 => "data_load_extend_i64",
            Opcode::data_load_extend_i32_s => "data_load_extend_i32_s",
            Opcode::data_load_extend_i32_u => "data_load_extend_i32_u",
            Opcode::data_load_extend_i16_s => "data_load_extend_i16_s",
            Opcode::data_load_extend_i16_u => "data_load_extend_i16_u",
            Opcode::data_load_extend_i8_s => "data_load_extend_i8_s",
            Opcode::data_load_extend_i8_u => "data_load_extend_i8_u",
            Opcode::data_load_extend_f64 => "data_load_extend_f64",
            Opcode::data_load_extend_f32 => "data_load_extend_f32",
            Opcode::data_store_extend_i64 => "data_store_extend_i64",
            Opcode::data_store_extend_i32 => "data_store_extend_i32",
            Opcode::data_store_extend_i16 => "data_store_extend_i16",
            Opcode::data_store_extend_i8 => "data_store_extend_i8",
            Opcode::data_store_extend_f64 => "data_store_extend_f64",
            Opcode::data_store_extend_f32 => "data_store_extend_f32",
            Opcode::data_load_dynamic_i64 => "data_load_dynamic_i64",
            Opcode::data_load_dynamic_i32_s => "data_load_dynamic_i32_s",
            Opcode::data_load_dynamic_i32_u => "data_load_dynamic_i32_u",
            Opcode::data_load_dynamic_i16_s => "data_load_dynamic_i16_s",
            Opcode::data_load_dynamic_i16_u => "data_load_dynamic_i16_u",
            Opcode::data_load_dynamic_i8_s => "data_load_dynamic_i8_s",
            Opcode::data_load_dynamic_i8_u => "data_load_dynamic_i8_u",
            Opcode::data_load_dynamic_f64 => "data_load_dynamic_f64",
            Opcode::data_load_dynamic_f32 => "data_load_dynamic_f32",
            Opcode::data_store_dynamic_i64 => "data_store_dynamic_i64",
            Opcode::data_store_dynamic_i32 => "data_store_dynamic_i32",
            Opcode::data_store_dynamic_i16 => "data_store_dynamic_i16",
            Opcode::data_store_dynamic_i8 => "data_store_dynamic_i8",
            Opcode::data_store_dynamic_f64 => "data_store_dynamic_f64",
            Opcode::data_store_dynamic_f32 => "data_store_dynamic_f32",
            // Category: Arithmetic
            Opcode::add_i32 => "add_i32",
            Opcode::sub_i32 => "sub_i32",
            Opcode::add_imm_i32 => "add_imm_i32",
            Opcode::sub_imm_i32 => "sub_imm_i32",
            Opcode::mul_i32 => "mul_i32",
            Opcode::div_i32_s => "div_i32_s",
            Opcode::div_i32_u => "div_i32_u",
            Opcode::rem_i32_s => "rem_i32_s",
            Opcode::rem_i32_u => "rem_i32_u",
            Opcode::add_i64 => "add_i64",
            Opcode::sub_i64 => "sub_i64",
            Opcode::add_imm_i64 => "add_imm_i64",
            Opcode::sub_imm_i64 => "sub_imm_i64",
            Opcode::mul_i64 => "mul_i64",
            Opcode::div_i64_s => "div_i64_s",
            Opcode::div_i64_u => "div_i64_u",
            Opcode::rem_i64_s => "rem_i64_s",
            Opcode::rem_i64_u => "rem_i64_u",
            Opcode::add_f32 => "add_f32",
            Opcode::sub_f32 => "sub_f32",
            Opcode::mul_f32 => "mul_f32",
            Opcode::div_f32 => "div_f32",
            Opcode::add_f64 => "add_f64",
            Opcode::sub_f64 => "sub_f64",
            Opcode::mul_f64 => "mul_f64",
            Opcode::div_f64 => "div_f64",
            // Category: Bitwise
            Opcode::and => "and",
            Opcode::or => "or",
            Opcode::xor => "xor",
            Opcode::not => "not",
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
            // Category: Math
            Opcode::abs_i32 => "abs_i32",
            Opcode::neg_i32 => "neg_i32",
            Opcode::abs_i64 => "abs_i64",
            Opcode::neg_i64 => "neg_i64",
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
            // Category: Conversion
            Opcode::truncate_i64_to_i32 => "truncate_i64_to_i32",
            Opcode::extend_i32_s_to_i64 => "extend_i32_s_to_i64",
            Opcode::extend_i32_u_to_i64 => "extend_i32_u_to_i64",
            Opcode::demote_f64_to_f32 => "demote_f64_to_f32",
            Opcode::promote_f32_to_f64 => "promote_f32_to_f64",
            Opcode::convert_f32_to_i32_s => "convert_f32_to_i32_s",
            Opcode::convert_f32_to_i32_u => "convert_f32_to_i32_u",
            Opcode::convert_f64_to_i32_s => "convert_f64_to_i32_s",
            Opcode::convert_f64_to_i32_u => "convert_f64_to_i32_u",
            Opcode::convert_f32_to_i64_s => "convert_f32_to_i64_s",
            Opcode::convert_f32_to_i64_u => "convert_f32_to_i64_u",
            Opcode::convert_f64_to_i64_s => "convert_f64_to_i64_s",
            Opcode::convert_f64_to_i64_u => "convert_f64_to_i64_u",
            Opcode::convert_i32_s_to_f32 => "convert_i32_s_to_f32",
            Opcode::convert_i32_u_to_f32 => "convert_i32_u_to_f32",
            Opcode::convert_i64_s_to_f32 => "convert_i64_s_to_f32",
            Opcode::convert_i64_u_to_f32 => "convert_i64_u_to_f32",
            Opcode::convert_i32_s_to_f64 => "convert_i32_s_to_f64",
            Opcode::convert_i32_u_to_f64 => "convert_i32_u_to_f64",
            Opcode::convert_i64_s_to_f64 => "convert_i64_s_to_f64",
            Opcode::convert_i64_u_to_f64 => "convert_i64_u_to_f64",
            // Category: Comparison
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
            Opcode::eq_f32 => "eq_f32",
            Opcode::ne_f32 => "ne_f32",
            Opcode::lt_f32 => "lt_f32",
            Opcode::gt_f32 => "gt_f32",
            Opcode::le_f32 => "le_f32",
            Opcode::ge_f32 => "ge_f32",
            Opcode::eq_f64 => "eq_f64",
            Opcode::ne_f64 => "ne_f64",
            Opcode::lt_f64 => "lt_f64",
            Opcode::gt_f64 => "gt_f64",
            Opcode::le_f64 => "le_f64",
            Opcode::ge_f64 => "ge_f64",
            // Category: Control flow
            Opcode::end => "end",
            Opcode::block => "block",
            Opcode::break_ => "break",
            Opcode::recur => "recur",
            Opcode::block_alt => "block_alt",
            Opcode::break_alt => "break_alt",
            Opcode::block_nez => "block_nez",
            Opcode::call => "call",
            Opcode::call_dynamic => "call_dynamic",
            Opcode::envcall => "envcall",
            Opcode::syscall => "syscall",
            Opcode::extcall => "extcall",
            // Category: Memory
            Opcode::memory_allocate => "memory_allocate",
            Opcode::memory_resize => "memory_resize",
            Opcode::memory_free => "memory_free",
            Opcode::memory_fill => "memory_fill",
            Opcode::memory_copy => "memory_copy",
            // Category: Machine
            Opcode::terminate => "terminate",
            Opcode::get_function => "get_function",
            Opcode::get_data => "get_data",
            Opcode::host_addr_function => "host_addr_function",
            Opcode::host_addr_function_dynamic => "host_addr_function_dynamic",
            Opcode::host_addr_data => "host_addr_data",
            Opcode::host_addr_data_extend => "host_addr_data_extend",
            Opcode::host_addr_data_dynamic => "host_addr_data_dynamic",
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            // Category: Fundamental
            "nop" => Opcode::nop,
            "imm_i32" => Opcode::imm_i32,
            "imm_i64" => Opcode::imm_i64,
            "imm_f32" => Opcode::imm_f32,
            "imm_f64" => Opcode::imm_f64,
            // Category: Local Variables
            "local_load_i64" => Opcode::local_load_i64,
            "local_load_i32_s" => Opcode::local_load_i32_s,
            "local_load_i32_u" => Opcode::local_load_i32_u,
            "local_load_i16_s" => Opcode::local_load_i16_s,
            "local_load_i16_u" => Opcode::local_load_i16_u,
            "local_load_i8_s" => Opcode::local_load_i8_s,
            "local_load_i8_u" => Opcode::local_load_i8_u,
            "local_load_f64" => Opcode::local_load_f64,
            "local_load_f32" => Opcode::local_load_f32,
            "local_store_i64" => Opcode::local_store_i64,
            "local_store_i32" => Opcode::local_store_i32,
            "local_store_i16" => Opcode::local_store_i16,
            "local_store_i8" => Opcode::local_store_i8,
            "local_store_f64" => Opcode::local_store_f64,
            "local_store_f32" => Opcode::local_store_f32,
            // Category: Data
            "data_load_i64" => Opcode::data_load_i64,
            "data_load_i32_s" => Opcode::data_load_i32_s,
            "data_load_i32_u" => Opcode::data_load_i32_u,
            "data_load_i16_s" => Opcode::data_load_i16_s,
            "data_load_i16_u" => Opcode::data_load_i16_u,
            "data_load_i8_s" => Opcode::data_load_i8_s,
            "data_load_i8_u" => Opcode::data_load_i8_u,
            "data_load_f64" => Opcode::data_load_f64,
            "data_load_f32" => Opcode::data_load_f32,
            "data_store_i64" => Opcode::data_store_i64,
            "data_store_i32" => Opcode::data_store_i32,
            "data_store_i16" => Opcode::data_store_i16,
            "data_store_i8" => Opcode::data_store_i8,
            "data_store_f64" => Opcode::data_store_f64,
            "data_store_f32" => Opcode::data_store_f32,
            "data_load_extend_i64" => Opcode::data_load_extend_i64,
            "data_load_extend_i32_s" => Opcode::data_load_extend_i32_s,
            "data_load_extend_i32_u" => Opcode::data_load_extend_i32_u,
            "data_load_extend_i16_s" => Opcode::data_load_extend_i16_s,
            "data_load_extend_i16_u" => Opcode::data_load_extend_i16_u,
            "data_load_extend_i8_s" => Opcode::data_load_extend_i8_s,
            "data_load_extend_i8_u" => Opcode::data_load_extend_i8_u,
            "data_load_extend_f64" => Opcode::data_load_extend_f64,
            "data_load_extend_f32" => Opcode::data_load_extend_f32,
            "data_store_extend_i64" => Opcode::data_store_extend_i64,
            "data_store_extend_i32" => Opcode::data_store_extend_i32,
            "data_store_extend_i16" => Opcode::data_store_extend_i16,
            "data_store_extend_i8" => Opcode::data_store_extend_i8,
            "data_store_extend_f64" => Opcode::data_store_extend_f64,
            "data_store_extend_f32" => Opcode::data_store_extend_f32,
            "data_load_dynamic_i64" => Opcode::data_load_dynamic_i64,
            "data_load_dynamic_i32_s" => Opcode::data_load_dynamic_i32_s,
            "data_load_dynamic_i32_u" => Opcode::data_load_dynamic_i32_u,
            "data_load_dynamic_i16_s" => Opcode::data_load_dynamic_i16_s,
            "data_load_dynamic_i16_u" => Opcode::data_load_dynamic_i16_u,
            "data_load_dynamic_i8_s" => Opcode::data_load_dynamic_i8_s,
            "data_load_dynamic_i8_u" => Opcode::data_load_dynamic_i8_u,
            "data_load_dynamic_f64" => Opcode::data_load_dynamic_f64,
            "data_load_dynamic_f32" => Opcode::data_load_dynamic_f32,
            "data_store_dynamic_i64" => Opcode::data_store_dynamic_i64,
            "data_store_dynamic_i32" => Opcode::data_store_dynamic_i32,
            "data_store_dynamic_i16" => Opcode::data_store_dynamic_i16,
            "data_store_dynamic_i8" => Opcode::data_store_dynamic_i8,
            "data_store_dynamic_f64" => Opcode::data_store_dynamic_f64,
            "data_store_dynamic_f32" => Opcode::data_store_dynamic_f32,
            // Category: Arithmetic
            "add_i32" => Opcode::add_i32,
            "sub_i32" => Opcode::sub_i32,
            "add_imm_i32" => Opcode::add_imm_i32,
            "sub_imm_i32" => Opcode::sub_imm_i32,
            "mul_i32" => Opcode::mul_i32,
            "div_i32_s" => Opcode::div_i32_s,
            "div_i32_u" => Opcode::div_i32_u,
            "rem_i32_s" => Opcode::rem_i32_s,
            "rem_i32_u" => Opcode::rem_i32_u,
            "add_i64" => Opcode::add_i64,
            "sub_i64" => Opcode::sub_i64,
            "add_imm_i64" => Opcode::add_imm_i64,
            "sub_imm_i64" => Opcode::sub_imm_i64,
            "mul_i64" => Opcode::mul_i64,
            "div_i64_s" => Opcode::div_i64_s,
            "div_i64_u" => Opcode::div_i64_u,
            "rem_i64_s" => Opcode::rem_i64_s,
            "rem_i64_u" => Opcode::rem_i64_u,
            "add_f32" => Opcode::add_f32,
            "sub_f32" => Opcode::sub_f32,
            "mul_f32" => Opcode::mul_f32,
            "div_f32" => Opcode::div_f32,
            "add_f64" => Opcode::add_f64,
            "sub_f64" => Opcode::sub_f64,
            "mul_f64" => Opcode::mul_f64,
            "div_f64" => Opcode::div_f64,
            // Category: Bitwise
            "and" => Opcode::and,
            "or" => Opcode::or,
            "xor" => Opcode::xor,
            "not" => Opcode::not,
            "count_leading_zeros_i32" => Opcode::count_leading_zeros_i32,
            "count_leading_ones_i32" => Opcode::count_leading_ones_i32,
            "count_trailing_zeros_i32" => Opcode::count_trailing_zeros_i32,
            "count_ones_i32" => Opcode::count_ones_i32,
            "shift_left_i32" => Opcode::shift_left_i32,
            "shift_right_i32_s" => Opcode::shift_right_i32_s,
            "shift_right_i32_u" => Opcode::shift_right_i32_u,
            "rotate_left_i32" => Opcode::rotate_left_i32,
            "rotate_right_i32" => Opcode::rotate_right_i32,
            "count_leading_zeros_i64" => Opcode::count_leading_zeros_i64,
            "count_leading_ones_i64" => Opcode::count_leading_ones_i64,
            "count_trailing_zeros_i64" => Opcode::count_trailing_zeros_i64,
            "count_ones_i64" => Opcode::count_ones_i64,
            "shift_left_i64" => Opcode::shift_left_i64,
            "shift_right_i64_s" => Opcode::shift_right_i64_s,
            "shift_right_i64_u" => Opcode::shift_right_i64_u,
            "rotate_left_i64" => Opcode::rotate_left_i64,
            "rotate_right_i64" => Opcode::rotate_right_i64,
            // Category: Math
            "abs_i32" => Opcode::abs_i32,
            "neg_i32" => Opcode::neg_i32,
            "abs_i64" => Opcode::abs_i64,
            "neg_i64" => Opcode::neg_i64,
            "abs_f32" => Opcode::abs_f32,
            "neg_f32" => Opcode::neg_f32,
            "copysign_f32" => Opcode::copysign_f32,
            "sqrt_f32" => Opcode::sqrt_f32,
            "min_f32" => Opcode::min_f32,
            "max_f32" => Opcode::max_f32,
            "ceil_f32" => Opcode::ceil_f32,
            "floor_f32" => Opcode::floor_f32,
            "round_half_away_from_zero_f32" => Opcode::round_half_away_from_zero_f32,
            "round_half_to_even_f32" => Opcode::round_half_to_even_f32,
            "trunc_f32" => Opcode::trunc_f32,
            "fract_f32" => Opcode::fract_f32,
            "cbrt_f32" => Opcode::cbrt_f32,
            "exp_f32" => Opcode::exp_f32,
            "exp2_f32" => Opcode::exp2_f32,
            "ln_f32" => Opcode::ln_f32,
            "log2_f32" => Opcode::log2_f32,
            "log10_f32" => Opcode::log10_f32,
            "sin_f32" => Opcode::sin_f32,
            "cos_f32" => Opcode::cos_f32,
            "tan_f32" => Opcode::tan_f32,
            "asin_f32" => Opcode::asin_f32,
            "acos_f32" => Opcode::acos_f32,
            "atan_f32" => Opcode::atan_f32,
            "pow_f32" => Opcode::pow_f32,
            "log_f32" => Opcode::log_f32,
            "abs_f64" => Opcode::abs_f64,
            "neg_f64" => Opcode::neg_f64,
            "copysign_f64" => Opcode::copysign_f64,
            "sqrt_f64" => Opcode::sqrt_f64,
            "min_f64" => Opcode::min_f64,
            "max_f64" => Opcode::max_f64,
            "ceil_f64" => Opcode::ceil_f64,
            "floor_f64" => Opcode::floor_f64,
            "round_half_away_from_zero_f64" => Opcode::round_half_away_from_zero_f64,
            "round_half_to_even_f64" => Opcode::round_half_to_even_f64,
            "trunc_f64" => Opcode::trunc_f64,
            "fract_f64" => Opcode::fract_f64,
            "cbrt_f64" => Opcode::cbrt_f64,
            "exp_f64" => Opcode::exp_f64,
            "exp2_f64" => Opcode::exp2_f64,
            "ln_f64" => Opcode::ln_f64,
            "log2_f64" => Opcode::log2_f64,
            "log10_f64" => Opcode::log10_f64,
            "sin_f64" => Opcode::sin_f64,
            "cos_f64" => Opcode::cos_f64,
            "tan_f64" => Opcode::tan_f64,
            "asin_f64" => Opcode::asin_f64,
            "acos_f64" => Opcode::acos_f64,
            "atan_f64" => Opcode::atan_f64,
            "pow_f64" => Opcode::pow_f64,
            "log_f64" => Opcode::log_f64,
            // Category: Conversion
            "truncate_i64_to_i32" => Opcode::truncate_i64_to_i32,
            "extend_i32_s_to_i64" => Opcode::extend_i32_s_to_i64,
            "extend_i32_u_to_i64" => Opcode::extend_i32_u_to_i64,
            "demote_f64_to_f32" => Opcode::demote_f64_to_f32,
            "promote_f32_to_f64" => Opcode::promote_f32_to_f64,
            "convert_f32_to_i32_s" => Opcode::convert_f32_to_i32_s,
            "convert_f32_to_i32_u" => Opcode::convert_f32_to_i32_u,
            "convert_f64_to_i32_s" => Opcode::convert_f64_to_i32_s,
            "convert_f64_to_i32_u" => Opcode::convert_f64_to_i32_u,
            "convert_f32_to_i64_s" => Opcode::convert_f32_to_i64_s,
            "convert_f32_to_i64_u" => Opcode::convert_f32_to_i64_u,
            "convert_f64_to_i64_s" => Opcode::convert_f64_to_i64_s,
            "convert_f64_to_i64_u" => Opcode::convert_f64_to_i64_u,
            "convert_i32_s_to_f32" => Opcode::convert_i32_s_to_f32,
            "convert_i32_u_to_f32" => Opcode::convert_i32_u_to_f32,
            "convert_i64_s_to_f32" => Opcode::convert_i64_s_to_f32,
            "convert_i64_u_to_f32" => Opcode::convert_i64_u_to_f32,
            "convert_i32_s_to_f64" => Opcode::convert_i32_s_to_f64,
            "convert_i32_u_to_f64" => Opcode::convert_i32_u_to_f64,
            "convert_i64_s_to_f64" => Opcode::convert_i64_s_to_f64,
            "convert_i64_u_to_f64" => Opcode::convert_i64_u_to_f64,
            // Category: Comparison
            "eqz_i32" => Opcode::eqz_i32,
            "nez_i32" => Opcode::nez_i32,
            "eq_i32" => Opcode::eq_i32,
            "ne_i32" => Opcode::ne_i32,
            "lt_i32_s" => Opcode::lt_i32_s,
            "lt_i32_u" => Opcode::lt_i32_u,
            "gt_i32_s" => Opcode::gt_i32_s,
            "gt_i32_u" => Opcode::gt_i32_u,
            "le_i32_s" => Opcode::le_i32_s,
            "le_i32_u" => Opcode::le_i32_u,
            "ge_i32_s" => Opcode::ge_i32_s,
            "ge_i32_u" => Opcode::ge_i32_u,
            "eqz_i64" => Opcode::eqz_i64,
            "nez_i64" => Opcode::nez_i64,
            "eq_i64" => Opcode::eq_i64,
            "ne_i64" => Opcode::ne_i64,
            "lt_i64_s" => Opcode::lt_i64_s,
            "lt_i64_u" => Opcode::lt_i64_u,
            "gt_i64_s" => Opcode::gt_i64_s,
            "gt_i64_u" => Opcode::gt_i64_u,
            "le_i64_s" => Opcode::le_i64_s,
            "le_i64_u" => Opcode::le_i64_u,
            "ge_i64_s" => Opcode::ge_i64_s,
            "ge_i64_u" => Opcode::ge_i64_u,
            "eq_f32" => Opcode::eq_f32,
            "ne_f32" => Opcode::ne_f32,
            "lt_f32" => Opcode::lt_f32,
            "gt_f32" => Opcode::gt_f32,
            "le_f32" => Opcode::le_f32,
            "ge_f32" => Opcode::ge_f32,
            "eq_f64" => Opcode::eq_f64,
            "ne_f64" => Opcode::ne_f64,
            "lt_f64" => Opcode::lt_f64,
            "gt_f64" => Opcode::gt_f64,
            "le_f64" => Opcode::le_f64,
            "ge_f64" => Opcode::ge_f64,
            // Category: Control flow
            "end" => Opcode::end,
            "block" => Opcode::block,
            "break" => Opcode::break_,
            "recur" => Opcode::recur,
            "block_alt" => Opcode::block_alt,
            "break_alt" => Opcode::break_alt,
            "block_nez" => Opcode::block_nez,
            "call" => Opcode::call,
            "call_dynamic" => Opcode::call_dynamic,
            "envcall" => Opcode::envcall,
            "syscall" => Opcode::syscall,
            "extcall" => Opcode::extcall,
            // Category: Memory
            "memory_allocate" => Opcode::memory_allocate,
            "memory_resize" => Opcode::memory_resize,
            "memory_free" => Opcode::memory_free,
            "memory_fill" => Opcode::memory_fill,
            "memory_copy" => Opcode::memory_copy,
            // Category: Machine
            "terminate" => Opcode::terminate,
            "get_function" => Opcode::get_function,
            "get_data" => Opcode::get_data,
            "host_addr_function" => Opcode::host_addr_function,
            "host_addr_function_dynamic" => Opcode::host_addr_function_dynamic,
            "host_addr_data" => Opcode::host_addr_data,
            "host_addr_data_extend" => Opcode::host_addr_data_extend,
            "host_addr_data_dynamic" => Opcode::host_addr_data_dynamic,
            //
            _ => panic!("Unknown instruction \"{}\".", name),
        }
    }
}
