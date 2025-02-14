// Copyright (c) 2024 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

pub mod opcode;

use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

// About Runtime Edition
// ---------------------
//
// Runtime editions are used to represent mutually incompatible generations,
// meaning different editions may have different syntax and features.
//
// An application and module must specify a runtime edition. Only when the
// specified edition matches the actual runtime edition exactly can the application
// and unit tests run.
//
// Note that an edition is different from a version number. Editions cannot be compared,
// nor do they have backward compatibility. For example, a runtime with edition "2028"
// cannot run applications with edition "2025" or "2030".
//
// If the edition of a module is inconsistent with the application's edition, the compiler
// will still attempt to compile it but will only use the edition specified by the application
// (i.e., the compiler will ignore the edition declared by the module). Note that this does
// not guarantee successful compilation. Application developers should check for updates to
// dependent modules and try to keep the module's edition consistent with the application's.
pub const RUNTIME_EDITION: &[u8; 8] = b"2025\0\0\0\0";
pub const RUNTIME_EDITION_STRING: &str = "2025";

// Semantic Versioning
// - https://semver.org/
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct EffectiveVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

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
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum VersionCompatibility {
    Equals,
    GreaterThan,
    LessThan,
    Conflict,
}

impl EffectiveVersion {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn from_u64(value: u64) -> Self {
        let patch = (value & 0xffff) as u16;
        let minor = ((value >> 16) & 0xffff) as u16;
        let major = ((value >> 32) & 0xffff) as u16;
        Self {
            major,
            minor,
            patch,
        }
    }

    /// "x.y.z"
    pub fn from_str(version: &str) -> Self {
        let nums = version
            .split('.')
            .map(
                |item| item.parse::<u16>().unwrap(), /* u16::from_str_radix(item, 10).unwrap() */
            )
            .collect::<Vec<_>>();
        assert!(nums.len() == 3);

        Self {
            major: nums[0],
            minor: nums[1],
            patch: nums[2],
        }
    }

    pub fn to_u64(&self) -> u64 {
        let mut value = self.major as u64;
        value = (value << 16) | self.minor as u64;
        value = (value << 16) | self.patch as u64;
        value
    }

    pub fn compatible(&self, other: &EffectiveVersion) -> VersionCompatibility {
        if self.major != other.major {
            // major differ
            VersionCompatibility::Conflict
        } else if self.major == 0 {
            // zero major
            if self.minor != other.minor {
                // minor differ
                VersionCompatibility::Conflict
            } else if self.patch > other.patch {
                VersionCompatibility::GreaterThan
            } else if self.patch < other.patch {
                VersionCompatibility::LessThan
            } else {
                VersionCompatibility::Equals
            }
        } else {
            // normal major
            if self.minor > other.minor {
                VersionCompatibility::GreaterThan
            } else if self.minor < other.minor {
                VersionCompatibility::LessThan
            } else if self.patch > other.patch {
                VersionCompatibility::GreaterThan
            } else if self.patch < other.patch {
                VersionCompatibility::LessThan
            } else {
                VersionCompatibility::Equals
            }
        }
    }
}

impl PartialOrd for EffectiveVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.to_u64().partial_cmp(&other.to_u64())
    }
}

impl Display for EffectiveVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

// pub const RUNTIME_MAJOR_VERSION: u16 = 1;
// pub const RUNTIME_MINOR_VERSION: u16 = 0;
// pub const RUNTIME_PATCH_VERSION: u16 = 0;

// the max version number the current runtime supported
pub const IMAGE_FORMAT_MAJOR_VERSION: u16 = 1;
pub const IMAGE_FORMAT_MINOR_VERSION: u16 = 0;

