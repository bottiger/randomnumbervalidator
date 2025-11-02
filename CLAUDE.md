# Overview

This is a simple application to validate a sequence of random numbers.

* The backend logic is written in Rust
* There is a simple frontend webinterface. 
** With a form to input a sequence of random numbers
** A presentation of the quality of those random numbers
* nist/ contains NIST testsuite for random numbers
** We do not touch this test-suite, but we parse for prepare the input numbers for the tests and present the reuslts of the test.

* Always run tests after making changes to the codebase
* Do not ask me som many questions about running commands. Just run them unless you are going to delete a large amount of files
* for every feature you add, add one or more tests for it