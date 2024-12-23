// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

pub mod opcode;

use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

pub const RUNTIME_CODE_NAME: &[u8; 6] = b"Selina"; // is also my lovely daughter's name (XiaoXuan for zh-Hans) :D

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct EffectiveVersion {
    pub major: u16,
    pub minor: u16,
}

impl EffectiveVersion {
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

impl Display for EffectiveVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

// Semantic Versioning
// - https://semver.org/
//
// an application will only run if its required major and minor
// versions match the current runtime version strictly.
pub const RUNTIME_MAJOR_VERSION: u16 = 1;
pub const RUNTIME_MINOR_VERSION: u16 = 0;
pub const RUNTIME_PATCH_VERSION: u16 = 0;

// the max version number the current runtime supported
pub const IMAGE_FORMAT_MAJOR_VERSION: u16 = 1;
pub const IMAGE_FORMAT_MINOR_VERSION: u16 = 0;

// the relationship between the version of application, shared modules and runtime
// ----------------------------------------------------------------------------
//
// for applications:
//
// every application declares a desired runtime version, which can only be run
// when the major and minor versions are identical. in short:
//
// - app required runtime version major == runtime version major
// - app required runtime version minor == runtime version minor
//
// for shared modules:
//
// shared modules do not declare desired runtime version, since it is
// not a standalone executable module.
// when a shared module is referenced (as dependency) by other
// application, it will be compiled to the same runtime version as the main module requires.
//
// - shared module compiled version major == app required runtime version major
// - shared module compiled version minor == app required runtime version minor
//
// dependencies
// ------------
//
// a application (or shared module) may depend on one or more other shared modules,
// when a application references a shared module, it is necessary to declare the
// major and minor version of the shared module.
//
// - dependency declare version major == shared module version major
// - dependency declare version minor == shared module version minor
//
// version conflicts
// -----------------
//
// If a shared module appears multiple times in the dependency tree with
// different versions and the major version numbers differ, the compiler
// will complain. However, if the major version numbers are the same, the
// highest minor version wil be selected.
//
// Note that this implies that in the actual application runtime, the minor
// version of a module might be higher than what the application explicitly
// declares. This is permissible because minor version updates are expected to
// maintain backward compatibility.
//
// For instance, if an application depends on a module with version 1.4.0, the
// actual runtime version of that module could be anywhere from 1.4.0 to 1.99.99.
//
// For the local and remote file-base shared modules and libraries,
// because they lack version information, if their sources
// (e.g., file paths or URLs) do not match, the compilation will fail.
//
// zero major version
// ------------------
// When a shared module is in beta stage, the major version number can
// be set to zero.
// A zero major version indicates that each minor version is incompatible. If an
// application's dependency tree contains minor version inconsistencies in modules
// with a zero major version, compilation will fail.
//
// note to the author of shared module
// -----------------------------------
//
// it is important to note that the public interface (i.e., API) of
// a shared module MUST REMAIN UNCHANGED throughout the major versions release.
// for example:
// - the API of version 1.9 and 1.1 should be the same, the newer may add interfaces,
//   but the existing interfaces should NOT be changed or removed.
// - the API of version 1.9 and 2.0 may be different.

/// the raw data type of operands
pub type Operand = [u8; 8];
pub const OPERAND_SIZE_IN_BYTES: usize = 8;

/// the data type of
/// - function parameters and results
/// - the operand of instructions
///
/// note that 'i32' here means a 32-bit integer, which is equivalent to
/// the 'uint32_t' in C or 'u32' in Rust. do not confuse it with 'i32' in Rust.
/// the same applies to the i8, i16 and i64.
///
/// the function `std::mem::transmute` can be used for converting data type
/// between `enum` and `u8` date, e.g.
///
/// ```rust
/// use anc_isa::OperandDataType;
/// let a = unsafe { std::mem::transmute::<OperandDataType, u8>(OperandDataType::F32) };
/// assert_eq!(a, 2);
/// ```
///
/// it can be also done through 'union', e.g.
///
/// ```rust
/// use anc_isa::OperandDataType;
/// union S2U {
///     src: OperandDataType,
///     dst: u8
/// }
///
/// let a = unsafe{ S2U { src: OperandDataType::F32 }.dst };
/// assert_eq!(a, 2);
/// ```
///
/// or, add `#[repr(u8)]` notation for converting enum to u8.
///
/// ```rust
/// use anc_isa::OperandDataType;
/// let a = OperandDataType::F32 as u8;
/// assert_eq!(a, 2);
/// ```
///
#[repr(u8)]
// https://doc.rust-lang.org/nomicon/other-reprs.html
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OperandDataType {
    I32 = 0x0,
    I64,
    F32,
    F64,
}

/// the data type of
/// - local variables
/// - data in the DATA sections and heap
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MemoryDataType {
    I32 = 0x0,
    I64,
    F32,
    F64,
    Bytes,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataSectionType {
    ReadOnly = 0x0, // .rodata
    ReadWrite,      // .data
    Uninit,         // .bss
}

impl Display for OperandDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperandDataType::I64 => f.write_str("i64"),
            OperandDataType::I32 => f.write_str("i32"),
            OperandDataType::F64 => f.write_str("f64"),
            OperandDataType::F32 => f.write_str("f32"),
        }
    }
}

