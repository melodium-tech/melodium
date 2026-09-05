# Semantic Versionning

Mélodium follows the semantic versioning norm [SemVer](https://semver.org/).
This versioning system aims to be consistent with software evolution and avoid custom version designation caveats.

A version is always designated as `x.y.z`, with optionnal pre-release identifier added at the end as `x.y.z-rc1`.

> This page is essentially an adaptation from the [Cargo Rust manual](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) to Mélodium, from which the versioning and dependency system is largely inspired.

## Version designation

```toml
[dependencies]
regex = "0.8.1"
```

The string `"0.8.1"` is a version requirement. Although it looks like a
specific *version* of the `regex` package, it actually specifies a *range* of
versions and allows [SemVer] compatible updates. An update is allowed if the new
version number does not modify the left-most non-zero number in the major, minor,
patch grouping. In this case, we may have version `0.8.3` if it is the latest `0.8.z` release, but would not
update us to `0.9.0`. If instead we had specified the version string as `1.0`,
cargo should update to `1.1` if it is the latest `1.y` release, but not `2.0`.
The version `0.0.x` is not considered compatible with any other version.

[SemVer]: https://semver.org

Here are some more examples of version requirements and the versions that would
be allowed with them:

```notrust
1.2.3  :=  >=1.2.3, <2.0.0
1.2    :=  >=1.2.0, <2.0.0
1      :=  >=1.0.0, <2.0.0
0.2.3  :=  >=0.2.3, <0.3.0
0.2    :=  >=0.2.0, <0.3.0
0.0.3  :=  >=0.0.3, <0.0.4
0.0    :=  >=0.0.0, <0.1.0
0      :=  >=0.0.0, <1.0.0
```

This compatibility convention is different from SemVer in the way it treats
versions before 1.0.0. While SemVer says there is no compatibility before
1.0.0, Mélodium considers `0.x.y` to be compatible with `0.x.z`, where `y ≥ z`
and `x > 0`.

It is possible to further tweak the logic for selecting compatible versions
using special operators as described in the next section.

Use the default version requirement strategy, e.g. `std = "1.2.3"` where possible to maximize compatibility.

## Version requirement syntax

### Caret requirements

**Caret requirements** are the default version requirement strategy. 
This version strategy allows [SemVer] compatible updates.
They are specified as version requirements with a leading caret (`^`).

`^1.2.3` is an example of a caret requirement.

Leaving off the caret is a simplified equivalent syntax to using caret requirements.
While caret requirements are the default, it is recommended to use the
simplified syntax when possible.

`log = "^1.2.3"` is exactly equivalent to `log = "1.2.3"`.

### Tilde requirements

**Tilde requirements** specify a minimal version with some ability to update.
If you specify a major, minor, and patch version or only a major and minor
version, only patch-level changes are allowed. If you only specify a major
version, then minor- and patch-level changes are allowed.

`~1.2.3` is an example of a tilde requirement.

```notrust
~1.2.3  := >=1.2.3, <1.3.0
~1.2    := >=1.2.0, <1.3.0
~1      := >=1.0.0, <2.0.0
```

### Wildcard requirements

**Wildcard requirements** allow for any version where the wildcard is
positioned.

`*`, `1.*` and `1.2.*` are examples of wildcard requirements.

```notrust
1.*   := >=1.0.0, <2.0.0
1.2.* := >=1.2.0, <1.3.0
```

> **Note**: Mélodium does not allow bare `*` versions.

### Comparison requirements

**Comparison requirements** allow manually specifying a version range or an
exact version to depend on.

Here are some examples of comparison requirements:

```notrust
>= 1.2.0
> 1
< 2
= 1.2.3
```

### Multiple version requirements

As shown in the examples above, multiple version requirements can be
separated with a comma, e.g., `>= 1.2, < 1.5`.
