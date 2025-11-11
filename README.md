# calf

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![codecov](https://img.shields.io/codecov/c/github/puffyCid/calf?style=for-the-badge)](https://codecov.io/github/puffyCid/calf)
![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/puffycid/calf/audit.yml?label=Audit&style=for-the-badge)

A small and *very* basic Rust library to read [QCOW](https://en.wikipedia.org/wiki/Qcow) disk images. It was primarily developed to learn how to write disk image forensic parsers.

This library would have been impossible without the excellent resources provided at:

- [qemu](https://www.qemu.org/docs/master/interop/qcow2.html)
- [libqcow](https://github.com/libyal/libqcow)

And other QCOW Rust examples
- [qcow-rs](https://github.com/panda-re/qcow-rs)