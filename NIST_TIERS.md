# NIST Test Tiers

The NIST test system now uses a tiered approach based on input data size. This provides flexibility and clear guidance about what tests can run with your data.

## Tier System Overview

| Tier | Name | Min Bits | Min Numbers (32-bit) | Tests Available |
|------|------|----------|---------------------|-----------------|
| 1 | Minimal | 100 | 3 | Basic tests only |
| 2 | Light | 1,000 | 31 | Basic + Block tests |
| 3 | Standard | 10,000 | 313 | Most NIST tests |
| 4 | Full | 100,000 | 3,125 | Complete NIST suite |
| 5 | Comprehensive | 1,000,000 | 31,250 | Optimal reliability |

## Tier 1: Minimal (100+ bits)
**Tests Available: 5**
- Frequency (Monobit)
- Runs
- FFT (Spectral)
- Cumulative Sums (Forward)
- Cumulative Sums (Reverse)

**Use Case**: Quick validation of very small datasets
**Note**: Universal test requires 1,000+ bits and is available starting at Tier 2

## Tier 2: Light (1,000+ bits)
**Tests Available: ~10**
- All Tier 1 tests, plus:
- Universal
- Block Frequency
- Non-Overlapping Template (multiple variants)
- Overlapping Template

**Use Case**: Basic randomness testing for small RNG outputs

## Tier 3: Standard (10,000+ bits)
**Tests Available: ~15**
- All Tier 2 tests, plus:
- Longest Run of Ones
- Rank
- Approximate Entropy
- Serial Test (2 variants)

**Use Case**: Comprehensive testing for moderate-sized datasets

## Tier 4: Full (100,000+ bits)
**Tests Available: ~40+**
- All Tier 3 tests, plus:
- Random Excursions (8 variants)
- Random Excursions Variant (18 variants)
- Linear Complexity

**Use Case**: Full NIST test suite with reliable statistics

## Tier 5: Comprehensive (1,000,000+ bits)
**Tests Available: All (~40+)**
- Same as Tier 4, but with maximum statistical reliability
- Recommended for cryptographic RNG validation

**Use Case**: Production-grade validation with optimal confidence

## How It Works

1. **Automatic Detection**: The system automatically determines your tier based on input size
2. **Graceful Degradation**: Runs all tests appropriate for your data size
3. **Clear Feedback**: Results show your current tier and requirements for next tier
4. **No Hard Limits**: Unlike the old system, you can now run tests on datasets as small as 100 bits

## Example Output

```
NIST Statistical Test Suite - Results
======================================

Dataset: 50,000 bits
Test Tier: Level 3 - Standard (Most NIST tests)

Overall: 14/15 tests passed (93.3%)

Test Coverage:
-------------
Current: Tier 3 (Most NIST tests) - 15 tests run
Next Tier: Level 4 (Full) requires 100,000 bits (~3,125 numbers)
```

## Migration Notes

- **Old behavior** (100k minimum) preserved at Tier 4
- **New flexibility**: Can now test datasets as small as 100 bits
- **Better UX**: Users see exactly which tests ran and why
- **Upgrade path**: Clear guidance on how to reach next tier

## Benefits

1. **Lower barrier to entry**: Users with small datasets can still get NIST validation
2. **Better feedback**: Users understand test coverage limitations
3. **Scalable**: System automatically unlocks more tests as data size increases
4. **Educational**: Users learn about NIST test requirements naturally
