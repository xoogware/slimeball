# slimeball

Slime world parsing in Rust

Slimeball is a library providing helpers for loading and storing Minecraft worlds saved in the
[Slime world format](https://hypixel.net/threads/dev-blog-5-storing-your-skyblock-island.2190753/).

Anvil is historically a [very bloated format](https://minecraft.wiki/w/Chunk_format#Block_format), storing
lots of data that is unnecessary for short-lived minigame servers. The Slime world format is similar, but contains much
less world generation-related data and compresses much better (uses zstd).

This library is intended for use in [crawlspace](https://github.com/xoogware/crawlspace), but a
converter to prune and convert Anvil worlds to Slime is also being developed here.
