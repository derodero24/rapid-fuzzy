---
"rapid-fuzzy": patch
---

Packaging fixes:

- The Windows on ARM64 addon (`rapid-fuzzy-win32-arm64-msvc`) now links the MSVC C runtime statically, as the x64 addon already did. It no longer needs `VCRUNTIME140.dll` (the Visual C++ Redistributable) and loads on a clean Windows on ARM machine.
