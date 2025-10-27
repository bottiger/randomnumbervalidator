use std::fs;
use std::io::Write;
use std::path::Path;

/// Wrapper for NIST Statistical Test Suite
pub struct NistWrapper {
    nist_path: String,
}

impl NistWrapper {
    pub fn new() -> Self {
        NistWrapper {
            nist_path: "nist/sts-2.1.2/sts-2.1.2".to_string(),
        }
    }

    /// Check if NIST test suite is available
    pub fn is_available(&self) -> bool {
        Path::new(&self.nist_path).join("assess").exists()
    }

    /// Prepare input file for NIST tests
    /// NIST expects binary data in a specific format
    pub fn prepare_input_file(&self, bits: &[u8], output_path: &str) -> Result<(), String> {
        let mut file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create input file: {}", e))?;

        // Write bits to file (NIST expects ASCII '0' and '1')
        for bit in bits {
            write!(file, "{}", bit)
                .map_err(|e| format!("Failed to write bit: {}", e))?;
        }

        Ok(())
    }

    /// Run NIST test suite (stub implementation)
    /// This would require interactive input to the NIST program,
    /// so for now we just check if it's available
    pub fn run_tests(&self, _input_file: &str) -> Result<String, String> {
        if !self.is_available() {
            return Ok("NIST test suite not compiled. Run 'make' in nist/sts-2.1.2/sts-2.1.2/ to enable.".to_string());
        }

        // Note: The actual NIST assess program is interactive
        // A full implementation would need to automate the interactive prompts
        // or modify the NIST source code to accept command-line arguments
        Ok("NIST test suite available but requires interactive setup. Future implementation will automate this.".to_string())
    }

    /// Parse NIST results from output files
    pub fn parse_results(&self, _results_dir: &str) -> Result<String, String> {
        // NIST outputs results to experiments/AlgorithmTesting/
        // This is a stub that would parse the results files
        Ok("Results parsing not yet implemented".to_string())
    }
}

impl Default for NistWrapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nist_wrapper_creation() {
        let wrapper = NistWrapper::new();
        // Just verify we can create the wrapper
        assert!(!wrapper.nist_path.is_empty());
    }

    #[test]
    fn test_prepare_input_file() {
        let wrapper = NistWrapper::new();
        let bits = vec![1, 0, 1, 1, 0, 0, 1, 0];
        let temp_file = "/tmp/test_nist_input.txt";

        let result = wrapper.prepare_input_file(&bits, temp_file);
        assert!(result.is_ok());

        // Clean up
        let _ = fs::remove_file(temp_file);
    }
}
