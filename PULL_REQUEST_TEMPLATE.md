# Windows Build Support with Full UI Compilation

## Summary
This PR adds comprehensive Windows support for building Stylus with the complete UI, enabling standalone compilation and execution on Windows platforms. Previously, Stylus could only be built on Linux/macOS or required Docker/WSL on Windows.

## Problem
- Stylus build failed on Windows due to Unix-specific code
- Deno bundling commands used paths incompatible with Windows
- Build scripts required Unix tools like gzip
- Windows users couldn't build the full UI locally

## Solution
### Technical Changes
1. **Fixed Unix-specific code** (`crates/stylus/src/main.rs`)
   - Replaced `cfg!(unix)` runtime checks with `#[cfg(unix)]` compile-time directives
   - Unix-specific file permissions code only compiles on Unix systems

2. **Updated Deno bundling** (`crates/stylus-ui/build.rs`)
   - Fixed relative paths: `web/deno.json` → `./web/deno.json`
   - Added conditional gzip compression: skip on Windows where not available

3. **Added cross-platform CSS build script** (`crates/stylus-ui/build_css.ts`)
   - TypeScript script using Deno for CSS import inlining
   - Works on Windows, Linux, and macOS
   - Processes all CSS partials into a single optimized file

### Results
- ✅ **Full React UI builds** on Windows (1.2MB minified JS bundle)
- ✅ **Complete CSS styles** compiled (29KB with all styles)
- ✅ **Standalone executable** with embedded UI (15.4MB total)
- ✅ **All UI features working**: visualizations, responsive design, dark/light mode
- ✅ **Native Windows development** without Docker/WSL

## Test Plan
- [x] Build debug version on Windows
- [x] Build release version on Windows
- [x] Test full UI functionality (JavaScript/CSS serving)
- [x] Verify all visualization types work
- [x] Test responsive design and theme switching
- [x] Confirm log viewing and modal functionality
- [x] Validate server starts and serves static content correctly

## Files Changed
- `crates/stylus/src/main.rs` - Fix Unix-specific conditional compilation
- `crates/stylus-ui/build.rs` - Update Deno bundling for Windows
- `crates/stylus-ui/build_css.ts` - New cross-platform CSS build script
- `crates/stylus-ui/deno.lock` - Updated Deno dependencies
- `crates/stylus-ui/web/deno.lock` - Updated web dependencies
- `crates/stylus-ui/web/node_modules/@types/node` - Remove broken symlink

## Performance Impact
- No impact on existing platforms (Linux/macOS)
- Windows builds now equivalent to other platforms
- Full UI bundle sizes: 1.2MB JS, 29KB CSS, 15.4MB executable

## Breaking Changes
- None. This is purely additive Windows support.

## Additional Notes
This change significantly improves Windows developer experience by enabling native builds. Users can now:
- Develop Stylus locally on Windows
- Build full-featured releases
- Test UI changes without Docker/WSL
- Contribute to Stylus from Windows machines

The implementation maintains full compatibility with existing build processes while adding robust Windows support.