impl Display for MemoryDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryDataType::I64 => f.write_str("i64"),
            MemoryDataType::I32 => f.write_str("i32"),
            MemoryDataType::F64 => f.write_str("f64"),
            MemoryDataType::F32 => f.write_str("f32"),
            MemoryDataType::Bytes => f.write_str("byte[]"),
        }
    }
}

impl Display for DataSectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            DataSectionType::ReadOnly => "read_only",
            DataSectionType::ReadWrite => "read_write",
            DataSectionType::Uninit => "uninit",
        };
        f.write_str(name)
    }
}

// values for foreign function interface (FFI)
//
// it is used for calling VM functions from the outside,
// or returning values to the foreign caller.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ForeignValue {
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
}

impl ForeignValue {
    pub fn as_u32(&self) -> u32 {
        if let ForeignValue::U32(v) = self {
            *v
        } else {
            panic!("Not an u32.")
        }
    }

    pub fn as_u64(&self) -> u64 {
        if let ForeignValue::U64(v) = self {
            *v
        } else {
            panic!("Not an u64.")
        }
    }

    pub fn as_f32(&self) -> f32 {
        if let ForeignValue::F32(v) = self {
            *v
        } else {
            panic!("Not a f32.")
        }
    }

    pub fn as_f64(&self) -> f64 {
        if let ForeignValue::F64(v) = self {
            *v
        } else {
            panic!("Not a f64.")
        }
    }
}

/// the type of dependent shared modules
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ModuleDependencyType {
    // module from local file system
    //
    // the value of this type is a path to a folder, e.g.
    //
    // modules: [
    //   "module_name": module::Local("~/myprojects/hello")
    // ]
    //
    // because of the lack of version information on the local file system,
    // this type of dependency can only be used as local development and testing.
    // DO NOT distribute modules containing this type of dependency to the
    // central repository, actually the compiler and runtime will
    // refuse to compile when a project tries to add a module containing
    // a local dependency via "Remote" and "Share".
    //
    // It's worth noting that the local module is recompiled at EVERY compilation.
    Local = 0x0,

    // module from a remote GIT repository
    //
    // the value of this type contains the Git repository url, commit (hash)
    // and path, e.g.
    //
    // modules: [
    //   "module_name": module::Remote(
    //     {
    //       url:"https://github.com/hemashushu/xiaoxuan-core-extension.git",
    //       revision="commit or tag",
    //       path="/modules/sha2"
    //     })
    // ]
    //
    // when a project is compiled or run, the remote resource is
    // downloaded first, and then cached in a local directory.
    //
    // note that the normal HTTP web service is not suitable for
    // remote modules bacause of the lack of version information.
    Remote,

    // module from the central repository
    //
    // the runtime specifies a default location as the
    // "shared modules repository", which is a Git repo
    // that provides the module index.
    //
    // users can also customize a different location or add
    // multiple repository in the runtime settings.
    //
    // the value of this type contains the version and an optional
    // repository name, e.g.
    //
    // modules:[
    //   "module_name": module::Share(
    //     {
    //       repository_name: "...",
    //       version: {major:M, minor:N}
    //     })
    // ]
    //
    // this type of module is downloaded and cached to a local directory, e.g.
    //
    // "{/usr/lib, ~/.local/lib}/anc/VER/modules/modname/VER"
    //
    // by default there are 2 central repositories:
    // - default
    // - default-mirror
    Share,

    // module that comes with the runtime
    //
    // this type of module is located locally in a directory, e.g.
    //
    // "{/usr/lib, C:/Program Fiels}/anc/VER/runtime/modules/modname"
    //
    // there is no value of this type because the module name is specified
    // in the configuration, e.g.
    //
    // modules:[
    //   "module_name": module::Runtime
    // ]
    Runtime,

    // this type is for assembler, linker and interpreter use only,
    // it represents the current module.
    // users cannot configure modules of this type.
    //
    // modules:[
    //   "module": module::Current
    // ]
    //
    // Under the hood
    // --------------
    // When generating a "object module", the assembler adds a dependency of
    // this type, so the linker can import functions and data from other submodules under
    // the same module.
    Current,
}

