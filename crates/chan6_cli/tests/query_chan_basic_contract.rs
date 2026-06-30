#[test]
fn query_chan_basic_contract_keys_are_documented_and_emitted() {
    let source = include_str!("../src/bin/query_chan_basic.rs");
    let doc = include_str!("../../../docs/query_chan_basic_contract.md");

    let top_level_keys = [
        "meta",
        "kline",
        "chan_basic",
        "merged_boxes",
        "fx_lines",
        "bi_lines",
        "segment_lines",
    ];

    for key in top_level_keys {
        assert!(
            source.contains(&format!("\"{key}\"")),
            "query_chan_basic.rs should emit top-level key {key}"
        );
        assert!(
            doc.contains(key),
            "query_chan_basic_contract.md should document key {key}"
        );
    }

    for required in [
        "\"schema_version\": 1",
        "\"query\": \"query-chan-basic\"",
        "\"kline_scope\": \"offset_limit_window\"",
        "\"render_hint\"",
    ] {
        assert!(
            source.contains(required),
            "query_chan_basic.rs should contain required contract fragment {required}"
        );
    }

    for required in [
        "Schema version: 1",
        "query_chan_basic",
        "`chan_basic` is the canonical algorithm output",
        "Do not rename or remove fields without bumping schema_version",
    ] {
        assert!(
            doc.contains(required),
            "query_chan_basic_contract.md should contain required text {required}"
        );
    }
}
