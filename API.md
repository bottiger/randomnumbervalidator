# Random Number Validator API

A public API for validating the quality of random numbers using NIST statistical tests.

## Base URL

- **Production**: `https://your-domain.com`
- **Local Development**: `http://localhost:3000`

## Interactive Documentation

Visit `/docs` for interactive API documentation where you can test endpoints directly in your browser:

- **Production**: `https://your-domain.com/docs`
- **Local**: `http://localhost:3000/docs`

## Endpoints

### POST `/api/validate`

Validates the quality of random numbers using the NIST Statistical Test Suite.

#### Request Body

```json
{
  "numbers": "string",
  "input_format": "numbers" | "base64",
  "range_min": number | null,
  "range_max": number | null,
  "bit_width": 8 | 16 | 32 | null,
  "debug_log": boolean
}
```

**Fields:**

- `numbers` (required): The random numbers to validate
  - For `input_format: "numbers"`: comma/space/newline separated numbers
  - For `input_format: "base64"`: base64-encoded binary data

- `input_format` (optional, default: `"numbers"`): Format of the input data
  - `"numbers"`: Numeric sequences
  - `"base64"`: Binary data encoded in base64

- `range_min` (optional): Minimum value of your RNG range
  - Required for custom ranges that don't fit standard bit widths (0-255, 0-65535, 0-4294967295)
  - Example: For dice rolls (1-6), set `range_min: 1`

- `range_max` (optional): Maximum value of your RNG range
  - Required for custom ranges
  - Example: For dice rolls (1-6), set `range_max: 6`

- `bit_width` (optional): Enforce a specific bit-width (8, 16, or 32)
  - Only valid for `input_format: "numbers"`
  - All numbers must fit within this bit-width

- `debug_log` (optional, default: `false`): Enable debug logging
  - Writes the converted bit stream to a timestamped file

#### Response

```json
{
  "valid": boolean,
  "quality_score": number,
  "message": "string",
  "nist_data": {
    "bit_count": number,
    "tests_passed": number,
    "total_tests": number,
    "success_rate": number,
    "individual_tests": [
      {
        "name": "string",
        "passed": boolean,
        "p_value": number,
        "p_values": [number],
        "description": "string",
        "metrics": [["string", "string"]] | null
      }
    ],
    "fallback_message": "string" | null,
    "raw_output": "string" | null
  },
  "nist_results": "string" | null,
  "debug_file": "string" | null
}
```

**Fields:**

- `valid`: Whether the random numbers are considered valid (quality_score >= 0.8)
- `quality_score`: Quality score from 0.0 to 1.0 based on NIST test success rate
- `message`: Human-readable summary message
- `nist_data`: Detailed NIST test results
  - `bit_count`: Total number of bits analyzed
  - `tests_passed`: Number of tests that passed
  - `total_tests`: Total number of tests executed
  - `success_rate`: Percentage of tests that passed (0.0 to 100.0)
  - `individual_tests`: Array of individual test results
- `debug_file`: Path to debug file (only if debug_log was enabled)

## Examples

### Example 1: Basic Number Validation

Test a simple sequence of numbers:

```bash
curl -X POST http://localhost:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{
    "numbers": "1,2,3,4,5,6,7,8,9,10,11,12,13,14,15"
  }'
```

**Response:**

```json
{
  "valid": false,
  "quality_score": 0.45,
  "message": "Analyzed 120 bits using 15 NIST tests (7/15 passed)",
  "nist_data": {
    "bit_count": 120,
    "tests_passed": 7,
    "total_tests": 15,
    "success_rate": 46.67,
    "individual_tests": [...]
  }
}
```

### Example 2: Custom Range (Dice Rolls)

Validate dice roll results (1-6):

```bash
curl -X POST http://localhost:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{
    "numbers": "3,1,4,6,2,5,3,1,6,4,2,5,3,6,1,4,5,2",
    "range_min": 1,
    "range_max": 6
  }'
```

### Example 3: Large Dataset with Specific Bit Width

Test random bytes (0-255) with 8-bit width:

```bash
curl -X POST http://localhost:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{
    "numbers": "142,73,208,91,45,167,234,88,19,203,...",
    "bit_width": 8
  }'
```

### Example 4: Base64-Encoded Binary Data

Test base64-encoded random bytes:

