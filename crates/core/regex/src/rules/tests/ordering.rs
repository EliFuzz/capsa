use super::super::{RULE_COUNT, RULES, VECTORSCAN_PATTERN_IDS, VECTORSCAN_PATTERNS};

#[test]
fn vectorscan_rule_arrays_share_ordering() {
    assert_eq!(RULES.len(), RULE_COUNT);
    assert_eq!(VECTORSCAN_PATTERNS.len(), RULE_COUNT);
    assert_eq!(VECTORSCAN_PATTERN_IDS.len(), RULE_COUNT);

    for (index, rule) in RULES.iter().enumerate() {
        assert_eq!(rule.id as usize, index);
        assert_eq!(VECTORSCAN_PATTERN_IDS[index] as usize, index);
    }
}
