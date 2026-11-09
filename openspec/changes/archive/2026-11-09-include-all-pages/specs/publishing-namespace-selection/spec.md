## ADDED Requirements

### Requirement: Select every page

The SDK SHALL let a caller turn any valid policy into one that selects every page. Such a policy SHALL select every valid logical page name that no excluded root matches, whatever its inclusion list holds. Exclusion SHALL still win, and a page identity with an empty or whitespace-only segment or a control character SHALL stay unselected.

#### Scenario: Every page with an exclusion

- **WHEN** a policy with no included roots and the excluded root `My/Private` selects every page
- **THEN** `My`, `Other`, `SolveSpace` and `研究/庭/花` are selected while `My/Private` and `My/Private/Note` are not

#### Scenario: Included roots change nothing

- **WHEN** a policy with the included root `My` and the excluded root `My/Private` selects every page
- **THEN** each candidate is selected exactly when it is selected by the policy with no included roots, the same excluded root and every page selected

#### Scenario: Invalid candidate under every page

- **WHEN** a policy selects every page and the candidate is `My//AI` or contains a control character
- **THEN** the candidate is unselected
