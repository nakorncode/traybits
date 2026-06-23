# Sound Assets

TrayBits bundles selected mobile-style notification tones from the Android Open
Source Project sound set. These replace the earlier generated, OpenCode,
Kenney, and Mixkit presets because the previous set did not have enough
notification character.

## AOSP notification presets

Source: Android Open Source Project, `platform/frameworks/base`,
`data/sounds/notifications/ogg`

- Source browser: https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/data/sounds/notifications/ogg/
- License overview: https://source.android.com/license
- License: Apache License 2.0 for the Android platform unless a file states a different license.

The original AOSP files are OGG. TrayBits stores converted mono WAV files so
the existing Windows `PlaySoundW` playback path can use them directly.

Bundled files:

- `aosp-argon.wav`
- `aosp-ceres.wav`
- `aosp-cobalt.wav`
- `aosp-helium.wav`
- `aosp-krypton.wav`
- `aosp-polaris.wav`
- `aosp-procyon.wav`
- `aosp-radon.wav`
- `aosp-spica.wav`
- `aosp-vega.wav`
