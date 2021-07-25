use primes::{
    print_results_stderr, report_results_stdout, FlagStorage, FlagStorageBitVector,
    FlagStorageBitVectorRotate, FlagStorageBitVectorStriped, FlagStorageByteVector, PrimeSieve,
};

use std::{
    thread,
    time::{Duration, Instant},
};
use structopt::StructOpt;

pub mod primes {
    use std::{collections::HashMap, time::Duration, usize};

    /// Validator to compare against known primes.
    /// Pulled this out into a separate struct, as it's defined
    /// `const` in C++. There are various ways to do this in Rust, including
    /// lazy_static, etc. Should be able to do the const initialisation in the future.
    pub struct PrimeValidator(HashMap<usize, usize>);

    impl Default for PrimeValidator {
        fn default() -> Self {
            let map = [
                (10, 4),   // Historical data for validating our results - the number of primes
                (100, 25), // to be found under some limit, such as 168 primes under 1000
                (1000, 168),
                (10000, 1229),
                (100000, 9592),
                (1000000, 78498),
                (10000000, 664579),
                (100000000, 5761455),
            ]
            .iter()
            .copied()
            .collect();
            PrimeValidator(map)
        }
    }

    impl PrimeValidator {
        // Return Some(true) or Some(false) if we know the answer, or None if we don't have
        // an entry for the given sieve_size.
        pub fn is_valid(&self, sieve_size: usize, result: usize) -> Option<bool> {
            if let Some(&expected) = self.0.get(&sieve_size) {
                Some(result == expected)
            } else {
                None
            }
        }

        #[cfg(test)]
        pub fn known_results_iter(&self) -> impl Iterator<Item = (&usize, &usize)> {
            self.0.iter()
        }
    }

    /// The actual sieve implementation, generic over the storage. This allows us to
    /// include the storage type we want without re-writing the algorithm each time.
    pub struct PrimeSieve<T: FlagStorage> {
        sieve_size: usize,
        flags: T,
    }

    impl<T> PrimeSieve<T>
    where
        T: FlagStorage,
    {
        pub fn new(sieve_size: usize) -> Self {
            let num_flags = sieve_size / 2 + 1;
            PrimeSieve {
                sieve_size,
                flags: T::create_true(num_flags),
            }
        }

        fn is_num_flagged(&self, number: usize) -> bool {
            if number % 2 == 0 {
                return false;
            }
            let index = number / 2;
            self.flags.get(index)
        }

        // count number of primes (not optimal, but doesn't need to be)
        pub fn count_primes(&self) -> usize {
            (1..self.sieve_size)
                .filter(|v| self.is_num_flagged(*v))
                .count()
        }

        // calculate the primes up to the specified limit
        pub fn run_sieve(&mut self) {
            let mut factor = 3;
            let q = (self.sieve_size as f32).sqrt() as usize;

            // note: need to check up to and including q, otherwise we
            // fail to catch cases like sieve_size = 1000
            while factor <= q {
                // find next factor - next still-flagged number
                factor = (factor / 2..self.sieve_size / 2)
                    .find(|n| self.flags.get(*n))
                    .unwrap()
                    * 2
                    + 1;

                // reset flags starting at `start`, every `factor`'th flag
                let start = factor * factor / 2;
                let skip = factor;
                self.flags.reset_flags(start, skip);

                factor += 2;
            }
        }
    }

    /// print results to console stderr for good feedback
    pub fn print_results_stderr<T: FlagStorage>(
        label: &str,
        prime_sieve: &PrimeSieve<T>,
        show_results: bool,
        duration: Duration,
        passes: usize,
        threads: usize,
        validator: &PrimeValidator,
    ) {
        if show_results {
            eprint!("2,");
            for num in (3..prime_sieve.sieve_size).filter(|n| prime_sieve.is_num_flagged(*n)) {
                print!("{},", num);
            }
            eprintln!();
        }

        let count = prime_sieve.count_primes();

        eprintln!(
            "{:15} Passes: {}, Threads: {}, Time: {:.10}, Average: {:.10}, Limit: {}, Counts: {}, Valid: {}",
            label,
            passes,
            threads,
            duration.as_secs_f32(),
            duration.as_secs_f32() / passes as f32,
            prime_sieve.sieve_size,
            count,
            match validator.is_valid(prime_sieve.sieve_size, count) {
                Some(true) => "Pass",
                Some(false) => "Fail",
                None => "Unknown"
            }
        );
    }

    /// print correctly-formatted results to `stderr` as per CONTRIBUTING.md
    /// - format is <name>;<iterations>;<total_time>;<num_threads>
    pub fn report_results_stdout(
        label: &str,
        bits_per_prime: usize,
        duration: Duration,
        passes: usize,
        threads: usize,
    ) {
        println!(
            "mike-barber_{};{};{:.10};{};algorithm=base,faithful=yes,bits={}",
            label,
            passes,
            duration.as_secs_f32(),
            threads,
            bits_per_prime
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primes::PrimeValidator;

    #[test]
    fn sieve_known_correct_bits() {
        sieve_known_correct::<FlagStorageBitVector>();
    }

    #[test]
    fn sieve_known_correct_bits_rolling() {
        sieve_known_correct::<FlagStorageBitVectorRotate>();
    }

    #[test]
    fn sieve_known_correct_bits_striped() {
        sieve_known_correct::<FlagStorageBitVectorStriped>();
    }

    #[test]
    fn sieve_known_correct_bytes() {
        sieve_known_correct::<FlagStorageByteVector>();
    }

    fn sieve_known_correct<T: FlagStorage>() {
        let validator = PrimeValidator::default();
        for (sieve_size, expected_primes) in validator.known_results_iter() {
            let mut sieve: PrimeSieve<T> = primes::PrimeSieve::new(*sieve_size);
            sieve.run_sieve();
            assert_eq!(
                *expected_primes,
                sieve.count_primes(),
                "wrong number of primes for sieve = {}",
                sieve_size
            );
        }
    }
}