// About the Version of Shared Modules
// -----------------------------------
//
// An application (or shared module) may depend on one or more other shared modules,
// when an application (or shared module) references a shared module, it is necessary
// to declare the major and minor version of the shared module.
//
// version conflicts
// -----------------
//
// If a shared module appears multiple times in the dependency tree with
// different versions and the major version numbers differ, the compiler
// will complain. However, if the major version numbers are the same, the
// highest minor version wil be selected.
//
// Note that this implies that in the actual application running, the minor
// version of a module might be higher than what the application
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
//
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
    //   "module_name": module::local({
    //       path: "~/myprojects/hello"
    //     })
    // ]
    //
    // because of the lack of version information on the local file system,
    // this type of dependency can only be used as local development and testing.
    // DO NOT distribute modules containing this type of dependency to the
    // central repository, actually the compiler and runtime will
    // refuse to compile when a "Remote" and "Share" module containing
    // a "Local" dependency.
    //
    // It's worth noting that the local module is recompiled at EVERY compilation
    // if the source code changed.
    Local = 0x0,

    // module from a remote GIT repository
    //
    // the value of this type contains the Git repository url, commit (hash)
    // and path, e.g.
    //
    // modules: [
    //   "module_name": module::remote(
    //     {
    //       url: "https://github.com/hemashushu/xiaoxuan-core-extension.git",
    //       revision: "commit or tag",
    //       path: "/modules/sha2"
    //     })
    // ]
    //
    // when a project is compiled or run, the remote resource is
    // downloaded first, and then cached in a local directory.
    //
    // note that the normal HTTP web service is **NOT** suitable for
    // remote modules bacause of the lack of update information.
    //
    // because the remote revisions are not comparable, this type of dependency
    // can only be used as internal development and testing.
    // DO NOT distribute modules containing this type of dependency to the
    // central registry, the compiler and runtime will refuse to compile when
    // a "Share" module containing a "Remote" dependency.
    Remote,

    // module from the central registry
    //
    // the runtime specifies a default location as the
    // "shared modules central registry", which is a Git repo
    // that provides the module index.
    //
    // users can also customize a different location or add
    // multiple registry in the runtime settings.
    //
    // the value of this type contains the version, e.g.
    //
    // modules:[
    //   "module_name": module::share(
    //     {
    //       version: "{major.minor.patch}"
    //     })
    // ]
    //
    // this type of module is downloaded and cached to a local directory, e.g.
    //
    // "{/usr/lib/anc, /usr/local/lib/anc, ~/.anc}/modules/modname/VERSION"
    Share,

    // module that comes with the runtime
    //
    // this type of module is located locally in a directory, e.g.
    //
    // "{/usr/lib/anc, /usr/local/lib/anc, ~/.anc}/runtimes/EDITION/modules/modname"
    //
    // there is no value of this type because the module name is specified
    // in the configuration, e.g.
    //
    // modules:[
    //   "module_name": module::runtime
    // ]
    Runtime,

    // this type is for assembler and linker use only,
    // and it only exists in the object files,
    // it represents the current module.
    //
    // users **CANNOT** configure modules of this type, e.g.
    //
    // modules:[
    //   "module": module::module  // INVALID
    // ]
    //
    // Under the hood
    // --------------
    // When generating a "object module", the assembler adds a dependency of
    // this type, so the linker can import functions and data from other submodules under
    // the same module.
    Module,
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
    //    "hello": library::local({
    //        path: "~/myprojects/hello/output/libhello.so.1"
    //      })
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
    //   "lz4": library::remote(
    //     {
    //       url: "https://github.com/hemashushu/xiaoxuan-cc-lz4.git",
    //       revision: "commit/tag",
    //     })
    // ]
    //
    // see also `ModuleDependentType::Remote`
    Remote,

    // library from the central registry
    //
    // an example of this type:
    //
    // libraries: [
    //   "zlib": library::share(
    //     {
    //       version: "{major.minor.patch}"
    //     })
    // ]
    //
    // this type of library is downloaded and cached to a local directory, e.g.
    // "{/usr/lib/ancc, /usr/local/lib/ancc, ~/.ancc}/modules/libname/VERSION"
    Share,

    // library from system
    //
    // the dependency name is the library's soname removes "lib" prefix and
    // ".so.N" suffix, e.g. "lz4", and the value is library's soname, e.g. "liblz4.so.1".
    //
    // e.g.
    // libraries: [
    //   "lz4": library::system("liblz4.so.1")
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

    #[serde(rename = "module")]
    Module,
}

// The name of `ModuleDependency::Module`.
pub const SELF_REFERENCE_MODULE_NAME: &str = "module";

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "library")]
pub enum ExternalLibraryDependency {
    #[serde(rename = "local")]
    Local(Box<DependencyLocal>),

    #[serde(rename = "remote")]
    Remote(Box<DependencyRemote>),

    #[serde(rename = "share")]
    Share(Box<DependencyShare>),

    #[serde(rename = "system")]
    System(/* the soname of library, e.g. libz.so.1 */ String),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "local")]
pub struct DependencyLocal {
    /// The module's path relative to the application (or module project) folder.
    /// It could also be a path of the file "*.so.VERSION" relative to the application
    /// if the dependency is external library.
    pub path: String,

    /// the default value is []
    #[serde(default)]
    pub parameters: HashMap<String, ParameterValue>,

    /// Optional
    /// the default value is DependencyCondition::True
    #[serde(default)]
    pub condition: DependencyCondition,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "remote")]
pub struct DependencyRemote {
    /// Git repository URL, should be "https" protocol
    pub url: String,

    /// Git commit or tag
    pub reversion: String,

    /// Optional
    /// the default value is []
    #[serde(default)]
    pub parameters: HashMap<String, ParameterValue>,

