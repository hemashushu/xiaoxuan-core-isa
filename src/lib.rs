// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

pub mod opcode;

use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

pub const RUNTIME_CODE_NAME: &[u8; 6] = b"Selina"; // is also my lovely daughter's name (XiaoXuan for zh-Hans) :D
pub const IMAGE_FILE_MAGIC_NUMBER: &[u8; 8] = b"ancmod\0\0"; // the abbr of "XiaoXuan Core Module"

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
// unlike many other language, 'XiaoXuan Core' application requires the version of the dependencies
// (shared modules) must be strictly consistent with the declaration, that is to say:
//
// - dependency declare version major == shared module version major
// - dependency declare version minor == shared module version minor
//
// version conflicts
// -----------------
//
// if a shared module is duplicated in the dependency tree with different versions,
// and the major version is different, the compiler will complain. otherwise,
// the version required by the application is preferred, or the maximum minor version
// is selected.
//
// for the local and remote shared modules and libraries (i.e. file base dependencies),
// because they lack version information, the file required by
// the application is preferred, or the first declear is selected.
//
// note to the author of shared module:
// it is important to note that the public interface (i.e., API) of
// a module MUST REMAIN UNCHANGED throughout the major release. for example,
// the API of version 1.9 and 1.1 should be the same (the newer may add
// interfaces, but the existing interfaces should NOT be changed or removed),
// but version 2.0 and 1.9 may be different.

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
    Raw,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataSectionType {
    ReadOnly = 0x0, // .rodata
    ReadWrite,      // .data
    Uninit,         // .bss
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
pub enum ModuleDependentType {
    // module from local file system
    //
    // the value of this type is a path to a folder, e.g.
    //
    // {
    //   name: "..."
    //   value: Local("~/myprojects/hello")
    // }
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
    // the value of this type contains the Git repository url, commit
    // and path, e.g.
    //
    // {
    //   name: "..."
    //   value: Remote(
    //     {
    //       url:"https://github.com/hemashushu/xiaoxuan-core-extension.git",
    //       revision="commit/tag",
    //       path="/modules/sha2"
    //     })
    // }
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
    // the value of this type is:
    // {
    //   name: "..."
    //   value: Share(
    //     {
    //       repository_name: "...",
    //       version: {major:M, minor:N}
    //     })
    // }
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
    // the value of this type is:
    //
    // {
    //   name: "..."
    //   value: Runtime
    // }
    Runtime,
}

/// the type of dependent libraries
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExternalLibraryDependentType {
    // library from the local file system
    //
    // the value of this type is a path to a file (with library so-name), e.g.
    //
    // {
    //   name: "world"
    //   value: Local("~/myprojects/hello/lib/libworld.so.1")
    // }
    //
    // see also `ModuleDependentType::Local`
    Local = 0x0,

    // library from a remote GIT repository
    //
    // e.g.
    //
    // {
    //   name: "lz4"
    //   value: Remote(
    //     {
    //       url:"https://github.com/hemashushu/xiaoxuan-core-extension.git",
    //       revision="commit/tag",
    //       path="/libraries/lz4/lib/liblz4.so.1"
    //     })
    // }
    //
    // see also `ModuleDependentType::Remote`
    Remote,

    // library from the central repository
    //
    // the value of this type is:
    // {
    //   name: "zlib"
    //   value: Share(
    //     {
    //       repository_name: "...",
    //       version: {major:M, minor:N}
    //     })
    // }
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
    // {
    //   name: "zstd"
    //   value: Runtime
    // }
    Runtime,

    // library from system
    //
    // the value of this type is `LIB_SO_NAME`
    //
    // e.g.
    // {
    //   name: "lz4"
    //   value: System("liblz4.so.1")
    // }
    System,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "module")]
pub enum ModuleDependentValue {
    Local(String),
    Remote(Box<DependentRemote>),
    Share(Box<DependentShare>),
    Runtime,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "library")]
