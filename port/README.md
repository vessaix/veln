# FreeBSD Port for veln

This directory contains the FreeBSD Port files for veln.

## Files Created

### 1. Makefile
The main port Makefile with:
- USES=cargo for Rust build support
- USE_GITHUB=yes to fetch from GitHub
- PLIST_FILES for the veln binary
- OPTIONS for API server (installs RC script)

### 2. pkg-descr
Long description of the port with all features listed.

### 3. distinfo
Checksums placeholder - needs to be regenerated with `make makesum` when ready.

### 4. files/veln.in
RC script template for running `veln api` as a service.

## Before Submission

1. **Create a LICENSE file** in the main repo (BSD-3-Clause)
2. **Tag a release** on GitHub (v0.1.0)
3. **Update distinfo** with actual checksums:
   ```bash
   cd /usr/ports/sysutils/veln
   make makesum
   ```

4. **Test the port**:
   ```bash
   cd /usr/ports/sysutils/veln
   make stage
   make package
   make install
   make deinstall
   ```

5. **Run portlint**:
   ```bash
   portlint -A
   ```

6. **Test with poudriere** (recommended):
   ```bash
   poudriere testport -j 14amd64 -p default sysutils/veln
   ```

## Installation Path

Copy to FreeBSD ports tree:
```bash
cp -r port/sysutils/veln /usr/ports/sysutils/
```

## Submission

Submit via:
- Bugzilla: https://bugs.freebsd.org/bugzilla/ (category: Ports & Packages)
- Or GitHub PR if using the new workflow
