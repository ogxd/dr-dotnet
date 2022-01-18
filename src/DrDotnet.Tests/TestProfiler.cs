using Microsoft.Diagnostics.NETCore.Client;
using NUnit.Framework;
using System;
using System.Diagnostics;
using System.Threading;

namespace DrDotnet.Tests
{
    public class TestProfiler
    {
        [Test]
        public void ProfilerCanAttachAndReceiveCallbacks()
        {
            string strExeFilePath = System.Reflection.Assembly.GetExecutingAssembly().Location;

            TestContext.WriteLine(strExeFilePath);

            string strWorkPath = System.IO.Path.GetDirectoryName(strExeFilePath);
            string profilerDll = System.IO.Path.Combine(strWorkPath, "Profiler.Windows.dll");

            Console.WriteLine("[ATTACHER] Looking for process...");

            ProcessStartInfo psi = new ProcessStartInfo();
            psi.RedirectStandardOutput = true;
            psi.FileName = "Fibonacci.exe";

            var process = Process.Start(psi);
            process.ErrorDataReceived += Process_OutputDataReceived;
            process.OutputDataReceived += Process_OutputDataReceived;

            ThreadPool.QueueUserWorkItem((_) => process.BeginOutputReadLine());

            Thread.Sleep(1000);

            Console.WriteLine("[ATTACHER] Attaching to process...");

            DiagnosticsClient client = new DiagnosticsClient(process.Id);
            Guid guid = new Guid("{805A308B-061C-47F3-9B30-F785C3186E84}");
            client.AttachProfiler(TimeSpan.FromSeconds(10), guid, profilerDll, null);

            Console.WriteLine("[ATTACHER] Attached!");

            Thread.Sleep(5000);
        }

        private void Process_OutputDataReceived(object sender, DataReceivedEventArgs e)
        {
            TestContext.WriteLine("[PROFILER] " + e.Data);
        }
    }
}