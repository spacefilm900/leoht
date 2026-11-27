# Fixtures

Static input files for tests. Prefer **generated** data
(`leoht-testkit`) over committed files; commit a fixture only when a generator
cannot reproduce the case (e.g. a real-world EXIF quirk).

Every committed fixture needs a row in [PROVENANCE.md](PROVENANCE.md). A file
without provenance will be rejected in review. This is part of the clean-room
policy (see `/CLEAN_ROOM.md`): no photos of unknown origin, no files copied from
other projects' test suites unless their licence is recorded here.