```bash
curl -X POST http://localhost:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{
    "numbers": "SGVsbG8gV29ybGQhIFRoaXMgaXMgYmFzZTY0IGVuY29kZWQgZGF0YQ==",
    "input_format": "base64"
  }'
```

### Example 5: Using JavaScript/TypeScript

```typescript
async function validateRandomNumbers(numbers: string) {
  const response = await fetch('http://localhost:3000/api/validate', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      numbers: numbers,
      input_format: 'numbers',
    }),
  });

  const result = await response.json();

  console.log(`Valid: ${result.valid}`);
  console.log(`Quality Score: ${result.quality_score}`);
  console.log(`Tests Passed: ${result.nist_data.tests_passed}/${result.nist_data.total_tests}`);

  return result;
}

// Usage
const numbers = Array.from({length: 1000}, () =>
  Math.floor(Math.random() * 256)
).join(',');

validateRandomNumbers(numbers);
```

### Example 6: Using Python

```python
import requests
import random

def validate_random_numbers(numbers):
    url = 'http://localhost:3000/api/validate'
    payload = {
        'numbers': numbers,
        'input_format': 'numbers'
    }

    response = requests.post(url, json=payload)
    result = response.json()

    print(f"Valid: {result['valid']}")
    print(f"Quality Score: {result['quality_score']}")
    print(f"Tests Passed: {result['nist_data']['tests_passed']}/{result['nist_data']['total_tests']}")

    return result

# Generate 1000 random numbers (0-255)
numbers = ','.join(str(random.randint(0, 255)) for _ in range(1000))
validate_random_numbers(numbers)
```

### Example 7: Debug Mode

Enable debug logging to inspect the bit stream:

```bash
curl -X POST http://localhost:3000/api/validate \
  -H "Content-Type: application/json" \
  -d '{
    "numbers": "42,17,89,123,255,0,128,64",
    "bit_width": 8,
    "debug_log": true
  }'
```

**Response includes:**

```json
{
  "valid": false,
  "quality_score": 0.33,
  "message": "...",
  "debug_file": "debug/bits_20250107_143052_123456.txt",
  "nist_data": {...}
}
```

## Understanding the Results

### Quality Score

- **0.8 - 1.0**: Excellent quality randomness (valid)
- **0.6 - 0.8**: Fair quality, some bias detected
- **0.0 - 0.6**: Poor quality, significant bias detected

### P-Values

Each NIST test returns a p-value (0.0 to 1.0):

- **p-value >= 0.01**: Test passed (data appears random)
- **p-value < 0.01**: Test failed (non-random pattern detected)

Higher p-values generally indicate more random-like behavior.

### Common Test Results

- **Frequency**: Tests if the number of 0s and 1s are approximately equal
- **BlockFrequency**: Tests frequency within M-bit blocks
- **Runs**: Tests the number of runs (uninterrupted sequences of identical bits)
- **LongestRun**: Tests the longest run of ones
- **Rank**: Tests the rank of binary matrices
- **FFT**: Tests periodic features using Fast Fourier Transform
- **ApproximateEntropy**: Tests the frequency of overlapping patterns

## Error Responses

### Invalid Input Format

```json
{
  "valid": false,
  "quality_score": 0.0,
  "message": "Input contains letters - only numbers and delimiters are allowed",
  "nist_data": null
}
```

### Range Validation Error

```json
{
  "valid": false,
  "quality_score": 0.0,
  "message": "Numbers (50-100) outside specified range (1-50)",
  "nist_data": null
}
```

### Insufficient Data

```json
{
  "valid": false,
  "quality_score": 0.0,
  "message": "No numbers provided",
  "nist_data": null
}
```

## Best Practices

1. **Sample Size**: Provide at least 1000-10000 numbers for reliable results
2. **Specify Range**: For non-standard ranges, always specify `range_min` and `range_max`
3. **Consistent Format**: Keep your number format consistent within a request
4. **Rate Limiting**: Be mindful of server load; the API performs compute-intensive statistical analysis
5. **Caching**: Results for identical inputs will be the same; consider client-side caching

## CORS

The API has permissive CORS settings enabled, allowing requests from any origin.

## Rate Limits

Currently, there are no enforced rate limits, but please use the API responsibly. Heavy usage may be subject to throttling in the future.

## Support

For issues, questions, or feature requests, please visit:
- GitHub: [Your Repository URL]
- Documentation: [Your Docs URL]

## Version

Current API version: `0.1.0`

For changelog and version history, see the main repository README.
