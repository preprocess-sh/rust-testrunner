# Test Runner

## What is this?
Preprocess supports code challenges that are validated by a testsuite for completion.

This is achieved by using a Lambda function that takes the user's code as input along with the pre-defined testsuite that is attached to the Challenge the user has completed.

This repository contains the source of the functions that are used to manage testruns. They are responsible for delegating the job of processing to a testrunner.

## Which languages does it support?
Preprocess is currently supporting code challenges based on C, C++, Go, Python, PHP, Ruby and Rust.

Support is planned for the following languages:
- Haskell
- Julia
- Lisp
- Perl
- R

## How does the client know when the tests are complete?
There is a separate function with the responsibilty of managing a queue of test tasks, that will then invoke the actual testrunner for the language that the user's code is written in.

The client then polls another function every couple seconds to see if the tests have completed.

DynamoDB is used for managing state between these functions.

