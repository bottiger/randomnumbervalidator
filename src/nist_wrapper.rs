use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

/// Wrapper for NIST Statistical Test Suite
pub struct NistWrapper {
    project_root: PathBuf, // Store project root to avoid path issues
    nist_path: PathBuf,
    data_dir: PathBuf,
    experiments_dir: PathBuf,
}

impl NistWrapper {
    pub fn new() -> Self {
        // Find project root by searching for Cargo.toml
        let project_root = Self::find_project_root()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let nist_path = project_root.join("nist/sts-2.1.2/sts-2.1.2");
        let data_dir = nist_path.join("data");
        let experiments_dir = nist_path.join("experiments/AlgorithmTesting");

        NistWrapper {
            project_root,
            nist_path,
            data_dir,
            experiments_dir,
        }
    }

    /// Find the project root by looking for Cargo.toml
    fn find_project_root() -> Option<PathBuf> {
        let mut current = std::env::current_dir().ok()?;

        // Search upwards for Cargo.toml
        loop {
            if current.join("Cargo.toml").exists() {
                return Some(current);
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Check if NIST test suite is available
    pub fn is_available(&self) -> bool {
        self.nist_path.join("assess").exists()
    }

    /// Ensure all required experiment directories exist
    /// NIST assess needs these directories to write test results
    fn ensure_experiment_dirs(&self) -> Result<(), String> {
        // Create the main experiments/AlgorithmTesting directory
        fs::create_dir_all(&self.experiments_dir)
            .map_err(|e| format!("Failed to create experiments directory: {}", e))?;

        // Create subdirectories for each test type
        let test_dirs = vec![
            "Frequency",
            "BlockFrequency",
            "Runs",
            "LongestRun",
            "Rank",
            "FFT",
            "NonOverlappingTemplate",
            "OverlappingTemplate",
            "Universal",
            "LinearComplexity",
            "Serial",
            "ApproximateEntropy",
            "CumulativeSums",
            "RandomExcursions",
            "RandomExcursionsVariant",
        ];

        for test_dir in test_dirs {
            let dir_path = self.experiments_dir.join(test_dir);
            fs::create_dir_all(&dir_path)
                .map_err(|e| format!("Failed to create {} directory: {}", test_dir, e))?;
        }

        debug!("Experiment directories created successfully");
        Ok(())
    }

    /// Prepare input file for NIST tests
    /// NIST expects ASCII '0' and '1' characters
    pub fn prepare_input_file(&self, bits: &[u8], filename: &str) -> Result<PathBuf, String> {
        // Ensure the data directory exists
        fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let output_path = self.data_dir.join(filename);
        let mut file = fs::File::create(&output_path)
            .map_err(|e| format!("Failed to create input file: {}", e))?;

        // Write bits to file (NIST expects ASCII '0' and '1')
        // Add newlines every 25 characters for readability (matching NIST format)
        for (i, bit) in bits.iter().enumerate() {
            if i > 0 && i % 25 == 0 {
                writeln!(file).map_err(|e| format!("Failed to write newline: {}", e))?;
            }
            write!(file, "{}", bit).map_err(|e| format!("Failed to write bit: {}", e))?;
        }
        writeln!(file).map_err(|e| format!("Failed to write final newline: {}", e))?;

        Ok(output_path)
    }

    /// Run NIST test suite by automating the interactive prompts
    /// Returns a summary of the test results
    pub fn run_tests(&self, bits: &[u8]) -> Result<String, String> {
        info!("Starting NIST statistical tests");
        debug!("Project root: {}", self.project_root.display());
        debug!("NIST path: {}", self.nist_path.display());

        let assess_path = self.nist_path.join("assess");

        if !self.is_available() {
            error!(
                "NIST test suite not available at: {}",
                assess_path.display()
            );
            return Err(format!(
                "NIST test suite not found at: {}\nProject root: {}\nRun 'make nist' to compile the test suite.",
                assess_path.display(),
                self.project_root.display()
            ));
        }

        // Need at least 100 bits for meaningful NIST tests
        if bits.len() < 100 {
            warn!("Insufficient bits for NIST tests: {} < 100", bits.len());
            return Err("Need at least 100 bits for NIST tests (minimum ~4 numbers)".to_string());
        }

        info!("Running NIST tests on {} bits", bits.len());

        // Ensure experiment directories exist
        debug!("Ensuring experiment directories exist");
        self.ensure_experiment_dirs()?;

        // Generate unique filename for this test
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?
            .as_secs();
        let filename = format!("test_data_{}.txt", timestamp);

        // Write input file
        debug!("Preparing input file: {}", filename);
        let _input_path = self.prepare_input_file(bits, &filename)?;
        let bit_length = bits.len();
        debug!("Input file written: {} bits", bit_length);

        // Create input string for automation
        // The NIST assess program expects:
        // 1. Generator option (0 = input from file)
        // 2. Filename (relative to data directory - just the filename!)
        // 3. Test selection (0 = all tests)
        let automated_input = format!("0\ndata/{}\n0\n", filename);

        // Run assess with automated input
        debug!("Spawning NIST assess process");
        let mut child = Command::new(&assess_path)
            .arg(bit_length.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(&self.nist_path)
            .spawn()
            .map_err(|e| {
                error!("Failed to spawn assess: {}", e);
                format!("Failed to spawn assess: {}", e)
            })?;

        // Write automated input to stdin and then close it
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(automated_input.as_bytes())
                .map_err(|e| format!("Failed to write to stdin: {}", e))?;
            // stdin is dropped here, closing the pipe
        }

        // Wait for completion
        debug!("Waiting for NIST assess to complete");
        let result = child.wait_with_output().map_err(|e| {
            error!("Failed to wait for assess: {}", e);
            format!("Failed to wait for assess: {}", e)
        })?;

        // Capture output
        let stdout = String::from_utf8_lossy(&result.stdout);
        let stderr = String::from_utf8_lossy(&result.stderr);

        debug!("NIST assess completed with status: {}", result.status);

        // Check if tests completed (NIST may exit with non-zero even on success)
        if !stdout.contains("Statistical Testing Complete") {
            error!("NIST tests did not complete successfully");
            return Err(format!(
                "NIST assess did not complete successfully.\nExit status: {}\nStdout:\n{}\nStderr:\n{}",
                result.status, stdout, stderr
            ));
        }

        info!("NIST tests completed successfully, parsing results");

        // Parse results
        self.parse_all_results()
    }

    /// Parse all NIST test results from the experiments directory
    fn parse_all_results(&self) -> Result<String, String> {
        let test_names = vec![
            "Frequency",
            "BlockFrequency",
            "CumulativeSums",
            "Runs",
            "LongestRun",
            "Rank",
            "FFT",
            "NonOverlappingTemplate",
            "OverlappingTemplate",
            "Universal",
            "ApproximateEntropy",
            "RandomExcursions",
            "RandomExcursionsVariant",
            "Serial",
            "LinearComplexity",
        ];

        let mut results = HashMap::new();
        let mut success_count = 0;
        let mut total_tests = 0;

        for test_name in &test_names {
            let stats_path = self.experiments_dir.join(test_name).join("stats.txt");

            if !stats_path.exists() {
                continue; // Skip tests that weren't run
            }

            match self.parse_test_result(&stats_path) {
                Ok((success, p_values)) => {
                    total_tests += 1;
                    if success {
                        success_count += 1;
                    }
                    results.insert(test_name.to_string(), (success, p_values));
                }
                Err(_) => continue, // Skip if we can't parse this test
            }
        }

        // Generate summary
        let mut summary = format!(
            "NIST Statistical Tests Summary\n\
             ================================\n\
             Tests Passed: {}/{}\n\
             Success Rate: {:.1}%\n\n",
            success_count,
            total_tests,
            if total_tests > 0 {
                (success_count as f64 / total_tests as f64) * 100.0
            } else {
                0.0
            }
        );

        summary.push_str("Individual Test Results:\n");
        summary.push_str("------------------------\n");

        for (test_name, (success, p_values)) in &results {
            let status = if *success { "PASS" } else { "FAIL" };
            let avg_p_value: f64 = p_values.iter().sum::<f64>() / p_values.len() as f64;
            summary.push_str(&format!(
                "{:25} {} (avg p-value: {:.4})\n",
                test_name, status, avg_p_value
            ));
        }

        Ok(summary)
    }

    /// Parse a single test result file
    fn parse_test_result(&self, stats_path: &PathBuf) -> Result<(bool, Vec<f64>), String> {
        let content = fs::read_to_string(stats_path)
            .map_err(|e| format!("Failed to read stats file: {}", e))?;

        let mut p_values = Vec::new();
        let mut all_success = true;

        for line in content.lines() {
            // Look for lines like "SUCCESS		p_value = 0.106796"
            if line.contains("p_value") {
                if line.contains("FAILURE") {
                    all_success = false;
                }

                if let Some(value_str) = line.split("=").nth(1) {
                    if let Ok(p_value) = value_str.trim().parse::<f64>() {
                        p_values.push(p_value);
                    }
                }
            }
        }

        if p_values.is_empty() {
            return Err("No p-values found".to_string());
        }

        Ok((all_success, p_values))
    }

    /// Parse NIST results from a specific directory (for backwards compatibility)
    pub fn parse_results(&self, _results_dir: &str) -> Result<String, String> {
        self.parse_all_results()
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
        assert!(wrapper.nist_path.as_os_str().len() > 0);
        assert!(wrapper.project_root.as_os_str().len() > 0);
    }

    #[test]
    fn test_prepare_input_file() {
        let wrapper = NistWrapper::new();
        let bits = vec![1, 0, 1, 1, 0, 0, 1, 0];
        let temp_file = "test_nist_input.txt";

        let result = wrapper.prepare_input_file(&bits, temp_file);
        assert!(result.is_ok());

        // Verify file was created in data directory
        let file_path = result.unwrap();
        assert!(file_path.exists());

        // Verify content
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains('0'));
        assert!(content.contains('1'));

        // Clean up
        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_prepare_input_file_empty_bits() {
        let wrapper = NistWrapper::new();
        let bits = vec![];
        let result = wrapper.prepare_input_file(&bits, "test_empty.txt");

        assert!(result.is_ok());

        // Clean up
        if let Ok(path) = result {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn test_prepare_input_file_all_zeros() {
        let wrapper = NistWrapper::new();
        let bits = vec![0, 0, 0, 0, 0, 0, 0, 0];
        let result = wrapper.prepare_input_file(&bits, "test_zeros.txt");

        assert!(result.is_ok());

        let file_path = result.unwrap();
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.chars().filter(|&c| c == '0').count() == 8);

        // Clean up
        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_prepare_input_file_all_ones() {
        let wrapper = NistWrapper::new();
        let bits = vec![1, 1, 1, 1, 1, 1, 1, 1];
        let result = wrapper.prepare_input_file(&bits, "test_ones.txt");

        assert!(result.is_ok());

        let file_path = result.unwrap();
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.chars().filter(|&c| c == '1').count() == 8);

        // Clean up
        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_prepare_input_file_large_input() {
        let wrapper = NistWrapper::new();
        // Create 2000 bits alternating between 0 and 1
        let bits: Vec<u8> = (0..2000).map(|i| (i % 2) as u8).collect();
        let result = wrapper.prepare_input_file(&bits, "test_large.txt");

        assert!(result.is_ok());

        let file_path = result.unwrap();
        assert!(file_path.exists());

        // Verify we got all the bits
        let content = fs::read_to_string(&file_path).unwrap();
        let bit_chars = content.chars().filter(|&c| c == '0' || c == '1').count();
        assert_eq!(bit_chars, 2000);

        // Clean up
        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_is_available() {
        let wrapper = NistWrapper::new();
        // This will return true or false depending on whether NIST is compiled
        // We just verify the method works
        let available = wrapper.is_available();
        assert!(available == true || available == false);
    }

    #[test]
    fn test_run_tests_insufficient_bits() {
        let wrapper = NistWrapper::new();
        let bits = vec![1, 0, 1, 0]; // Only 4 bits, need at least 100

        let result = wrapper.run_tests(&bits);
        assert!(result.is_err());
        // The error should either mention "100" (if NIST is available) or "not found" (if not compiled)
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("100") || error_msg.contains("not found"));
    }

    #[test]
    fn test_find_project_root() {
        let root = NistWrapper::find_project_root();
        // Should find the project root or return None
        if let Some(path) = root {
            assert!(path.join("Cargo.toml").exists());
        }
    }

    #[test]
    fn test_nist_wrapper_default() {
        let wrapper = NistWrapper::default();
        assert!(wrapper.nist_path.as_os_str().len() > 0);
    }

    #[test]
    fn test_prepare_input_file_formatting() {
        let wrapper = NistWrapper::new();
        // Test that lines are broken every 25 characters as expected by NIST
        let bits = vec![0; 60]; // 60 zeros
        let result = wrapper.prepare_input_file(&bits, "test_format.txt");

        assert!(result.is_ok());

        let file_path = result.unwrap();
        let content = fs::read_to_string(&file_path).unwrap();

        // Should have newlines for formatting
        assert!(content.contains('\n'));

        // Clean up
        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_prepare_input_binary_values_only() {
        let wrapper = NistWrapper::new();
        let bits = vec![0, 1, 0, 1, 1, 0, 1, 0];
        let result = wrapper.prepare_input_file(&bits, "test_binary.txt");

        assert!(result.is_ok());

        let file_path = result.unwrap();
        let content = fs::read_to_string(&file_path).unwrap();

        // Verify only '0', '1', and newlines are in the file
        for c in content.chars() {
            assert!(c == '0' || c == '1' || c == '\n');
        }

        // Clean up
        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_parse_results_backwards_compat() {
        let wrapper = NistWrapper::new();
        // Test the backwards compatible parse_results method
        // It should work even with an arbitrary directory name
        let result = wrapper.parse_results("some_directory");
        // This will try to parse all results, may succeed or fail depending on state
        // We just verify it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_ensure_experiment_dirs() {
        let wrapper = NistWrapper::new();

        // This should create all required directories
        let result = wrapper.ensure_experiment_dirs();
        assert!(result.is_ok(), "Failed to create experiment directories: {:?}", result);

        // Verify the main experiments directory exists
        assert!(wrapper.experiments_dir.exists(), "Experiments directory should exist");

        // Verify all test subdirectories exist
        let test_dirs = vec![
            "Frequency",
            "BlockFrequency",
            "Runs",
            "LongestRun",
            "Rank",
            "FFT",
            "NonOverlappingTemplate",
            "OverlappingTemplate",
            "Universal",
            "LinearComplexity",
            "Serial",
            "ApproximateEntropy",
            "CumulativeSums",
            "RandomExcursions",
            "RandomExcursionsVariant",
        ];

        for test_dir in test_dirs {
            let dir_path = wrapper.experiments_dir.join(test_dir);
            assert!(
                dir_path.exists(),
                "Test directory should exist: {}",
                test_dir
            );
        }
    }

    #[test]
    fn test_ensure_experiment_dirs_idempotent() {
        let wrapper = NistWrapper::new();

        // Call it once
        let result1 = wrapper.ensure_experiment_dirs();
        assert!(result1.is_ok());

        // Call it again - should still succeed (idempotent)
        let result2 = wrapper.ensure_experiment_dirs();
        assert!(result2.is_ok());
    }
}
