using System;
using System.Diagnostics;
using System.Globalization;
using System.Linq;
using System.Threading.Tasks;

namespace Solution4
{
    public class Program
    {
        const int _defaultSieveSize = 1_000_000;

        async static Task Main()
        {
            CultureInfo.CurrentCulture = CultureInfo.InvariantCulture;
            RunTests();
            Warmup();

            //for (var i = 0; i < 3; ++i)
            //    RunSingleThreaded(report: true);

            //for (var i = 0; i < 3; ++i)
            //    await RunMultiThreaded(report: true);

            RunSingleThreaded(_defaultSieveSize, report: true);
            await RunMultiThreaded(_defaultSieveSize, report: true);
        }

        static void RunTests()
        {
            foreach (var size in KnownPrimes.KnownSizes)
            {
                using var sieve = new PrimeSieve(size);
                sieve.RunSieve();
                Trace.Assert(sieve.IsValid);
                Console.Error.WriteLine($"Sieve passed test: {size}");
            }
        }

        static void Warmup()
        {
            Console.Error.WriteLine("Warming up...");
            RunSingleThreaded(_defaultSieveSize, report: false);
        }

        static void RunSingleThreaded(int sieveSize, bool report)
        {
            // TODO: I think DateTime.UtcNow is actually faster than hitting
            // Stopwatch repeatedly. Should probably change this.
            var stopTicks = Stopwatch.Frequency * 5;
            var sw = Stopwatch.StartNew();

            var count = 0;
            while (sw.ElapsedTicks < stopTicks)
            {
                using var sieve = new PrimeSieve(sieveSize);
                sieve.RunSieve();
                ++count;
            }
            sw.Stop();

            if (report)
            {
                using var modelSieve = new PrimeSieve(sieveSize);
                modelSieve.RunSieve();
                PrintReport("st", modelSieve, 1, sw.ElapsedMilliseconds / 1000.0, count);
            }
        }

        async static Task RunMultiThreaded(int sieveSize, bool report)
        {
            var stopTicks = Stopwatch.Frequency * 5;
            var sw = Stopwatch.StartNew();

            var numThreads = Environment.ProcessorCount;

            // start `numThreads` tasks
            var tasks = new Task<int>[numThreads];
            for (var i = 0; i < tasks.Length; ++i)
            {
                tasks[i] = Task.Run(() =>
                {
                    var localCount = 0;
                    while (sw.ElapsedTicks < stopTicks)
                    {
                        using var sieve = new PrimeSieve(sieveSize);
                        sieve.RunSieve();
                        ++localCount;
                    }
                    return localCount;
                });
            }
            // wait for all tasks to complete, then stop the timer and collect counts
            await Task.WhenAll(tasks);
            sw.Stop();
            var count = tasks.Select(t => t.Result).Sum();

            if (report)
            {
                using var modelSieve = new PrimeSieve(sieveSize);
                modelSieve.RunSieve();
                PrintReport("mt", modelSieve, numThreads, sw.ElapsedMilliseconds / 1000.0, count);
            }
        }


        static void PrintReport(string label, PrimeSieve sieve, int threads, double elapsedSeconds, int passes)
        {
            if (!sieve.IsValid)
            {
                throw new ApplicationException($"Sieve validation failed. Count of primes is {sieve.CountPrimes()}");
            }
            Console.WriteLine($"mike-barber_csharp_{label};{passes};{elapsedSeconds:0.000000};{threads};algorithm=base,faithful=yes,bits=1");
        }
    }
}
