# Introduction

This is a small utility that can be used to listen to the sample sound packs contained in the Adafruit [Trellis M4 Beat Sequencers](https://learn.adafruit.com/trellis-m4-beat-sequencer/eight-step-simple-sequencer) project.

The utility will download the `sound_packs.zip` referenced in the tutorial. These will be extracted to `~/.trellis_sounds/sound_packs/`.

Listing the sound packs and playing one or all sound packs are the only operations currently supported.

Future enhancements may provide the ability to upload a sound pack to a Trellis M4 or modifying sounds for use on the device.

# Installation

Currently requires that you install Rust. See [https://rustup.sh](https://rustup.sh).

```cargo install --git https://github.com/joshuabenuck/trellis_sounds```

# Use

To get help run:

```trellis_sounds --help```

*Note:* The first time any of the commands below are run, the sound packs will be automatically downloaded.

To list all sound packs in the download:

```trellis_sounds --list```

To play a single sound pack:

```trellis_sounds --play <packname>```

To play all of them, use `all` as the packname.

# Limitations

* Has only been tested on a Windows 10 system.
* No pre-built binaries available.
* User must manually delete `~/.trellis_sounds`.
* No support for alternate file locations.
