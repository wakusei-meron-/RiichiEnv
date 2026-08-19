# Calculation contract M0

`riichienv-calc` is the deterministic four-player calculation authority.  It
accepts an already reconstructed `CalculationInput`; it never accepts replay
events, source-specific snapshots, red-five identities, or string melds.

`contract_version` is required and must equal `m0-v1`.  `counts34` and
`unavailable_counts34` are exactly 34 tile34 counts.  A red five has already
been normalized to its base five by the adapter.  `meld_count` is the logical
number of chi, pon, daiminkan, ankan, and kakan (each is one), from zero to
four.

`unavailable_counts34` is the decision-time count of every physically known
copy: the concealed hand (including the current draw), all rivers, all open or
closed meld tiles, and each currently revealed dora indicator.  Each physical
tile is counted once.  The core checks `unavailable >= concealed` and uses
`4 - unavailable` for remaining copies.  A discard does not change this input:
the copy merely moves from concealed hand to river.

For draw analysis, `sum(counts34) + 3 * meld_count` must be 13.  For discard
analysis it must be 14.  General shanten accepts either.  All malformed input
is a typed error; there is no clamp or legacy fallback.  Batch duplicate IDs
and contract mismatches fail the whole request; invalid individual scenes are
returned in their ordered batch item as typed errors.

`improving_tiles` strictly reduce shanten.  `agari_tiles` are the subset which
turn a zero-shanten hand into -1.  Both lists carry remaining copies and are
reported separately.  Open hands set chiitoitsu and kokushi to `null`.

Source adapters own `state_projection_version`.  Unknown meld encoding or an
unobservable kan-dora reveal boundary must be reported as
`EVENT_BOUNDARY_UNAVAILABLE` before calling this core.
