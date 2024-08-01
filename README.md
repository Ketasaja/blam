# Blam — Better Lua assert messages

`blam file.lua file.luau directory`

## What?

Replaces nonexistent or empty string message arguments of Lua `assert` calls with the expression being asserted. `assert(jump())` becomes `assert(jump(), "jump()")`. Compatible with Lua 5.1, Lua 5.2, Lua 5.3, Lua 5.4, and Luau.

## Why?

Often `assert` messages are meant for developers, not users. `assert(#inventory > 0)` is about as clear to someone who knows Lua as `assert(#inventory > 0, "inventory is empty")`. But the default `assertion failed!` message is useful to nobody, and forces developers to open the file to figure out what's being asserted, even if once they know the assertion, the problem is obviously elsewhere.

`assert` is even more commonly useful in Luau, where it's used to refine types.

Blam also quiets [Selene](https://github.com/Kampfkarren/selene/) warnings for `assert` calls without a message argument.
