# Worlds are configuration files, not an enum

Each **World** is defined by its own TOML file, and the main
`open-billy.toml` points to an explicit ordered list such as
`worlds = ["worlds/jungle.toml", "worlds/construction.toml", …]`. This keeps
adding or replacing a world in data rather than code, at the cost of a config
loader and validation layer instead of a closed enum. The shells read each
world’s **Monster Roster**, labels, colors, tile paths, physics, and token
allow-list from these files.
