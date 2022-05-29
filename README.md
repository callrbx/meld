![Gitlab pipeline status (self-hosted)](https://img.shields.io/gitlab/pipeline-status/meld/meld-rust?branch=main&gitlab_url=https%3A%2F%2Fgitlab.parker.systems)

![GitLab Release (custom instance)](https://img.shields.io/gitlab/v/release/meld/meld-rust?gitlab_url=https%3A%2F%2Fgitlab.parker.systems&include_prereleases)

## Meld Docs

The spec and design choices for Meld

## Goal

Meld will enable easy, cross system configuration control in order to reduce complexity in replicating environments on multiple systems.
___

## Terms

* Bin
  * The directory to store configs and the db
  * Each bin should track a logical grouping of configs
    * A users DE config, for example
* Configs
  * Indiviual files, commited either directly or via their parent folder(s)
  * These are all stored as described in the blobs/ dir
* Subset
  * A further breakdown of configs within a bin
    * Inside of the DE bin, a user could track their "i3" and "polybar" configs as subsets
    * This is helpful since both of these configs are folders with mutiple internal files
* Family
  * A logical grouping of subsets
    * The "wm" family could contain subsets "i3", "polybar", and "dunst" for example
* Map Path
  * The mapped folder on the system
    * \$HOME\$/.config/i3/config - stored
    * /home/icon/.config/i3/config - resolved path
  * Should be the minimum amount of path needed to be resolved correctly
    * This is to avoid having a bunch of variables
    * Since configs are stored similarly once you get to the package setup, the "prefix" is the only part that needs mapping
* Blob
  * The name of a tracked config, stored in bin/blobs/NAME
  * The blob name is the SHA512 hash of the Store Path
  * Blob versions are tracked like bin/blobs/BLOB/n
* Map
  * In order to properly track states of directories, we take "snapshots" of them
  * These snapshots are named with the \<SHA512 of dir path\>-\<version of the snapshot\>
  * The map files contain the \<BLOB\>-\<Version\> of the configs in the dir at the moment the snapshot is taken

___

## Design Objectives

* Language Agnostic
  * 'meld' is a layout and protocol
  * enables easy client implimentation
* Mostly Human readable files
  * No weird or custom file formats
* Use a "one bin per area" philosophy
  * enables logical grouping of similar configs
  * subsets/families can be used for further control
  * ideal useage - one meld bin for all your DE configs
  * non-deal usage - one meld bin for your entire system configs
    * while this would probably still work, why make it more complex?
* Structured in a way that is client agnostic, or easy to serve
  * Should be able to work behind a SSH or HTTP proxy
  * Should also be able to operate directly out of a bin's directory
  * Bins created with different clients should be interoperable
* Enable Mappings for maximum flexibility between systems
  * B1 - \$HOME\$:/home/icon
  * B2 - \$HOME\$:/home/drew.parker
  * Stored Path: \$HOME\$/.config/example.conf
* Ease of use and design consistency should be maintaned above all else
* Enable basic version control through the use of a git subsystem (TODO)

___

## Supported Actions and options

* init
  * initialze a new bin
    * -p - initialize all parent directories needed
    * -f - force use of an existing directory
    * --comments - add some information about the bin to the "binfo" table in the db
      * needs more looking into; potentially store directly in a "info.txt" in the bin root (TODO)

* push
  * track a new config to the bin (or update an existing config)
  * -s/--subset - add subset information
  * -t/--tag - add tag information
  * -f/--family - add family information
* pull
  * install a config from the bin
  * -t/--tag - pull a config matching the most recent specific tagged version
  * -v/--version - pull a config matching the specified version
  * -r/--recent - if -t/-v specified and not found, this is used to pull the most recent regardless
* list
  * list all tracked configs in the bin
    * add some display options here (TODO)
* pivot
  * rename a variable inside of the tracked table
  * essentially redefines a variable in the db
  * \$HOME\$ -> \$HOME_ICON\$
* sync up/down
  * up - pull new versions of all tracked configs into the bin
    * warn if new configs cannot be pulled
  * down - install all configs from a bin
    * warn/prompt if overwriting existing configs
  * do stuff with subsets here (TODO)
* validate bin/configs/checksums
  * ensure the dir contains all neededm meld files
  * all tracked configs exist on a system (basically sync dryrun? (TODO))
  * Hash all blob files and ensure their tracked hash matches

___

## Meld DB and Bin Layouts

The meld.db file is a SQLite file with 2 tables:

1. Configs - Primary table for matching configs to blob names
    * id - The SHA512 hash of the Stored Path
    * subset - An optional string to identify if the config is a member of a subset
        * Blank if not in a subset
    * family - An optional string to identify if the config is a member of a family
        * Blank if not in a family
2. Versions - Enable basic version control
    * id - SHA512 hash of blob contents
      * If the version refers to a Directory, this is "DIR"
    * ver - The current version of the config (increments by one on pushes of previously tracked configs)
    * tag - A tag for marking specific versions (ie tagging a config that works on older softare versions)
    * owner - The ID (ie blob name) of the Config this Version entry belongs to
3. Maps - Enable snapshoting of directory states (only set if push is called on a dir)
    * id - SHA512 hash of the dir path
    * ver - The snapshot version - only increments if one of the internal files has been updated
    * nhash - A hash of all the concated content hashes of the configs inside of the dir (ie hash(hash1 + hash2 + hash3))

___
The Meld Directory layout is:

```
meld_dir/
|  config.yml             # config and metadata about the bin (not currently implimented)
|  meld.db                # sqlite db file
|__blobs/
   |__<HASH1>/            # a config with 2 tracked versions
       |  1
       |  2
   |__<HASH2>/            # a config with 1 tracked config, and a blob config
       |  config.yml      # optional config; sets options like pruning (not currently implimented)
       |  1               
|__maps/
   |__<HASH2>-<Version>   # a map file for snapshoting the contents of a dir
```

___
Example Meld Usage and Tree
Debuging is Enabled through the setting of [`RUST_LOG`](https://docs.rs/env_logger/latest/env_logger/)
```
> RUST_LOG=debug meld /tmp/meld_test init
[2022-05-29T02:25:46Z INFO  libmeld::bin] Creating bin at /tmp/meld_test
[2022-05-29T02:25:46Z INFO  libmeld::bin] Creating "/tmp/meld_test"
[2022-05-29T02:25:46Z INFO  libmeld::bin] Creating "/tmp/meld_test/maps"
[2022-05-29T02:25:46Z INFO  libmeld::bin] Creating "/tmp/meld_test/blobs"
[2022-05-29T02:25:46Z INFO  libmeld::db] Creating "/tmp/meld_test/meld.db"
[2022-05-29T02:25:46Z INFO  meld] No Errors
> meld /tmp/meld_test push Cargo.toml
> echo "NONSENSE" > Cargo.toml
> meld /tmp/meld_test push Cargo.toml
> rm Cargo.toml
> meld /tmp/meld_test pull ~/Projects/meld/meld-rust/Cargo.toml -v 1
> head Cargo.toml
[package]
name = "meld"
version = "0.1.0"
edition = "2021"
description = "a meld client written in Rust"
authors = ["drew <drew@parker.systems>"]
readme = "README.md"
license = "MIT"

[lib]
> meld /tmp/meld_test push src/
> tree /tmp/meld_test
/tmp/meld_test
├── blobs
│   ├── 18990ecaf4a799b7119dfef47a51eec51a6ba2c7c376527460d82cae2ec1ceeb23ab037904349405cd24c380aa80bcf737035cd22ff0dada53e7eed30dde4b9f
│   │   └── 1
│   ├── 1c54ad4c8d8eb4aeb8a85ac17b2505a1fab357ebbb53837ef753ea202e716470a43749c0e8aa25c8ec519d75677b593e3173a356cd0497aff691ec36b74a533c
│   ├── 355e95d5dbcbe1fba3e41ef3fcb60aa071028359269213a1e3bf034abbcf634c6ec8a10f12c602788cb38292729cd207309f8ba98ff9bbe9fc19795b186990a1
│   │   ├── 1
│   │   └── 2
│   ├── 396814739440b6c50dbd6df3844f6a0296e5d4656dc41c923a45365ba4dbdca050ad0e482075aebc5a587763fc02272dd6112de2ca6332f952d54fbe9a5b5e6f
│   │   └── 1
│   ├── 3befc5987409a51244955c15972cce01651df9f092a589211899bbf2a2d5b8b680b67bdd9d86af5099abf7b782fb0bfa33f009dd7baa99585d609e562733f1ec
│   │   └── 1
│   ├── 40dcd0373e6ee3e5987e0df6a6d449078edb858ca7d983d6dd5b0fc520f8748c8156e0f4231b5b90b31dfe4b2887ff35ca6587f7d413b2a0e2ae5841e2c42e6b
│   │   └── 1
│   ├── 5afe942f5ddb73f436c005a377163e1b1675812a77eae2942f0e0bf69b7ff13ede869e3e61240e2e776167745c7ace07545e8969d88b571a5fef192dc0421c6c
│   │   └── 1
│   ├── 6f8390c0bc657f16393eeee71327e6dce63d36136160b7e134cda47af0404ab0af84dbe86623872bda701bb924f05d928eaafe74f054a6bc0cd6211dd596077b
│   │   └── 1
│   ├── 7d71b39efc830dd5a6b750a04da9e2eb70a643a1d258dc6597506bd8e179879edb3f6750daeeadca0b0d297771fda17e612f610355c63a60d5c0c87ff39e684d
│   ├── 86aa385dc723c2b2a35e5cba823337be01599326345091b71b3f86283bd7a5a5303b50b5c382f6b3fb4a4b41307a7b0a653bdf0bfb2f9f59b30be916d0dd55ad
│   │   └── 1
│   ├── 92cdce5675765e1606b71f7603170dee57a26adda822a5cd6b86d059f4d0481e83a67130fb00289612b3220ee563fc3b6dcfbeab645bf578744527c77502e3d4
│   │   └── 1
│   ├── b04d9ac3de1e4e2d3a633832cc71955dcf696039d11a6750e34df8ad2e863afd158b77f9fc0ea589dff1daad5dd9d6df1946e9449ce43f9a38a4bf5c75b83f91
│   │   └── 1
│   ├── c3373f98aac47c4332ab89e27a3dbf1af844109b31a16eb0ed2f4b39df924dcb1c31df737f257eb162b68f7b9ac60ac2d5f620e83451bfc4db8f411e7e0b1521
│   │   └── 1
│   └── f731bebb01b825610631b257585d7cec56b1154441b529f0f9636b3b74554fd750c1ef75cfd1e1e18b7fabedbc53ca3cab3db05209ce020149739a0a14029819
│       └── 1
├── maps
│   └── 155b01e6d21788db91e748932deaca4962ee6f723cc7afd32e82ccf555304499dad06ef5d92dae480903592dec7a1a77ad46d841d455cddb97bfac2bc6d169c7-1
└── meld.db

16 directories, 15 files
> cat /tmp/meld_test/maps/*
7d71b39efc830dd5a6b750a04da9e2eb70a643a1d258dc6597506bd8e179879edb3f6750daeeadca0b0d297771fda17e612f610355c63a60d5c0c87ff39e684d-1
18990ecaf4a799b7119dfef47a51eec51a6ba2c7c376527460d82cae2ec1ceeb23ab037904349405cd24c380aa80bcf737035cd22ff0dada53e7eed30dde4b9f-1
1c54ad4c8d8eb4aeb8a85ac17b2505a1fab357ebbb53837ef753ea202e716470a43749c0e8aa25c8ec519d75677b593e3173a356cd0497aff691ec36b74a533c-1
40dcd0373e6ee3e5987e0df6a6d449078edb858ca7d983d6dd5b0fc520f8748c8156e0f4231b5b90b31dfe4b2887ff35ca6587f7d413b2a0e2ae5841e2c42e6b-1
86aa385dc723c2b2a35e5cba823337be01599326345091b71b3f86283bd7a5a5303b50b5c382f6b3fb4a4b41307a7b0a653bdf0bfb2f9f59b30be916d0dd55ad-1
396814739440b6c50dbd6df3844f6a0296e5d4656dc41c923a45365ba4dbdca050ad0e482075aebc5a587763fc02272dd6112de2ca6332f952d54fbe9a5b5e6f-1
b04d9ac3de1e4e2d3a633832cc71955dcf696039d11a6750e34df8ad2e863afd158b77f9fc0ea589dff1daad5dd9d6df1946e9449ce43f9a38a4bf5c75b83f91-1
5afe942f5ddb73f436c005a377163e1b1675812a77eae2942f0e0bf69b7ff13ede869e3e61240e2e776167745c7ace07545e8969d88b571a5fef192dc0421c6c-1
c3373f98aac47c4332ab89e27a3dbf1af844109b31a16eb0ed2f4b39df924dcb1c31df737f257eb162b68f7b9ac60ac2d5f620e83451bfc4db8f411e7e0b1521-1
6f8390c0bc657f16393eeee71327e6dce63d36136160b7e134cda47af0404ab0af84dbe86623872bda701bb924f05d928eaafe74f054a6bc0cd6211dd596077b-1
f731bebb01b825610631b257585d7cec56b1154441b529f0f9636b3b74554fd750c1ef75cfd1e1e18b7fabedbc53ca3cab3db05209ce020149739a0a14029819-1
92cdce5675765e1606b71f7603170dee57a26adda822a5cd6b86d059f4d0481e83a67130fb00289612b3220ee563fc3b6dcfbeab645bf578744527c77502e3d4-1
3befc5987409a51244955c15972cce01651df9f092a589211899bbf2a2d5b8b680b67bdd9d86af5099abf7b782fb0bfa33f009dd7baa99585d609e562733f1ec-1
```