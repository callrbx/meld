
# Meld Docs

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
  * Either individual configs (i3/config) or a directory of configs (vim/) that are tracked in the bin
* Set
  * Usually same level as the bin
* Subset
  * A furhter breakdown of configs within a bin
    * Inside of the DE bin, a user could track their "konsole" and "dolphin" configs as subsets
* Store Path
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

___

## Design Objectives

* Language Agnostic
  * 'meld' is a layout and protocol
  * enables easy client implimentation

* Mostly Human readable files
  * No weird or custom file formats
* Use a "one bin per set" philosophy
  * enables logical grouping of similar configs
  * subsets can be used for further control

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
  * track a new config to the bin
  * -s - initialize subset information
* pull
  * install a config from the bin
  * -r - revert
    * pull, but on a specific version of the config blob
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

1. Tracked - Primary table for matching configs to blob names
    * id - The SHA512 hash of the Stored Path
    * subset - An optional string to identify if the config is a member of a subset
        * Blank if not in a subset
2. Versions - Enable basic version control (TODO)
    * id - SHA512 hash of blob contents
    * ver - The current version of the config (increments by one on pushes of previously tracked configs)
    * sphash - The SHA512 hash of the blobs Stored Path

Versioning will be enabled through "SELECT"ing from Versions where 'spash' matches the 'id' in tracked, and 'rev' matches the revision you want to use.
___
The Meld Directory layout is:

```
meld_dir/
|  config.yml             # config and metadata about the bin
|  meld.db                # sqlite db file
|__blobs/
   |__<HASH1>/            # a config with 2 tracked versions
       |  1
       |  2
   |__<HASH2>/            # a config with 1 tracked config, and a blob config
       |  config.yml      # optional config; sets options like pruning
       |  1               
```
