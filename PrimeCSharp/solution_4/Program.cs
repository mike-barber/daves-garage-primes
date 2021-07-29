using System;
using System.Diagnostics;
using System.Globalization;
using System.Linq;
using System.Threading.Tasks;

namespace Solution4
{
    public class Program
    {
        const int _sieveSize = 1_000_000;

        static void Main()
        {
            CultureInfo.CurrentCulture = CultureInfo.InvariantCulture;

            for (var i = 0; i < 3; ++i)
                RunSingleThreaded();

            for (var i = 0; i < 3; ++i)
                RunMultiThreaded();

            RunTests();
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

        static void RunSingleThreaded()
        {
            var stopTicks = Stopwatch.Frequency * 5;
            var sw = Stopwatch.StartNew();

            var count = 0;
            while (sw.ElapsedTicks < stopTicks)
            {
                using var sieve = new PrimeSieve(_sieveSize);
                sieve.RunSieve();
                ++count;
            }
            sw.Stop();

            using var modelSieve = new PrimeSieve(_sieveSize);
            modelSieve.RunSieve();
            PrintReport("st", modelSieve, 1, sw.ElapsedMilliseconds / 1000.0, count);
        }

        static void RunMultiThreaded()
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
                        using var sieve = new PrimeSieve(_sieveSize);
                        sieve.RunSieve();
                        ++localCount;
                    }
                    return localCount;
                });
            }
            // wait for all tasks to complete, then stop the timer and collect counts
            Task.WaitAll(tasks);
            sw.Stop();
            var count = tasks.Select(t => t.GetAwaiter().GetResult()).Sum();

            using var modelSieve = new PrimeSieve(_sieveSize);
            modelSieve.RunSieve();
            PrintReport("mt", modelSieve, numThreads, sw.ElapsedMilliseconds / 1000.0, count);
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