    /// Optional
    /// the default value is DependencyCondition::True
    #[serde(default)]
    pub condition: DependencyCondition,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "share")]
pub struct DependencyShare {
    /// Semver e.g. "1.0.1"
    pub version: String,

    /// Optional
    /// the default value is []
    #[serde(default)]
    pub parameters: HashMap<String, ParameterValue>,

    /// Optional
    /// the default value is DependencyCondition::True
    #[serde(default)]
    pub condition: DependencyCondition,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "param")]
pub enum ParameterValue {
    #[serde(rename = "string")]
    String(String),

    #[serde(rename = "number")]
    Number(i64),

    #[serde(rename = "bool")]
    Bool(bool),

    #[serde(rename = "prop")]
    Prop(/* property name */ String),

    #[serde(rename = "eval")]
    Eval(String),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "cond")]
pub enum DependencyCondition {
    #[serde(rename = "true")]
    True,

    #[serde(rename = "false")]
    False,

    #[serde(rename = "is_true")]
    IsTrue(/* property or constant name */ String),

    #[serde(rename = "is_false")]
    IsFalse(/* property or constant name */ String),

    #[serde(rename = "eval")]
    Eval(/* expression */ String),
}

impl Default for DependencyCondition {
    fn default() -> Self {
        Self::True
    }
}

