# scavenger

A tiny svg path parser.

Simplifies svg paths to only include the commands `M`, `L`, `C`, `S`, `Q`, `T`, `Z`.

- Relative commands are converted to absolute commands.
- Elliptical arc commands are converted to quadratic bezier curves. (bezier steps can be configured)
