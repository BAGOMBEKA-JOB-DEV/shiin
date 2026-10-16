//! Exercises shell tokenization and SQL classification without executing commands.
#![no_main]
#![forbid(unsafe_code)]

use libfuzzer_sys::fuzz_target;
use shiin_schema::Confidence;

fuzz_target!(|text: &str| {
    let _ = shiin_classify::shell_words(text);
    let _ = shiin_classify::is_opaque(text);
    let (_, effects) = shiin_classify::classify_sql(text);
    assert!(!effects.is_empty());
    // Unsupported SQL syntax cannot become exact after normalization.
    if text.contains('\'') || text.contains(';') || text.contains('(') || text.contains("--") {
        assert!(
            effects
                .iter()
                .all(|effect| effect.confidence == Confidence::Opaque)
        );
    }
});