impl Display for ExternalLibraryDependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExternalLibraryDependencyType::Local => f.write_str("local"),
            ExternalLibraryDependencyType::Remote => f.write_str("remote"),
            ExternalLibraryDependencyType::Share => f.write_str("share"),
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
    use std::collections::HashMap;

    use pretty_assertions::assert_eq;

    use crate::{
        DependencyCondition, DependencyLocal, DependencyRemote, DependencyShare, EffectiveVersion,
        ExternalLibraryDependency, ModuleDependency, VersionCompatibility, RUNTIME_EDITION,
        RUNTIME_EDITION_STRING,
    };

    #[test]
    fn test_effective_version() {
        let v0 = EffectiveVersion::new(0x11, 0x13, 0x17);
        let n0 = v0.to_u64();
        assert_eq!(n0, 0x0011_0013_0017_u64);

        let v1 = EffectiveVersion::from_u64(n0);
        assert_eq!(v1.major, 0x11);
        assert_eq!(v1.minor, 0x13);
        assert_eq!(v1.patch, 0x17);

        let v2 = EffectiveVersion::from_str("11.13.17");
        assert_eq!(v2.major, 11);
        assert_eq!(v2.minor, 13);
        assert_eq!(v2.patch, 17);
    }

    #[test]
    fn test_effective_version_comparison() {
        let v0 = EffectiveVersion::new(0x11, 0x13, 0x17);
        let v1 = EffectiveVersion::new(0x11, 0x13, 0x17);
        let v2 = EffectiveVersion::new(0x13, 0x11, 0x7);
        let v3 = EffectiveVersion::new(0x11, 0x17, 0x13);
        let v4 = EffectiveVersion::new(0x11, 0x13, 0x23);

        // Eq
        assert!(v0 == v1);
        assert!(v0 != v2);

        // Cmp
        assert!(v0 >= v1);
        assert!(v0 <= v1);
        assert!(v0 < v2);
        assert!(v0 < v3);
        assert!(v0 < v4);
    }

    #[test]
    fn test_effective_version_competibility() {
        assert_eq!(
            EffectiveVersion::from_str("1.2.3").compatible(&EffectiveVersion::from_str("1.2.3")),
            VersionCompatibility::Equals
        );

        assert_eq!(
            EffectiveVersion::from_str("1.2.3").compatible(&EffectiveVersion::from_str("1.1.3")),
            VersionCompatibility::GreaterThan
        );

        assert_eq!(
            EffectiveVersion::from_str("1.2.3").compatible(&EffectiveVersion::from_str("1.2.2")),
            VersionCompatibility::GreaterThan
        );

        assert_eq!(
            EffectiveVersion::from_str("1.2.3").compatible(&EffectiveVersion::from_str("1.11.3")),
            VersionCompatibility::LessThan
        );

        assert_eq!(
            EffectiveVersion::from_str("1.2.3").compatible(&EffectiveVersion::from_str("2.1.3")),
            VersionCompatibility::Conflict
        );

        // zero-major
        assert_eq!(
            EffectiveVersion::from_str("0.2.3").compatible(&EffectiveVersion::from_str("0.2.3")),
            VersionCompatibility::Equals
        );

        assert_eq!(
            EffectiveVersion::from_str("0.2.3").compatible(&EffectiveVersion::from_str("0.2.2")),
            VersionCompatibility::GreaterThan
        );

        assert_eq!(
            EffectiveVersion::from_str("0.2.3").compatible(&EffectiveVersion::from_str("0.2.11")),
            VersionCompatibility::LessThan
        );

        assert_eq!(
            EffectiveVersion::from_str("0.2.3").compatible(&EffectiveVersion::from_str("0.3.2")),
            VersionCompatibility::Conflict
        );
    }

    #[test]
    fn test_runtime_edition() {
        let strlen = RUNTIME_EDITION
            .iter()
            .position(|c| *c == 0)
            .unwrap_or(RUNTIME_EDITION.len());

        assert_eq!(
            std::str::from_utf8(&RUNTIME_EDITION[..strlen]).unwrap(),
            RUNTIME_EDITION_STRING
        );
    }

    #[test]
    fn test_serialize_dependency() {
        assert_eq!(
            ason::to_string(&ModuleDependency::Local(Box::new(DependencyLocal {
                path: "~/projects/helloworld".to_owned(),
                parameters: HashMap::default(),
                condition: DependencyCondition::True
            })))
            .unwrap(),
            r#"module::local({
    path: "~/projects/helloworld"
    parameters: [
    ]
    condition: cond::true
})"#
        );

        assert_eq!(
            ason::to_string(&ModuleDependency::Remote(Box::new(DependencyRemote {
                url: "https://github.com/hemashushu/xiaoxuan-core-module.git".to_owned(),
                reversion: "v1.0.0".to_owned(),
                parameters: HashMap::default(),
                condition: DependencyCondition::False,
            })))
            .unwrap(),
            r#"module::remote({
    url: "https://github.com/hemashushu/xiaoxuan-core-module.git"
    reversion: "v1.0.0"
    parameters: [
    ]
    condition: cond::false
})"#
        );

        assert_eq!(
            ason::to_string(&ModuleDependency::Share(Box::new(DependencyShare {
                version: "2.3".to_owned(),
                parameters: HashMap::default(),
                condition: DependencyCondition::IsTrue("enable_abc".to_owned()),
            })))
            .unwrap(),
            r#"module::share({
    version: "2.3"
    parameters: [
    ]
    condition: cond::is_true("enable_abc")
})"#
        );

        assert_eq!(
            ason::to_string(&ModuleDependency::Share(Box::new(DependencyShare {
                version: "11.13".to_owned(),
                parameters: HashMap::default(),
                condition: DependencyCondition::Eval("enable_abc && enable_xyz".to_owned()),
            })))
            .unwrap(),
            r#"module::share({
    version: "11.13"
    parameters: [
    ]
    condition: cond::eval("enable_abc && enable_xyz")
})"#
        );
    }

    #[test]
    fn test_deserialize_dependency() {
        assert_eq!(
            ason::from_str::<ExternalLibraryDependency>(
                r#"library::local({
                path: "~/projects/helloworld/libabc.so.1"
            })"#
            )
            .unwrap(),
            ExternalLibraryDependency::Local(Box::new(DependencyLocal {
                path: "~/projects/helloworld/libabc.so.1".to_owned(),
                parameters: HashMap::default(),
                condition: DependencyCondition::True
            }))
        );

        assert_eq!(
            ason::from_str::<ExternalLibraryDependency>(
                r#"library::remote({
                url: "https://github.com/hemashushu/xiaoxuan-cc-lz4.git"
                reversion: "v1.0.0"
                condition: cond::false
            })"#
            )
            .unwrap(),
            ExternalLibraryDependency::Remote(Box::new(DependencyRemote {
                url: "https://github.com/hemashushu/xiaoxuan-cc-lz4.git".to_owned(),
                reversion: "v1.0.0".to_owned(),
                parameters: HashMap::default(),
                condition: DependencyCondition::False
            }))
        );

        assert_eq!(
            ason::from_str::<ExternalLibraryDependency>(
                r#"library::share({
                version: "2.3"
                condition: cond::is_true("enable_abc")
            })"#
            )
            .unwrap(),
            ExternalLibraryDependency::Share(Box::new(DependencyShare {
                version: "2.3".to_owned(),
                parameters: HashMap::default(),
                condition: DependencyCondition::IsTrue("enable_abc".to_owned()),
            }))
        );

        assert_eq!(
            ason::from_str::<ExternalLibraryDependency>(
                r#"library::share({
                version: "11.13"
                condition: cond::is_true("enable_abc && enable_xyz")
            })"#
            )
            .unwrap(),
            ExternalLibraryDependency::Share(Box::new(DependencyShare {
                version: "11.13".to_owned(),
                parameters: HashMap::default(),
                condition: DependencyCondition::IsTrue("enable_abc && enable_xyz".to_owned()),
            }))
        );
    }
}