/// the type of dependent libraries
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExternalLibraryDependencyType {
    // library from the local file system
    //
    // the dependency name is the library's soname removes "lib" prefix and
    // ".so.N" suffix.
    // the value of this type is a path to a file (with library so-name), e.g.
    //
    // libraries: [
    //    "hello": library::Local("~/myprojects/hello/lib/libhello.so.1")
    // ]
    //
    // notes that the format of "so-name" is "libfoo.so.MAJOR_VERSION_NUMBER",
    // do not confuse it with the "real-name" (e.g. "libfoo.so.MAJOR.MINOR") and the
    // "link name" (e.g. "libfoo.so").
    Local = 0x0,

    // library from a remote GIT repository
    //
    // e.g.
    //
    // libraries: [
    //   "lz4": library::Remote(
    //     {
    //       url:"https://github.com/hemashushu/xiaoxuan-core-extension.git",
    //       revision="commit/tag",
    //       path="/libraries/lz4/lib/liblz4.so.1"
    //     })
    // ]
    //
    // see also `ModuleDependentType::Remote`
    Remote,

    // library from the central repository
    //
    // an example of this type:
    //
    // libraries: [
    //   "zlib": library::Share(
    //     {
    //       repository_name: "...",
    //       version: {major:M, minor:N}
    //     })
    // ]
    //
    // this type of library is downloaded and cached to a local directory, e.g.
    // "{/usr/lib, ~/.local/lib}/anc/VER/libraries/libname/VER"
    Share,

    // library that comes with the runtime
    //
    // this type of module is located locally in a directory, e.g.
    //
    // "{/usr/lib, C:/Program Fiels}/anc/VER/libraries/libname/lib/libfile.so"
    //
    // libraries: [
    //   "zstd": library::Runtime
    // ]
    Runtime,

    // library from system
    //
    // the dependency name is the library's soname removes "lib" prefix and
    // ".so.N" suffix, e.g. "lz4", and the value is library's soname, e.g. "liblz4.so.1".
    //
    // e.g.
    // libraries: [
    //   "lz4": System("liblz4.so.1")
    // ]
    System,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "module")]
pub enum ModuleDependency {
    #[serde(rename = "local")]
    Local(Box<DependencyLocal>),

    #[serde(rename = "remote")]
    Remote(Box<DependencyRemote>),

    #[serde(rename = "share")]
    Share(Box<DependencyShare>),

    #[serde(rename = "runtime")]
    Runtime,

    #[serde(rename = "current")]
    Current,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "library")]
pub enum ExternalLibraryDependency {
    #[serde(rename = "local")]
    Local(Box<DependencyLocal>),

    #[serde(rename = "remote")]
    Remote(Box<DependencyRemote>),

    #[serde(rename = "share")]
    Share(Box<DependencyShare>),

    #[serde(rename = "runtime")]
    Runtime,

    #[serde(rename = "system")]
    System(/* the soname of library, e.g. libz.so.1 */ String),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "value")]
pub enum PropertyValue {
    #[serde(rename = "string")]
    String(String),

    #[serde(rename = "number")]
    Number(i64),

    #[serde(rename = "bool")]
    Bool(bool),

    #[serde(rename = "eval")]
    Eval(String),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "cond")]
pub enum DependencyCondition {
    #[serde(rename = "is_true")]
    IsTrue(String),

    #[serde(rename = "is_false")]
    IsFalse(String),

    #[serde(rename = "eval")]
    Eval(String),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "local")]
pub struct DependencyLocal {
    /// The module's path relative to the application (or module project) folder.
    /// It could also be a path of the file "*.so.VERSION" relative to the application
    /// if the dependency is external library.
    pub path: String,
    pub values: Option<HashMap<String, PropertyValue>>,
    pub condition: Option<DependencyCondition>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "remote")]
pub struct DependencyRemote {
    pub url: String,
    pub reversion: String, // commit or tag
    pub path: String,
    pub values: Option<HashMap<String, PropertyValue>>,
    pub condition: Option<DependencyCondition>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "share")]
pub struct DependencyShare {
    pub repository: Option<String>, // the name of repository
    pub version: String,            // e.g. "1.0"
    pub values: Option<HashMap<String, PropertyValue>>,
    pub condition: Option<DependencyCondition>,
}

impl Display for ExternalLibraryDependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExternalLibraryDependencyType::Local => f.write_str("local"),
            ExternalLibraryDependencyType::Remote => f.write_str("remote"),
            ExternalLibraryDependencyType::Share => f.write_str("share"),
            ExternalLibraryDependencyType::Runtime => f.write_str("runtime"),
            ExternalLibraryDependencyType::System => f.write_str("system"),
        }
    }
}

