// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

pub mod opcode;

use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

// About Runtime Edition
// ---------------------
//
// Runtime editions represent mutually incompatible generations of the runtime.
// Different editions may introduce new syntax and features.
//
// Applications and modules must specify a runtime edition. The application and
// unit tests can only run if the specified edition matches the runtime edition exactly.
//
// Note: An edition is not the same as a version number. Editions cannot be compared
// or assumed to have backward compatibility. For example, a runtime with edition "2028"
// cannot run applications with editions "2025" or "2030".
//
// If a module's edition differs from the application's edition, the compiler will
// attempt to compile it using the application's edition. However, this does not
// guarantee successful compilation. Developers should ensure that module editions
// are consistent with the application's edition.
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

    /// Parses a version string in the format "x.y.z".
    pub fn from_version_string(version: &str) -> Self {
        let nums = version
            .split('.')
            .map(
                |item| item.parse::<u16>().unwrap(), // or `u16::from_str_radix(item, 10).unwrap()`
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
            // Major version differs.
            VersionCompatibility::Conflict
        } else if self.major == 0 {
            // Zero major version.
            if self.minor != other.minor {
                // Minor version differs.
                VersionCompatibility::Conflict
            } else if self.patch > other.patch {
                VersionCompatibility::GreaterThan
            } else if self.patch < other.patch {
                VersionCompatibility::LessThan
            } else {
                VersionCompatibility::Equals
            }
        } else {
            // Normal major version.
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

// The maximum version number supported by the current runtime.
pub const IMAGE_FORMAT_MAJOR_VERSION: u16 = 1;
pub const IMAGE_FORMAT_MINOR_VERSION: u16 = 0;

// About the Version of Shared Modules
// -----------------------------------
//
// Applications or shared modules may depend on one or more other shared modules.
// When referencing a shared module, it is necessary to declare its major and minor version.
//
// Version Conflicts
// -----------------
//
// If a shared module appears multiple times in the dependency tree with
// differing major version numbers, the compiler will raise an error.
// If the major version numbers are the same, the highest minor version
// will be selected.
//
// This means that at runtime, the minor version of a module may be higher
// than what the application explicitly declares. This is acceptable because
// minor version updates are expected to maintain backward compatibility.
//
// For example, if an application depends on a module with version 1.4.0,
// the actual runtime version could range from 1.4.0 to 1.99.99.
//
// For local and remote file-based shared modules and libraries, which lack
// version information, compilation will fail if their sources (e.g., file paths
// or URLs) do not match.
//
// Zero Major Version
// ------------------
//
// A zero major version indicates that the module is in a beta stage, and each
// minor version is incompatible. If the dependency tree contains minor version
// inconsistencies for modules with a zero major version, compilation will fail.
//
// Note to Authors of Shared Modules
// ----------------------------------
//
// The public interface (API) of a shared module MUST REMAIN UNCHANGED throughout
// the release of major versions. For example:
// - The API of version 1.9 and 1.1 should be the same. Newer versions may add interfaces,
//   but existing interfaces should NOT be changed or removed.
// - The API of version 1.9 and 2.0 may differ.

/// The raw data type of operands.
pub type Operand = [u8; 8];
pub const OPERAND_SIZE_IN_BYTES: usize = 8;

/// The data type for:
/// - Function parameters and results.
/// - Instruction operands.
///
/// Note: The `i32` here refers to a 32-bit integer, equivalent to `uint32_t` in C or `u32` in Rust.
/// Do not confuse it with Rust's `i32`. The same applies to `i8`, `i16`, and `i64`.
///
/// P.S. the function `std::mem::transmute` can be used for converting data type
/// between `enum` and `u8` date, e.g.
///
/// ```rust
/// use anc_isa::OperandDataType;
/// let a = unsafe { std::mem::transmute::<OperandDataType, u8>(OperandDataType::F32) };
/// assert_eq!(a, 2);
/// ```
///
/// It can be also done through 'union', e.g.
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
/// https://doc.rust-lang.org/nomicon/other-reprs.html
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OperandDataType {
    I32 = 0x0,
    I64,
    F32,
    F64,
}

/// The data type for:
/// - Local variables.
/// - Data in the DATA sections and heap.
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
    ReadOnly = 0x0, // similar to the section ".rodata" in ELF.
    ReadWrite,      // similar to the section ".data" in ELF.
    Uninit,         // similar to the section ".bss" in ELF.
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

// Values for Foreign Function Interface (FFI)
//
// Used for calling VM functions from the outside or returning values to the foreign caller.
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
            panic!("Not a u32.")
        }
    }

    pub fn as_u64(&self) -> u64 {
        if let ForeignValue::U64(v) = self {
            *v
        } else {
            panic!("Not a u64.")
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

/// The type of dependent shared modules.
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ModuleDependencyType {
    // Module from the local file system.
    //
    // The value is a path to a folder, e.g.:
    //
    // ```ason
    // modules: [
    //   "module_name": module::local({
    //       path: "~/myprojects/hello"
    //     })
    // ]
    // ```
    //
    // Local modules are recompiled during every compilation if their source code changes.
    // This type of dependency is suitable for local development and testing only.
    // Modules with "Local" dependencies should not be distributed to the central repository.
    Local = 0x0,

    // Module from a remote Git repository.
    //
    // The value contains the Git repository URL, commit (hash), and path, e.g.:
    //
    // ```ason
    // modules: [
    //   "module_name": module::remote({
    //       url: "https://github.com/hemashushu/xiaoxuan-core-extension.git",
    //       revision: "commit or tag",
    //       path: "/modules/sha2"
    //     })
    // ]
    // ```
    //
    // Remote modules are downloaded and cached locally during compilation or runtime.
    // This type of dependency is suitable for internal development and testing only.
    // Modules with "Remote" dependencies should not be distributed to the central repository.
    Remote,

    // Module from the central registry.
    //
    // The runtime specifies a default location for the central registry, which is a Git repository
    // providing the module index. Users can customize this location or add multiple registries.
    //
    // The value contains the version, e.g.:
    //
    // ```ason
    // modules: [
    //   "module_name": module::share({
    //       version: "{major.minor.patch}"
    //     })
    // ]
    // ```
    Share,

    // Module bundled with the runtime.
    //
    // These modules are located in specific directories, e.g.:
    // "{/usr/lib/anc, /usr/local/lib/anc, ~/.anc}/runtimes/EDITION/modules/MODULE_NAME"
    //
    // The value is the ASON variant `module::runtime`, e.g.:
    //
    // ```ason
    // modules:[
    //   "module_name": module::runtime
    // ]
    // ```
    Runtime,

    // Represents the current module.
    //
    // This type is generated by the assembler automatically and
    // only present in the "import module section" of **object files**.
    // It cannot be configured by users.
    //
    // Note:
    // When objects files are linked, all internal references of functions and data
    // should be resolved, and this virtual module item in the "import module section"
    // would be removed. Therefore, this type would not be present in the shared module and
    // applicaton module image files.
    Current,
}

/// The type of dependent libraries.
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExternalLibraryDependencyType {
    // Library from the local file system.
    //
    // The dependency name is the library's soname without the "lib" prefix and ".so.MAJOR" suffix.
    // The value is a path to the library file, e.g.:
    //
    // ```ason
    // libraries: [
    //    "hello": library::local({
    //        path: "~/myprojects/hello/output/libhello.so.1"
    //      })
    // ]
    // ```
    //
    // Note: The difference between soname, real name, and link name:
    // - soname: "libNAME.so.MAJOR"
    // - real name: "libNAME.so.MAJOR.MINOR.PATCH"
    // - link name: "libNAME.so" (the linker likes "ld" and "lld" only use the NAME part)
    Local = 0x0,

    // Library from a remote Git repository.
    //
    // Example:
    //
    // ```ason
    // libraries: [
    //   "lz4": library::remote({
    //       url: "https://github.com/hemashushu/xiaoxuan-cc-lz4.git",
    //       revision: "commit/tag",
    //     })
    // ]
    // ```
    Remote,

    // Library from the central registry.
    //
    // Example:
    //
    // ```ason
    // libraries: [
    //   "zlib": library::share({
    //       version: "{major.minor.patch}"
    //     })
    // ]
    // ```
    Share,

    // Library from the system.
    //
    // The dependency name is the library's soname without the "lib" prefix and ".so.N" suffix.
    //
    // Example:
    //
    // ```ason
    // libraries: [
    //   "lz4": library::system("liblz4.so.1")
    // ]
    // ```
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
    Current,
}

// The name of `ModuleDependency::Current`.
//
// Note:
// This virtual module name may be present in the "import" statements of assembly files,
// but it will not be present in the "full_name" field in the "import function section",
// "import data section", "function name section", and the "data name section".
// The "full_name" always use the actual name of module.
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
    /// It could also be a path of the file "lib*.so.VERSION" relative to the application
    /// if the dependency is an external library.
    pub path: String,

    /// Optional.
    /// The default value is [].
    #[serde(default)]
    pub parameters: HashMap<String, ParameterValue>,

    /// Optional.
    /// The default value is DependencyCondition::True.
    #[serde(default)]
    pub condition: DependencyCondition,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "remote")]