pub enum ExternalLibraryDependentValue {
    Local(/* library soname path */ String),
    Remote(Box<DependentRemote>),
    Share(Box<DependentShare>),
    Runtime,
    System(/* library soname */ String),
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
#[serde(rename = "remote")]
pub struct DependentRemote {
    pub url: String,
    pub reversion: String, // commit or tag
    pub path: String,
    pub values: Option<HashMap<String, PropertyValue>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "share")]
pub struct DependentShare {
    pub repository: Option<String>, // the name of repository
    pub version: String,            // e.g. "1.0"
    pub values: Option<HashMap<String, PropertyValue>>,
}

impl Display for ExternalLibraryDependentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExternalLibraryDependentType::Local => f.write_str("local"),
            ExternalLibraryDependentType::Remote => f.write_str("remote"),
            ExternalLibraryDependentType::Share => f.write_str("share"),
            ExternalLibraryDependentType::Runtime => f.write_str("runtime"),
            ExternalLibraryDependentType::System => f.write_str("system"),
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
        DependentRemote, DependentShare, ExternalLibraryDependentValue, ModuleDependentValue,
    };

    #[test]
    fn test_serialize_module_dependent_value() {
        assert_eq!(
            ason::to_string(&ModuleDependentValue::Local(
                "~/myprojects/hello".to_owned()
            ))
            .unwrap(),
            r#"module::Local("~/myprojects/hello")"#
        );

        assert_eq!(
            ason::to_string(&ModuleDependentValue::Remote(Box::new(DependentRemote {
                url: "https://github.com/hemashushu/xiaoxuan-core-extension.git".to_owned(),
                reversion: "v1.0.0".to_owned(),
                path: "/modules/sha2".to_owned(),
                values: None,
            })))
            .unwrap(),
            r#"module::Remote({
    url: "https://github.com/hemashushu/xiaoxuan-core-extension.git"
    reversion: "v1.0.0"
    path: "/modules/sha2"
    values: Option::None
})"#
        );

        // let mut props = HashMap::new();
        // props.insert("foo".to_owned(), PropertyValue::String("Hello".to_owned()));
        // props.insert("bar".to_owned(), PropertyValue::Number(123));

        assert_eq!(
            ason::to_string(&ModuleDependentValue::Share(Box::new(DependentShare {
                repository: Option::Some("default".to_owned()),
                version: "11.13".to_owned(),
                values: None,
            })))
            .unwrap(),
            r#"module::Share({
    repository: Option::Some("default")
    version: "11.13"
    values: Option::None
})"#
        );

        // values: Option::Some({
        //     "foo": value::string("Hello")
        //     "bar": value::number(123_i64)
        // })

        assert_eq!(
            ason::to_string(&ModuleDependentValue::Runtime).unwrap(),
            r#"module::Runtime"#
        );
    }

    #[test]
    fn test_deserialize_external_library_dependent_value() {
        let s0 = r#"library::Share({
            repository: Option::Some("default")
            version: "17.19"
        })"#;

        let v0: ExternalLibraryDependentValue = ason::from_str(s0).unwrap();
        assert_eq!(
            v0,
            ExternalLibraryDependentValue::Share(Box::new(DependentShare {
                repository: Option::Some("default".to_owned()),
                version: "17.19".to_owned(),
                values: None,
            }))
        );

        let s1 = r#"library::Remote({
            url: "https://github.com/hemashushu/xiaoxuan-core-extension.git"
            reversion: "v1.0.0"
            path: "/libraries/lz4/lib/liblz4.so.1"
        })"#;
        let v1: ExternalLibraryDependentValue = ason::from_str(s1).unwrap();
        assert_eq!(
            v1,
            ExternalLibraryDependentValue::Remote(Box::new(DependentRemote {
                url: "https://github.com/hemashushu/xiaoxuan-core-extension.git".to_owned(),
                reversion: "v1.0.0".to_owned(),
                path: "/libraries/lz4/lib/liblz4.so.1".to_owned(),
                values: None,
            }))
        );
    }
}
