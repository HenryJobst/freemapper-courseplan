These keys are for tests only: the test key pair of freemapper-core's `freemapper-update` crate
(`crates/freemapper-update/tests/keys/test.*`, key id 09F80E494B3AE92B), copied here because the integration tests
sign manifests with it. It has no password, and its secret key is public on purpose. A release build never accepts
it: `src/update/mod.rs` adds it to the accepted keys only in unit tests and in a debug build with the `test-stub`
feature, and refuses to compile with `FREEMAPPER_COURSEPLAN_DIST=github-release` if `assets/update/minisign.pub`
(the release key) ever holds it.

The `.minisig` files in `../fixtures/update/` were made with
`minisign -S -s test.key -m manifest-99.json -x manifest-99.json.minisig -t "freemapper-courseplan test fixture"`
(`manifest-99-manual.json` is the same release with `min_self_update_from` 98.0.0). The manifests carry
`"app": "freemapper-courseplan"`. `manifest-99.json.wrong-key.minisig` signs the same manifest with a throwaway key
that no build accepts; its secret half was deleted after signing. The tests of a bad signature use it.

The test `update_yes_installs_over_a_copied_binary` and the install-mode test sign their own manifest at run time the
same way. They are skipped when `minisign` is not on the PATH, unless `FREEMAPPER_COURSEPLAN_TEST_REQUIRE_MINISIGN`
is set (CI sets it), which makes them fail instead.

Run them with `cargo test -p freemapper-courseplan-cli --features test-stub --test update`.
