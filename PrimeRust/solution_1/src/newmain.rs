use primes::{
    print_results_stderr, report_results_stdout, FlagStorage, FlagStorageBitVector,
    FlagStorageBitVectorRotate, FlagStorageBitVectorStriped, FlagStorageByteVector, PrimeSieve,
};

use std::{
    thread,
    time::{Duration, Instant},
};
use structopt::StructOpt;

/// Rust program to calculate number of primes under a given limit.
#[derive(StructOpt, Debug)]
#[structopt(name = "abstracted")]
struct CommandLineOptions {
    /// Number of threads. If not specified, do two runs for both
    /// single threaded case and maximum concurrency.
    #[structopt(short, long)]
    threads: Option<usize>,

    /// Run duration
    #[structopt(short, long, default_value = "5")]
    seconds: u64,

    /// Prime sieve limit -- count primes that occur under or equal to this number.
    /// If you want this compared with known results, pick an order of 10: 10,100,...100000000
    #[structopt(short, long, default_value = "1000000")]
    limit: usize,

    /// Number of times to run the experiment
    #[structopt(short, long, default_value = "1")]
    repetitions: usize,

    /// Print out all primes found
    #[structopt(short, long)]
    print: bool,

    /// Run variant that uses bit-level storage
    #[structopt(long)]
    bits: bool,

    /// Run variant that uses bit-level storage, applied using rotate
    #[structopt(long)]
    bits_rotate: bool,

    /// Run variant that uses bit-level storage, using striped storage
    #[structopt(long)]
    bits_striped: bool,

    /// Run variant that uses byte-level storage
    #[structopt(long)]
    bytes: bool,
}

fn main() {
    // command line options are handled by the `structopt` and `clap` crates, which
    // makes life very pleasant indeed. At the cost of a bit of compile time :)
    let opt = CommandLineOptions::from_args();

    let limit = opt.limit;
    let repetitions = opt.repetitions;
    let run_duration = Duration::from_secs(opt.seconds);

    let thread_options = match opt.threads {
        Some(t) => vec![t],
        None => vec![1, num_cpus::get()],
    };

    // run all implementations if no options are specified (default)
    let run_all = [opt.bits, opt.bits_rotate, opt.bits_striped, opt.bytes]
        .iter()
        .all(|b| !*b);

    for threads in thread_options {
        if opt.bytes || run_all {
            thread::sleep(Duration::from_secs(1));
            print_header(threads, limit, run_duration);
            for _ in 0..repetitions {
                run_implementation::<FlagStorageByteVector>(
                    "byte-storage",
                    8,
                    run_duration,
                    threads,
                    limit,
                    opt.print,
                );
            }
        }

        if opt.bits || run_all {
            thread::sleep(Duration::from_secs(1));
            print_header(threads, limit, run_duration);
            for _ in 0..repetitions {
                run_implementation::<FlagStorageBitVector>(
                    "bit-storage",
                    1,
                    run_duration,
                    threads,
                    limit,
                    opt.print,
                );
            }
        }

        if opt.bits_rotate || run_all {
            thread::sleep(Duration::from_secs(1));
            print_header(threads, limit, run_duration);
            for _ in 0..repetitions {
                run_implementation::<FlagStorageBitVectorRotate>(
                    "bit-storage-rotate",
                    1,
                    run_duration,
                    threads,
                    limit,
                    opt.print,
                );
            }
        }

        if opt.bits_striped || run_all {
            thread::sleep(Duration::from_secs(1));
            print_header(threads, limit, run_duration);
            for _ in 0..repetitions {
                run_implementation::<FlagStorageBitVectorStriped>(
                    "bit-storage-striped",
                    1,
                    run_duration,
                    threads,
                    limit,
                    opt.print,
                );
            }
        }
    }
}

fn print_header(threads: usize, limit: usize, run_duration: Duration) {
    eprintln!();
    eprintln!(
        "Computing primes to {} on {} thread{} for {} second{}.",
        limit,
        threads,
        match threads {
            1 => "",
            _ => "s",
        },
        run_duration.as_secs(),
        match run_duration.as_secs() {
            1 => "",
            _ => "s",
        }
    );
}

fn run_implementation<T: 'static + FlagStorage + Send>(
    label: &str,
    bits_per_prime: usize,
    run_duration: Duration,
    num_threads: usize,
    limit: usize,
    print_primes: bool,
) {
    // spin up N threads; each will terminate itself after `run_duration`, returning
    // the last sieve as well as the total number of counts.
    let start_time = Instant::now();
    let threads: Vec<_> = (0..num_threads)
        .map(|_| {
            std::thread::spawn(move || {
                let mut local_passes = 0;
                let mut last_sieve = None;
                while (Instant::now() - start_time) < run_duration {
                    let mut sieve: PrimeSieve<T> = primes::PrimeSieve::new(limit);
                    sieve.run_sieve();
                    last_sieve.replace(sieve);
                    local_passes += 1;
                }
                // return local pass count and last sieve
                (local_passes, last_sieve)
            })
        })
        .collect();

    // wait for threads to finish, and record end time
    let results: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
    let end_time = Instant::now();

    // get totals and print results based on one of the sieves
    let total_passes = results.iter().map(|r| r.0).sum();
    let check_sieve = &results.first().unwrap().1;
    if let Some(sieve) = check_sieve {
        let duration = end_time - start_time;
        // print results to stderr for convenience
        print_results_stderr(
            label,
            &sieve,
            print_primes,
            duration,
            total_passes,
            num_threads,
            &primes::PrimeValidator::default(),
        );
        // and report results to stdout for reporting
        report_results_stdout(label, bits_per_prime, duration, total_passes, num_threads);
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
