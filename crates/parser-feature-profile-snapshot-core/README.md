# adze-parser-feature-profile-snapshot-core

SRP microcrate that owns `ParserFeatureProfileSnapshot`, a serializable value object for parser feature flags.

## Responsibility

- Capture parser feature flags in a stable snapshot shape
- Convert to/from `adze_feature_policy_core::ParserFeatureProfile`
- Resolve backend selection helpers from snapshot state

## Non-responsibilities

- BDD governance progress tracking
- Parse-table metadata envelope types