pub struct DependencyRemote {
    /// Git repository URL, should use the "https" protocol.
    pub url: String,

    /// Git commit or tag.
    pub reversion: String,

    /// Optional.
    /// The default value is [].
    #[serde(default)]
    pub parameters: HashMap<String, ParameterValue>,

    /// Optional.
    /// The default value is DependencyCondition::True.
    #[serde(default)]
    pub condition: DependencyCondition,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "share")]
pub struct DependencyShare {
    /// Semver, e.g., "1.0.1".
    pub version: String,

    /// Optional.
    /// The default value is [].
    #[serde(default)]
    pub parameters: HashMap<String, ParameterValue>,

    /// Optional.
    /// The default value is DependencyCondition::True.
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

// The error in Rust
// -----------------
//
// Sometimes you may want to get a specified type from 'dyn RuntimeError'.
// You can downcast the 'dyn RuntimeError' object to a specified type, e.g.:
//
// let some_error:T = unsafe {
//     &*(runtime_error as *const dyn RuntimeError as *const T)
// };
//
// The 'runtime_error' is a 'fat' pointer, consisting of a pointer to the data and
// another pointer to the 'vtable'.
//
// P.S., the slice object is also a 'fat' pointer, e.g.
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
// References:
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

        let v2 = EffectiveVersion::from_version_string("11.13.17");
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
            EffectiveVersion::from_version_string("1.2.3")
                .compatible(&EffectiveVersion::from_version_string("1.2.3")),
            VersionCompatibility::Equals
        );

        assert_eq!(
            EffectiveVersion::from_version_string("1.2.3")
                .compatible(&EffectiveVersion::from_version_string("1.1.3")),
            VersionCompatibility::GreaterThan
        );

        assert_eq!(
            EffectiveVersion::from_version_string("1.2.3")
                .compatible(&EffectiveVersion::from_version_string("1.2.2")),
            VersionCompatibility::GreaterThan
        );

        assert_eq!(
            EffectiveVersion::from_version_string("1.2.3")
                .compatible(&EffectiveVersion::from_version_string("1.11.3")),
            VersionCompatibility::LessThan
        );

        assert_eq!(
            EffectiveVersion::from_version_string("1.2.3")
                .compatible(&EffectiveVersion::from_version_string("2.1.3")),
            VersionCompatibility::Conflict
        );

        // Zero-major
        assert_eq!(
            EffectiveVersion::from_version_string("0.2.3")
                .compatible(&EffectiveVersion::from_version_string("0.2.3")),
            VersionCompatibility::Equals
        );

        assert_eq!(
            EffectiveVersion::from_version_string("0.2.3")
                .compatible(&EffectiveVersion::from_version_string("0.2.2")),
            VersionCompatibility::GreaterThan
        );

        assert_eq!(
            EffectiveVersion::from_version_string("0.2.3")
                .compatible(&EffectiveVersion::from_version_string("0.2.11")),
            VersionCompatibility::LessThan
        );

        assert_eq!(
            EffectiveVersion::from_version_string("0.2.3")
                .compatible(&EffectiveVersion::from_version_string("0.3.2")),
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
