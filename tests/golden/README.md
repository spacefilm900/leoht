# Golden images

Reference outputs for `tests/golden.rs`. A test renders an image from a
deterministic input and compares it to the PNG here, allowing a tiny per-channel
tolerance (see the test file for why).

* **Never edit these by hand.** Regenerate with `cargo xtask bless`.
* A changed golden in a PR is a *visible behaviour change*: the PR description
  must say why, and the reviewer must look at the before/after images.
* Keep them small (tens of KB). They are checked in as ordinary files.