// the error in Rust
// -----------------
//
// sometimes you may want to get a specified type from 'dyn RuntimeError',
// there is a approach to downcast the 'dyn RuntimeError' object to a specified type, e.g.
//
// let some_error:T = unsafe {
//     &*(runtime_error as *const dyn RuntimeError as *const T)
// };
//
// the 'runtime_error' is a 'fat' pointer, it consists of a pointer to the data and
// a another pointer to the 'vtable'.
//
// BTW, the slice object is also a 'fat' pointer, e.g.
//
// let v:Vec<u8> = vec![1,2,3];
// let p_fat = &v[..] as *const _;     // this is a fat pointer
// let p_thin = p_fat as *const ();    // obtains the first pointer and discard the second pointer
// let addr = p_thin as usize;         // check the address in memory
//
// for simplicity, 'RuntimeError' may provides function 'as_any' for downcasing, e.g.
//
// let some_error = runtime_error
//     .as_any
//     .downcast_ref::<T>()
//     .expect("...");
//
// ref:
// - https://alschwalm.com/blog/static/2017/03/07/exploring-dynamic-dispatch-in-rust/
// - https://geo-ant.github.io/blog/2023/rust-dyn-trait-objects-fat-pointers/
// - https://doc.rust-lang.org/std/any/
// - https://bennett.dev/rust/downcast-trait-object/
//
// pub trait SomeError: Debug + Display + Send + Sync + 'static {
//     fn as_any(&self) -> &dyn Any;
// }
//
// pub type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        DependencyCondition, DependencyLocal, DependencyRemote, DependencyShare,
        ExternalLibraryDependency, ModuleDependency,
    };

    #[test]
    fn test_serialize_module_dependency() {
        assert_eq!(
            ason::to_string(&ModuleDependency::Local(Box::new(DependencyLocal {
                path: "~/myprojects/hello".to_owned(),
                values: None,
                condition: None
            })))
            .unwrap(),
            r#"module::local({
    path: "~/myprojects/hello"
    values: Option::None
    condition: Option::None
})"#
        );

        assert_eq!(
            ason::to_string(&ModuleDependency::Remote(Box::new(DependencyRemote {
                url: "https://github.com/hemashushu/xiaoxuan-core-extension.git".to_owned(),
                reversion: "v1.0.0".to_owned(),
                path: "/modules/sha2".to_owned(),
                values: None,
                condition: None,
            })))
            .unwrap(),
            r#"module::remote({
    url: "https://github.com/hemashushu/xiaoxuan-core-extension.git"
    reversion: "v1.0.0"
    path: "/modules/sha2"
    values: Option::None
    condition: Option::None
})"#
        );

        assert_eq!(
            ason::to_string(&ModuleDependency::Share(Box::new(DependencyShare {
                repository: Option::Some("default".to_owned()),
                version: "11.13".to_owned(),
                values: None,
                condition: Some(DependencyCondition::IsTrue("enable_me".to_owned()))
            })))
            .unwrap(),
            r#"module::share({
    repository: Option::Some("default")
    version: "11.13"
    values: Option::None
    condition: Option::Some(cond::is_true("enable_me"))
})"#
        );

        assert_eq!(
            ason::to_string(&ModuleDependency::Runtime).unwrap(),
            r#"module::runtime"#
        );
    }

    #[test]
    fn test_deserialize_external_library_dependency() {
        let s0 = r#"library::share({
            repository: Option::Some("default")
            version: "17.19"
        })"#;

        let v0: ExternalLibraryDependency = ason::from_str(s0).unwrap();
        assert_eq!(
            v0,
            ExternalLibraryDependency::Share(Box::new(DependencyShare {
                repository: Option::Some("default".to_owned()),
                version: "17.19".to_owned(),
                values: None,
                condition: None
            }))
        );

        let s1 = r#"library::remote({
            url: "https://github.com/hemashushu/xiaoxuan-core-extension.git"
            reversion: "v1.0.0"
            path: "/libraries/lz4/lib/liblz4.so.1"
            condition: Option::Some(cond::is_false("enable_me"))
        })"#;
        let v1: ExternalLibraryDependency = ason::from_str(s1).unwrap();
        assert_eq!(
            v1,
            ExternalLibraryDependency::Remote(Box::new(DependencyRemote {
                url: "https://github.com/hemashushu/xiaoxuan-core-extension.git".to_owned(),
                reversion: "v1.0.0".to_owned(),
                path: "/libraries/lz4/lib/liblz4.so.1".to_owned(),
                values: None,
                condition: Some(DependencyCondition::IsFalse("enable_me".to_owned()))
            }))
        );
    }
}
