# Script files

## Launch

Mélodium scripts can be launched using `melodium` command:
```
melodium main_script.mel
```

Or if the main script includes a shebang:
```
./main_script.mel
```
Recommended shebang being `#!/usr/bin/env melodium`.

Script file _must_ contains basic identity informations to be used as entrypoint:
- name (required),
- version (optional, in [semver](https://semver.org/) sematic),
- requirements (optional).
```
#!/usr/bin/env melodium
#! name = my_script
#! version = 0.1.0
#! require = std:0.8.0 fs:0.8.0 …

// Content
```

<!-- ## Hierarchy and inclusion

There are 3 roots when reffering to external elements in Mélodium:
- `main`, relative to the main script file;
- `core`, relative to the build-in language elements;
- `std`, relative to the standard library.

The `local` root can also be used and refers to the current location of the script.
Areas in Mélodium are matching the filesystem hierarchy.

```
// Import element John from local/foo/bar, coming from file ./foo/bar.mel
use local/foo/bar::John

// Import element Doe from main/baz, coming from file <main dir>/baz.mel
use main/baz::Doe
``` -->

## Note about encoding

Mélodium script files are plain UTF-8 text, without byte order mark.
This choice is made for three main reasons:
1. a choice on encoding, even arbitrary, is better than no choice;
1. Unicode provides the wider support for any characters from all human languages and scripts, existing and future, ensuring continuity;
1. the Mélodium engine is implemented in Rust, itself natively representing text as UTF-8.
