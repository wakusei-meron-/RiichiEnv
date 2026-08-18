# Fork policy

This repository is a long-lived fork of `smly/RiichiEnv`.  The M0 calculation
contract is fork-owned and is not planned for upstream contribution.  Keep the
Apache-2.0 license and upstream attribution in retained source files.

Review upstream changes selectively: record the upstream commit, explain why
it is relevant, cherry-pick it into an isolated branch, and run the complete
calculation fixture suite before merging.  Do not merge upstream wholesale or
publish this fork under the old distribution names.

Release provenance is part of the contract.  The release workflow writes the
validated release commit to `riichienv-calc/SOURCE_GIT_SHA` before producing
the sdist or wheels.  `core_metadata().core_git_sha` must never be `unknown` in
a release artifact.  Consumers pin the GitHub Release artifact URL and its
SHA-256 manifest; PyPI, crates.io, and npm publishing are intentionally absent.
