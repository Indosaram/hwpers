# Release Checklist for v0.3.0

## Pre-release Steps ✅

- [x] Update version in `Cargo.toml` to 0.3.0
- [x] Update version in `README.md` documentation
- [x] Create/update `CHANGELOG.md` with all changes
- [x] Create `RELEASE_NOTES.md` with user-friendly description
- [x] Run all tests (`cargo test`)
- [x] Build in release mode (`cargo build --release`)
- [x] Verify no compiler warnings
- [x] Create git tag `v0.3.0`
- [x] Commit all changes

## Publishing Steps 📦

1. **Verify package contents**:
   ```bash
   cargo package --list
   ```

2. **Build and test the package**:
   ```bash
   cargo build
   cargo test
   ```

3. **Publish to crates.io** (dry run first):
   ```bash
   cargo publish --dry-run
   cargo publish
   ```

4. **Push to GitHub**:
   ```bash
   git push origin main
   git push origin v0.3.0
   ```

5. **Create GitHub Release**:
   - Go to https://github.com/Indosaram/hwpers/releases
   - Click "Draft a new release"
   - Select tag `v0.3.0`
   - Title: "v0.3.0 - HWP Writer Implementation"
   - Copy content from `RELEASE_NOTES.md`
   - Attach any binary artifacts if needed
   - Publish release

## Post-release Steps 🎉

- [ ] Verify crate is available on crates.io
- [ ] Update documentation on docs.rs
- [ ] Announce release on relevant channels
- [ ] Close related issues/PRs
- [ ] Plan next version milestones

## Version Summary

**Version**: 0.3.0  
**Release Date**: 2025-01-29  
**Type**: Minor Release (New Features)  

**Key Features**:
- HWP document writing capability
- Hyperlink support
- Header/footer functionality
- Page layout configuration
- Paragraph formatting

**Breaking Changes**: None (backward compatible)

## Notes

- This is an early release of writer functionality
- Some features are partially implemented
- See README.md for known limitations
- Report issues at: https://github.com/Indosaram/hwpers/issues